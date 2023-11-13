use coffee::graphics::{Batch, Color, Frame, Image, Point, Sprite, Transformation, Vector, Window};
use coffee::input::{keyboard, mouse, KeyboardAndMouse};
use coffee::load::Task;
use coffee::ui::{UserInterface, Renderer, Element, Row, Justify, Align, Column, Text};
use coffee::{Game, Timer};
use glam::DVec2;
use rayon::prelude::*;

use crate::world::{World, ThreadsWorld, RayonWorld, SequentialWorld};
use crate::config::Config;

#[derive(Debug)]
enum WorldType {
    Threads,
    Rayon,
    Sequential,
}

pub struct Application {
    /// Environment variables
    config: Config,
    /// Stores the world data and handles updating of particles
    world: Box<dyn World>,
    /// The state of which world implementation is currently being used
    world_type: WorldType,
    /// Position of the camera for render particles
    camera_position: Point,
    /// Container for sprites of particles to render
    batch: Batch,
}

impl Application {
    fn change_world_algorithm(&mut self, new_algorithm: WorldType) {
        println!("Changed algorithm to {:?}", new_algorithm);
        self.world_type = new_algorithm;
        let particles = self.world.get_particles();
        self.world = match self.world_type {
            WorldType::Threads => Box::new(ThreadsWorld::new(self.config.num_threads, particles)),
            WorldType::Rayon => Box::new(RayonWorld { particles }),
            WorldType::Sequential => Box::new(SequentialWorld { particles }),
        };
    }
}

impl Game for Application {
    type Input = KeyboardAndMouse; // No input data
    type LoadingScreen = (); // No loading screen
    const TICKS_PER_SECOND: u16 = 60;

    fn load(_window: &Window) -> Task<Application> {
        let config = Config::new();

        Task::stage("Loading sprites...", Image::load(config.sprite_file.as_str())).map(|sprite| 
            Application {
                world: Box::new(ThreadsWorld::new(config.num_threads, Vec::new())),
                world_type: WorldType::Threads,
                camera_position: Point::new((config.screen_width / 2) as f32, (config.screen_height / 2) as f32),
                batch: Batch::new(sprite),
                config
        })
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        // Clear the current frame
        frame.clear(Color::BLACK);

        // update camera position
        let mut target = frame.as_target();
        let camera_transform = Transformation::translate(Vector::new(self.camera_position.x, self.camera_position.y));
        let mut camera = target.transform(camera_transform);

        // generate particles to draw
        let particles = self.world.get_particles();
        let sprites = particles.par_iter().map(|particle| Sprite {
            source: self.config.sprite_source,
            position: Point::new(particle.position.x as f32, particle.position.y as f32) * self.config.world_scale - Vector::new(self.config.horizontal_offset, self.config.vertical_offset),
            scale: (self.config.sprite_scale, self.config.sprite_scale),
        });

        // render screen
        self.batch.clear();
        self.batch.par_extend(sprites);
        self.batch.draw(&mut camera);
    }

    fn update(&mut self, _window: &Window) {
        self.world.update(self.config.time_scale);
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        // calculate world position from screen positions
        let cursor_position = input.mouse().cursor_position();
        let x_position = ((cursor_position.x - self.camera_position.x) / self.config.world_scale) as f64;
        let y_position = ((cursor_position.y - self.camera_position.y) / self.config.world_scale) as f64;

        // change world algorithm
        if input.keyboard().was_key_released(keyboard::KeyCode::Tab) {
            match self.world_type {
                WorldType::Threads => self.change_world_algorithm(WorldType::Rayon),
                WorldType::Rayon => self.change_world_algorithm(WorldType::Sequential),
                WorldType::Sequential => self.change_world_algorithm(WorldType::Threads),
            }
        }

        // create particles
        if input.mouse().is_button_pressed(mouse::Button::Left) {
            self.world.create_particle(
                DVec2::new(x_position, y_position),
                DVec2::ZERO,
                1.0e2,
            )
        }
        if input.keyboard().was_key_released(keyboard::KeyCode::Key1) {
            self.world.create_particle(
                DVec2::new(x_position, y_position),
                DVec2::ZERO,
                1.0e12,
            )
        }

        // move camera
        if input.keyboard().is_key_pressed(keyboard::KeyCode::W) {
            self.camera_position.y += 5.;
        }
        if input.keyboard().is_key_pressed(keyboard::KeyCode::S) {
            self.camera_position.y -= 5.;
        }
        if input.keyboard().is_key_pressed(keyboard::KeyCode::A) {
            self.camera_position.x += 5.;
        }
        if input.keyboard().is_key_pressed(keyboard::KeyCode::D) {
            self.camera_position.x -= 5.;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    
}

impl UserInterface for Application {
    type Message = Message;

    type Renderer = Renderer;

    fn react(&mut self, message: Self::Message, _window: &mut Window) {
        match message {
        }
    }

    fn layout(&mut self, window: &Window,) -> Element<Message> {
        Row::new()
            .padding(20)
            .spacing(20)
            .width(window.width() as u32)
            .height(window.height() as u32)
            .justify_content(Justify::Center)
            .align_items(Align::End)
            .push(Column::new()
                .padding(10)
                .push(Text::new(&format!("Scale: {} meter(s) / pixel", 1. / self.config.world_scale)))
                .push(Text::new(&format!("Number of particles: {}", self.world.get_particles().len())))
                .push(Text::new(&format!("Time Scale: {:.5} seconds / 1 real second", self.config.time_scale * Self::TICKS_PER_SECOND as f64))))
            .push(Column::new())
            .push(Column::new())
        .into()
    }
}
