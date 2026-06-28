use crate::app::App;
use crate::i18n::get_translations;
use crate::types::Msg;
use web_sys::HtmlInputElement;
use yew::prelude::*;

impl App {
    pub fn render_task_modal(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        let on_input = {
            let link = ctx.link().clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                link.send_message(Msg::TaskModalInputChanged(input.value()));
            })
        };

        html! {
            <div class="modal task-modal-overlay">
                <div class="modal-content card">
                    <h2>{tr.edit_task}</h2>
                    <div class="modal-body">
                        <label>{tr.task_text}</label>
                        <input
                            type="text"
                            placeholder={tr.task_placeholder}
                            value={self.task_modal_text.clone()}
                            oninput={on_input}
                            autofocus=true
                        />
                    </div>
                    <div class="modal-actions">
                        <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveTask)}>
                            {tr.save}
                        </button>
                        if self.task_modal_index.is_some() {
                            <button class="btn btn-danger" onclick={ctx.link().callback(|_| Msg::DeleteTask)}>
                                {tr.delete}
                            </button>
                        }
                        <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::CloseTaskModal)}>
                            {tr.cancel}
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
