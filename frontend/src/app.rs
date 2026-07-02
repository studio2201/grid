pub mod update;
pub mod update_handlers;
pub mod view;

use crate::types::*;
use yew::prelude::*;

pub struct App {
    pub site_title: String,
    pub theme: String,
    pub language: Language,
    pub is_authenticated: bool,
    pub pin_required: bool,
    pub pin_length: usize,
    pub pin_input: String,
    pub error_message: Option<String>,
    pub board_data: Option<BoardData>,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
    pub show_version: bool,
    pub show_github: bool,

    // UI states
    pub active_board_id: String,

    pub task_modal_column_id: Option<String>,
    pub task_modal_index: Option<usize>,
    pub task_modal_text: String,
    pub show_task_modal: bool,

    // Drag data
    pub dragged_column_id: Option<String>,
    pub dragged_task_index: Option<usize>,

    // Toast
    pub toasts: Vec<Toast>,
    pub next_toast_id: usize,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::create_app(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.update_app(ctx, msg)
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.view_app(ctx)
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            use wasm_bindgen::JsCast;
            let window = web_sys::window().unwrap();
            
            let link_online = ctx.link().clone();
            let on_online = wasm_bindgen::prelude::Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                link_online.send_message(Msg::OnlineStatusChanged(true));
            });
            window
                .add_event_listener_with_callback("online", on_online.as_ref().unchecked_ref())
                .unwrap();
            on_online.forget();

            let link_offline = ctx.link().clone();
            let on_offline = wasm_bindgen::prelude::Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                link_offline.send_message(Msg::OnlineStatusChanged(false));
            });
            window
                .add_event_listener_with_callback("offline", on_offline.as_ref().unchecked_ref())
                .unwrap();
            on_offline.forget();
        }
    }
}
