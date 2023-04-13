use app::Application;
use coffee::{Result, graphics::WindowSettings, ui::UserInterface};

mod app;
mod config;
mod particle;


fn main() -> Result<()> {
    <Application as UserInterface>::run(WindowSettings {
        title: String::from("Particle Physics Simulator"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
