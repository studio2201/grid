use crate::app::App;
use crate::app::Msg;
use crate::types::{BoardData, Toast};
use gloo_net::http::Request;
use shared_frontend::i18n::strings::{StringKey, lookup};
use yew::prelude::*;

impl App {
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
            link.send_message(Msg::ShowToast(
                lookup(StringKey::StatusSaving, lang).to_string(),
                false,
            ));
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(req) = Request::post("/api/tasks").json(&payload) {
                    match req.send().await {
                        Ok(resp) if resp.status() == 200 => {
                            link.send_message(Msg::ShowToast(
                                lookup(StringKey::StatusSaved, lang).to_string(),
                                false,
                            ));
                        }
                        Ok(resp) if resp.status() == 409 => {
                            link.send_message(Msg::ShowToast(
                                lookup(StringKey::StatusConflictError, lang).to_string(),
                                true,
                            ));
                        }
                        _ => {
                            link.send_message(Msg::ShowToast(
                                lookup(StringKey::StatusSaveError, lang).to_string(),
                                true,
                            ));
                        }
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
