use idp_gui::App;

pub fn main() -> iced::Result {
    env_logger::init();

    use iced::Sandbox;
    App::run(iced::Settings {
        default_text_size: 16.0,
        ..Default::default()
    })
}
