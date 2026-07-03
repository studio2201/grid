use crate::app::App;
use crate::i18n::get_translations;
use crate::types::*;
use gloo_net::http::Request;
use shared_frontend::storage::StorageService;
use shared_frontend::theme::{Theme, mapping::Scheme};
use shared_frontend::i18n::strings::{lookup, StringKey};
use yew::prelude::*;

impl App {
    pub fn create_app(ctx: &Context<Self>) -> Self {
        let raw_theme = StorageService.get_item("theme");
        let theme = if let Some(scheme) = Scheme::from_id(&raw_theme) {
            scheme.to_theme().name().to_string()
        } else if raw_theme.is_empty() {
            Theme::default().name().to_string()
        } else {
            Theme::from_name(&raw_theme)
                .unwrap_or_default()
                .name()
                .to_string()
        };
        if theme != raw_theme {
            StorageService.set_item("theme", &theme);
        }

        let stored_lang = StorageService.get_item("language");
        let language = Language::from_code(&if stored_lang.is_empty() { "en".to_string() } else { stored_lang });

        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(el) = document.document_element()
        {
            let _ = el.set_attribute("class", &theme);
            let _ = el.set_attribute("data-theme", &theme);
        }

        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get("/api/pin-required").send().await {
                Ok(resp) if resp.status() == 200 => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        link.send_message(Msg::FetchConfigSuccess(json));
                    }
                }
                _ => {}
            }
        });

        let app = Self {
            site_title: "Grid".to_string(),
            theme,
            language,
            is_authenticated: false,
            pin_required: false,
            pin_length: 0,
            pin_input: String::new(),
            error_message: None,
            board_data: None,
            active_board_id: "work".to_string(),
            task_modal_column_id: None,
            task_modal_index: None,
            task_modal_text: String::new(),
            show_task_modal: false,
            dragged_column_id: None,
            dragged_task_index: None,
            toasts: Vec::new(),
            next_toast_id: 0,
            enable_translation: false,
            enable_themes: true,
            enable_print: false,
            show_version: true,
            show_github: true,
        };
        app.update_document_title();
        app
    }

    pub fn update_app(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::FetchConfigSuccess(json) => self.handle_fetch_config_success(ctx, json),
            Msg::FetchTasksSuccess(data) => self.handle_fetch_tasks_success(ctx, data),
            Msg::FetchTasksError => {
                let tr = crate::i18n::get_translations(self.language);
                self.show_toast(tr.toast_failed_load_tasks.to_string(), true, ctx);
                true
            }
            Msg::VerifyPinSuccess => {
                self.is_authenticated = true;
                self.error_message = None;
                self.load_tasks(ctx);
                self.show_toast(lookup(StringKey::StatusPinSuccess, self.language).to_string(), false, ctx);
                true
            }
            Msg::VerifyPinFailure(err) => {
                self.is_authenticated = false;
                if !err.is_empty() {
                    let tr = get_translations(self.language);
                    self.error_message = Some(if err == "Invalid PIN" {
                        tr.invalid_pin.to_string()
                    } else {
                        err
                    });
                }
                self.show_toast(lookup(StringKey::StatusPinFailure, self.language).to_string(), true, ctx);
                true
            }
            Msg::PinInputChanged(val) => {
                self.pin_input = val;
                self.error_message = None;
                true
            }
            Msg::VerifyPin => self.handle_verify_pin(ctx),
            Msg::Logout => {
                let res = self.handle_logout(ctx);
                self.show_toast(lookup(StringKey::StatusLogout, self.language).to_string(), false, ctx);
                res
            }
            Msg::PrintBoard => {
                if let Some(window) = web_sys::window() {
                    let print_res = window.print();
                    if print_res.is_ok() {
                        self.show_toast(lookup(StringKey::StatusPrintSuccess, self.language).to_string(), false, ctx);
                    } else {
                        self.show_toast(lookup(StringKey::StatusPrintFailure, self.language).to_string(), true, ctx);
                    }
                }
                false
            }
            Msg::SwitchLanguage(lang) => {
                self.language = lang;
                StorageService.set_item("language", lang.code());
                true
            }
            Msg::ToggleTheme => {
                let res = self.handle_toggle_theme();
                self.show_toast(lookup(StringKey::StatusThemeChanged, self.language).to_string(), false, ctx);
                res
            }
            Msg::OpenAddTaskModal(col_id) => {
                self.task_modal_column_id = Some(col_id);
                self.task_modal_index = None;
                self.task_modal_text.clear();
                self.show_task_modal = true;
                true
            }
            Msg::OpenEditTaskModal(col_id, idx) => {
                if let Some(ref data) = self.board_data
                    && let Some(board) = data.boards.get(&self.active_board_id)
                    && let Some(col) = board.columns.get(&col_id)
                    && let Some(task) = col.tasks.get(idx)
                {
                    self.task_modal_column_id = Some(col_id);
                    self.task_modal_index = Some(idx);
                    self.task_modal_text = task.clone();
                    self.show_task_modal = true;
                }
                true
            }
            Msg::TaskModalInputChanged(val) => {
                self.task_modal_text = val;
                true
            }
            Msg::SaveTask => self.handle_save_task(ctx),
            Msg::DeleteTask => self.handle_delete_task(ctx),
            Msg::DeleteTaskDirect(col_id, idx) => self.handle_delete_task_direct(ctx, col_id, idx),
            Msg::CloseTaskModal => {
                self.show_task_modal = false;
                true
            }
            Msg::DragStart(col_id, idx, e) => self.handle_drag_start(col_id, idx, e),
            Msg::DragOver(e) => {
                e.prevent_default();
                false
            }
            Msg::Drop(dest_col_id, dest_idx, e) => self.handle_drop(ctx, dest_col_id, dest_idx, e),
            Msg::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
                true
            }
            Msg::ShowToast(message, is_error) => {
                self.show_toast(message, is_error, ctx);
                true
            }
            Msg::OnlineStatusChanged(online) => {
                let (msg_key, is_error) = if online {
                    (StringKey::StatusOnline, false)
                } else {
                    (StringKey::StatusOffline, true)
                };
                self.show_toast(lookup(msg_key, self.language).to_string(), is_error, ctx);
                true
            }
        }
    }

    pub fn update_document_title(&self) {
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
        {
            document.set_title(&self.site_title);
        }
    }

    pub fn load_tasks(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            // The backend exposes the kanban state at /api/tasks (auth +
            // rate-limit + origin-validation middleware all applied).
            // Previously this was /data/tasks.json, a duplicate route with
            // weaker middleware; that route was removed in this PR.
            match Request::get("/api/tasks").send().await {
                Ok(resp) if resp.status() == 200 => {
                    if let Ok(data) = resp.json::<BoardData>().await {
                        link.send_message(Msg::FetchTasksSuccess(data));
                    } else {
                        link.send_message(Msg::FetchTasksError);
                    }
                }
                _ => {
                    link.send_message(Msg::FetchTasksError);
                }
            }
        });
    }

    pub fn save_tasks_backend(&self, ctx: &Context<Self>) {
        if let Some(ref data) = self.board_data {
            let payload = data.clone();
            let link = ctx.link().clone();
            let lang = self.language;
            link.send_message(Msg::ShowToast(lookup(StringKey::StatusSaving, lang).to_string(), false));
            wasm_bindgen_futures::spawn_local(async move {
                match Request::post("/api/tasks")
                    .json(&payload)
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.status() == 200 => {
                        link.send_message(Msg::ShowToast(lookup(StringKey::StatusSaved, lang).to_string(), false));
                    }
                    Ok(resp) if resp.status() == 409 => {
                        link.send_message(Msg::ShowToast(lookup(StringKey::StatusConflictError, lang).to_string(), true));
                    }
                    _ => {
                        link.send_message(Msg::ShowToast(lookup(StringKey::StatusSaveError, lang).to_string(), true));
                    }
                }
            });
        }
    }

    pub fn show_toast(&mut self, message: String, is_error: bool, ctx: &Context<Self>) {
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        self.toasts.push(Toast {
            id,
            message,
            is_error,
        });
        let link = ctx.link().clone();
        gloo_timers::callback::Timeout::new(3000, move || {
            link.send_message(Msg::DismissToast(id));
        })
        .forget();
    }
}
