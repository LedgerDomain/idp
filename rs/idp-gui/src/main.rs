use idp_gui::App;

#[tokio::main]
async fn main() -> iced::Result {
    env_logger::init();

    println!(
        "If this crashes in video initialization, you may need to set the env var WGPU_BACKEND=gl"
    );

    use iced::Application;
    App::run(iced::Settings {
        default_text_size: 16.0,
        ..Default::default()
    })
}
