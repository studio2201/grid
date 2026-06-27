use crate::state::AppState;
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use shared_assets::auth::{is_locked_out, record_attempt, reset_attempts};
use shared_assets::server::get_client_ip;
use std::net::SocketAddr;
use std::time::Duration;

pub const COOKIE_NAME: &str = "GRID_PIN";

pub async fn is_authenticated(headers: &HeaderMap, state: &AppState) -> bool {
    let pin = match &state.config.server.pin {
        Some(p) => p,
        None => return true,
    };

    let cookie_pin = headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c_str| {
            c_str
                .split(';')
                .find(|s| s.trim().starts_with(&format!("{}=", COOKIE_NAME)))
                .and_then(|s| s.split('=').nth(1))
                .map(|s| s.trim().to_string())
        });

    let header_pin = headers.get("x-pin").and_then(|h| h.to_str().ok());

    match (cookie_pin, header_pin) {
        (Some(cookie), _) => state.active_sessions.read().await.contains(&cookie),
        // `constant_time_eq` returns false in constant time regardless of
        // where the first byte mismatches. This avoids the byte-by-byte
        // timing leak of a naive `==` compare.
        (None, Some(hdr)) => constant_time_eq::constant_time_eq(
            hdr.as_bytes(),
            pin.as_bytes(),
        ),
        (None, None) => false,
    }
}

pub async fn require_pin(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !is_authenticated(req.headers(), &state).await {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

pub async fn origin_validation_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let origins_env = &state.config.server.allowed_origins;
    if origins_env == "*" {
        return Ok(next.run(req).await);
    }

    let referer = req.headers().get("referer").and_then(|v| v.to_str().ok());
    let host = req.headers().get("host").and_then(|v| v.to_str().ok());

    let origin = if let Some(ref_val) = referer {
        if let Ok(url) = reqwest::Url::parse(ref_val) {
            url.origin().ascii_serialization()
        } else {
            ref_val.to_string()
        }
    } else if let Some(host_val) = host {
        let proto = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http");
        format!("{}://{}", proto, host_val)
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    let allowed_list: Vec<String> = origins_env
        .split(',')
        .map(|s| {
            let s_trim = s.trim();
            if let Ok(url) = reqwest::Url::parse(s_trim) {
                url.origin().ascii_serialization()
            } else {
                s_trim.to_string()
            }
        })
        .collect();

    let normalized_origin = if let Ok(url) = reqwest::Url::parse(&origin) {
        url.origin().ascii_serialization()
    } else {
        origin.clone()
    };

    if allowed_list.contains(&normalized_origin) {
        Ok(next.run(req).await)
    } else {
        tracing::warn!("Blocked request from origin: {}", origin);
        Err(StatusCode::FORBIDDEN)
    }
}

// NOTE: `security_headers_middleware` was removed in favour of
// `shared_assets::middleware::security_headers_layer` (see main.rs).
// Grid's previous CSP permitted inline scripts and `unsafe-eval`, which is
// the same posture shared-assets uses so Yew CSR works; do not tighten
// without auditing the frontend.

pub async fn origin_validation_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let origins_env = &state.config.server.allowed_origins;
    if origins_env == "*" {
        return Ok(next.run(req).await);
    }

    let referer = req.headers().get("referer").and_then(|v| v.to_str().ok());
    let host = req.headers().get("host").and_then(|v| v.to_str().ok());

    let origin = if let Some(ref_val) = referer {
        if let Ok(url) = reqwest::Url::parse(ref_val) {
            url.origin().ascii_serialization()
        } else {
            ref_val.to_string()
        }
    } else if let Some(host_val) = host {
        let proto = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http");
        format!("{}://{}", proto, host_val)
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    let allowed_list: Vec<String> = origins_env
        .split(',')
        .map(|s| {
            let s_trim = s.trim();
            if let Ok(url) = reqwest::Url::parse(s_trim) {
                url.origin().ascii_serialization()
            } else {
                s_trim.to_string()
            }
        })
        .collect();

    let normalized_origin = if let Ok(url) = reqwest::Url::parse(&origin) {
        url.origin().ascii_serialization()
    } else {
        origin.clone()
    };

    if allowed_list.contains(&normalized_origin) {
        Ok(next.run(req).await)
    } else {
        tracing::warn!("Blocked request from origin: {}", origin);
        Err(StatusCode::FORBIDDEN)
    }
}

#[derive(serde::Deserialize)]
pub struct VerifyPinPayload {
    pub pin: Option<String>,
}

pub fn generate_session_id() -> String {
    use std::fs::File;
    use std::io::Read;
    let file = File::open("/dev/urandom").ok();
    let mut bytes = [0u8; 16];
    if let Some(mut f) = file
        && f.read_exact(&mut bytes).is_ok()
    {
        return bytes.iter().map(|b| format!("{:02x}", b)).collect();
    }
    let random_val = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(random_val.to_string().as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn verify_pin(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(payload): Json<VerifyPinPayload>,
) -> impl IntoResponse {
    let pin_req = &state.config.server.pin;
    if pin_req.is_none() {
        return (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response();
    }

    let ip = get_client_ip(
        &headers,
        addr,
        state.config.server.trust_proxy,
        &state.config.server.trusted_proxies,
    );

    // Lockout is tracked in shared-assets' process-global state and keyed
    // by string IP. Grid's `IpAddr` -> String conversion is `Display`-based.
    let ip_str = ip.to_string();
    let lockout_dur = Duration::from_secs(state.config.server.lockout_time_minutes * 60);
    if is_locked_out(&ip_str, state.config.server.max_attempts, lockout_dur) {
        let secs_remaining = shared_assets::auth::lockout_remaining_secs(&ip_str, lockout_dur);
        let minutes_remaining = (secs_remaining / 60).max(1);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Too many attempts. Please try again in {} minute(s).", minutes_remaining)
            })),
        )
            .into_response();
    }

    let expected_pin = pin_req.as_ref().unwrap();
    let pin_str = payload.pin.as_deref().unwrap_or("").trim();

    if pin_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "PIN is required." })),
        )
            .into_response();
    }

    if constant_time_eq::constant_time_eq(pin_str.as_bytes(), expected_pin.as_bytes()) {
        reset_attempts(&ip_str);

        let session_id = generate_session_id();
        state
            .active_sessions
            .write()
            .await
            .insert(session_id.clone());

        let cookie_max_age = Duration::from_secs((state.config.server.cookie_max_age_hours * 3600) as u64);
        let secure = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("https"))
            .unwrap_or_else(|| state.config.server.base_url.starts_with("https"));

        let cookie_val = format!(
            "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{}",
            COOKIE_NAME,
            session_id,
            cookie_max_age.as_secs(),
            if secure { "; Secure" } else { "" }
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            header::SET_COOKIE,
            header::HeaderValue::from_str(&cookie_val).unwrap(),
        );
        (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response()
    } else {
        let attempt = record_attempt(&ip_str);
        let remaining = (state.config.server.max_attempts as i64 - attempt.count as i64).max(0);

        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid PIN",
                "attemptsLeft": remaining
            })),
        )
            .into_response()
    }
}

