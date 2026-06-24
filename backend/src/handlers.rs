use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use chrono::Utc;
use std::fs;
use std::path::Path as StdPath;
use std::sync::LazyLock;
use std::time::Instant;
use crate::state::AppState;

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

pub fn initialize_storage() {
    if !StdPath::new("data").exists() {
        let _ = fs::create_dir_all("data");
    }
    let tasks_path = StdPath::new("data/tasks.json");
    if !tasks_path.exists() {
        let default_structure = serde_json::json!({
            "boards": {
                "work": {
                    "name": "Work",
                    "columns": {
                        "todo": { "name": "To Do", "tasks": [] },
                        "doing": { "name": "Doing", "tasks": [] },
                        "done": { "name": "Done", "tasks": [] }
                    }
                }
            },
            "activeBoard": "work"
        });
        let _ = fs::write(
            tasks_path,
            serde_json::to_string_pretty(&default_structure).unwrap(),
        );
    }
}

pub async fn serve_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339(),
        "uptime": START_TIME.elapsed().as_secs()
    }))
}

pub async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let path = StdPath::new("frontend/dist/index.html");
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            let rendered = content.replace("{{SITE_TITLE}}", &state.config.site_title);
            Html(rendered).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_tasks() -> impl IntoResponse {
    match tokio::fs::read_to_string("data/tasks.json").await {
        Ok(data) => (StatusCode::OK, data).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn save_tasks(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    match tokio::fs::write(
        "data/tasks.json",
        serde_json::to_string_pretty(&payload).unwrap(),
    )
    .await
    {
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
