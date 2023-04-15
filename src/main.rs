/// Author: Crawford Heidinger
/// Student number: 100753120

use app::Application;
use coffee::{Result, graphics::WindowSettings, ui::UserInterface};

mod app;
mod config;
mod particle;
mod benchmark;


/// Entry point to the program.
fn main() -> Result<()> {
    <Application as UserInterface>::run(WindowSettings {
        title: String::from("Particle Physics Simulator"),
        size: (1920, 1080),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}
