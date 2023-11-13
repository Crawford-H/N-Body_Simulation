mod application;
mod particle;
mod world;
mod config;

use coffee::{graphics::WindowSettings, ui::UserInterface};

use crate::application::Application;

fn main() -> Result<(), coffee::Error> {
    <Application as UserInterface>::run(WindowSettings {
        title: String::from("Particle Physics Simulator"),
        size: (1920, 1080),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
