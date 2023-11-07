use std::time::Instant;

use coffee::graphics::{Batch, Color, Frame, Image, Point, Sprite, Transformation, Vector, Window};
use coffee::input::{keyboard, mouse, KeyboardAndMouse};
use coffee::load::Task;
use coffee::{Game, Timer};
use glam::DVec2;
use rayon::prelude::*;

use crate::rayon_world::RayonWorld;
use crate::sequential_world::SequentialWorld;
use crate::worker_threads::WorldWorkerThreads;
use crate::world::World;
use crate::config::Config;

#[derive(Debug)]
enum WorldType {
    WorkerThreads,
    Rayon,
    Sequential,
}

pub struct Application {
    config: Config,

    // member variables for data on world
    world: Box<dyn World>,
    world_type: WorldType,
    time_since_last_frame: Instant,

    // member variables for rendering
    camera_position: Point,
    world_scale: f32,
    time_scale: f64,
    batch: Batch,
}

impl Application {
    fn change_world_algorithm(&mut self, new_algorithm: WorldType) {
        println!("Changed algorithm to {:?}", new_algorithm);
        self.world_type = new_algorithm;
        let particles = self.world.get_particles();
        self.world = match self.world_type {
            WorldType::WorkerThreads => Box::new(WorldWorkerThreads::new(self.config.num_threads, particles)),
            WorldType::Rayon => Box::new(RayonWorld { particles }),
            WorldType::Sequential => Box::new(SequentialWorld { particles }),
        };
    }
}

impl Game for Application {
    type Input = KeyboardAndMouse; // No input data
    type LoadingScreen = (); // No loading screen

    fn load(_window: &Window) -> Task<Application> {
        let config = Config::new();

        Task::stage("Loading sprites...", Image::load(config.sprite_file.as_str())).map(|sprite| 
            Application {
                world: Box::new(WorldWorkerThreads::new(config.num_threads, Vec::new())),
                world_type: WorldType::WorkerThreads,
                time_scale: config.default_time_scale,
                world_scale: config.default_world_scale,
                camera_position: Point::new((config.screen_width / 2) as f32, (config.screen_height / 2) as f32),
                batch: Batch::new(sprite),
                time_since_last_frame: Instant::now(),
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

        // update particles in world
        let dt = self.time_since_last_frame.elapsed().as_secs_f64() * self.time_scale;
        self.time_since_last_frame = Instant::now();
        self.world.update(dt);

        // generate particles to draw
        let particles = self.world.get_particles();
        let sprites = particles.par_iter().map(|particle| Sprite {
            source: self.config.sprite_source,
            position: Point::new(particle.position.x as f32, particle.position.y as f32) * self.world_scale - Vector::new(self.config.horizontal_offset, self.config.vertical_offset),
            scale: (self.config.sprite_scale, self.config.sprite_scale),
        });

        // render screen
        self.batch.clear();
        self.batch.par_extend(sprites);
        self.batch.draw(&mut camera);
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        // calculate world position from screen positions
        let cursor_position = input.mouse().cursor_position();
        let x_position = ((cursor_position.x - self.camera_position.x) / self.world_scale) as f64;
        let y_position = ((cursor_position.y - self.camera_position.y) / self.world_scale) as f64;

        // change world algorithm
        if input.keyboard().was_key_released(keyboard::KeyCode::Tab) {
            match self.world_type {
                WorldType::WorkerThreads => self.change_world_algorithm(WorldType::Rayon),
                WorldType::Rayon => self.change_world_algorithm(WorldType::Sequential),
                WorldType::Sequential => self.change_world_algorithm(WorldType::WorkerThreads),
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