pub async fn logout(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let cookie_val = headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c_str| {
            c_str
                .split(';')
                .find(|s| s.trim().starts_with(&format!("{}=", COOKIE_NAME)))
                .and_then(|s| s.split('=').nth(1))
                .map(|s| s.trim().to_string())
        });

    if let Some(session_id) = cookie_val {
        state.active_sessions.write().await.remove(&session_id);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        header::HeaderValue::from_static("GRID_PIN=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0"),
    );
    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({ "success": true })),
    )
        .into_response()
}

pub async fn auth_check(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    if !is_authenticated(&headers, &state).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    StatusCode::OK.into_response()
}

pub async fn pin_required(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ip = get_client_ip(
        headers.headers(),
        addr,
        state.config.server.trust_proxy,
        &state.config.server.trusted_proxies,
    );
    let lockout_dur = Duration::from_secs(state.config.server.lockout_time_minutes * 60);
    Json(serde_json::json!({
        "required": state.config.server.pin.is_some(),
        "length": state.config.server.pin.as_ref().map(|p| p.len()).unwrap_or(0),
        "locked": is_locked_out(&ip, state.config.server.max_attempts, lockout_dur),
        "enable_translation": state.config.server.enable_translation,
        "enable_themes": state.config.server.enable_themes,
        "enable_print": state.config.server.enable_print,
        "show_version": state.config.server.show_version,
        "show_github": state.config.server.show_github,
    }))
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0);

    let ip = get_client_ip(
        req.headers(),
        addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0))),
        state.config.server.trust_proxy,
        &state.config.server.trusted_proxies,
    );

    if !state.check_rate_limit(&ip).await {
        let body = serde_json::json!({
            "error": "Too many requests. Please slow down."
        });
        let mut response = axum::response::Json(body).into_response();
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        return Ok(response);
    }

    Ok(next.run(req).await)
}
