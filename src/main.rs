mod application;
mod particle;
mod world;
mod config;

use coffee::{graphics::WindowSettings, Game};

use crate::application::Application;

fn main() -> Result<(), coffee::Error> {
    Application::run(WindowSettings {
        title: String::from("Particle Physics Simulator"),
        size: (1920, 1080),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
