mod app;
mod components;
mod i18n;
mod types;
mod board_normalizer;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
