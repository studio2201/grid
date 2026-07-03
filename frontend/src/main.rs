mod app;
mod components;
mod i18n;
mod types;
mod utils;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
