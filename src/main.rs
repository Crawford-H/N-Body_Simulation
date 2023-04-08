use coffee::{Game, Result, graphics::WindowSettings};

mod app;
mod config;
mod particle;


fn main() -> Result<()> {
    app::Application::run(WindowSettings {
        title: String::from("Particle Physics Simulator"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
