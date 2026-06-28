use crate::app::App;
use crate::i18n::get_translations;
use crate::types::Msg;
use web_sys::HtmlInputElement;
use yew::prelude::*;

impl App {
    pub fn render_pin_entry(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        let on_submit = {
            let link = ctx.link().clone();
            Callback::from(move |e: SubmitEvent| {
                e.prevent_default();
                link.send_message(Msg::VerifyPin);
            })
        };
        let on_input = {
            let link = ctx.link().clone();
            let pin_length = self.pin_length;
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                let val = input.value();
                let filtered: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                input.set_value(&filtered);

                link.send_message(Msg::PinInputChanged(filtered.clone()));
                if filtered.len() == pin_length {
                    link.send_message(Msg::VerifyPin);
                }
            })
        };

        html! {
            <div class="login-container">
                <div class="login-box">
                    <div class="pin-header">
                        <h2>{tr.enter_pin}</h2>
                    </div>
                    <form id="pin-form" onsubmit={on_submit}>
                        <div class="pin-wrapper">
                            <input
                                id="pin-input"
                                type="password"
                                class="pin-input-field"
                                value={self.pin_input.clone()}
                                oninput={on_input}
                                autocomplete="current-password"
                                autofocus=true
                                maxlength={self.pin_length.to_string()}
                                placeholder={"•".repeat(self.pin_length)}
                            />
                        </div>
                    </form>
                    <div class="pin-status">
                        if let Some(ref err) = self.error_message {
                            <p id="pin-error" class="pin-error" style="display: block;">{err}</p>
                        }
                    </div>
                </div>
            </div>
        }
    }
}
