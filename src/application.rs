use std::time::Instant;

use coffee::graphics::{
    Batch, Color, Frame, Image, Point, Rectangle, Sprite, Transformation, Vector, Window,
};
use coffee::input::{keyboard, mouse, KeyboardAndMouse};
use coffee::load::Task;
use coffee::{Game, Timer};
use glam::DVec2;
use rayon::prelude::*;

use crate::world::WorldWorkerThreads;

// sprite constants
const SPRITE_FILE: &str = "resources/star.png";
const SPRITE_WIDTH: f32 = 512.;
const SPRITE_HEIGHT: f32 = 512.;
const SPRITE_SCALE: f32 = 0.025;
const SPRITE_SOURCE: Rectangle<u16> = Rectangle {
    x: 0,
    y: 0,
    height: SPRITE_HEIGHT as u16,
    width: SPRITE_WIDTH as u16,
};
const HORIZONTAL_OFFSET: f32 = SPRITE_WIDTH * SPRITE_SCALE / 2.;
const VERTICAL_OFFSET: f32 = SPRITE_HEIGHT * SPRITE_SCALE / 2.;
// constants for rendering and processing
const NUM_THREADS: usize = 20;
const HEIGHT: u32 = 1080;
const WIDTH: u32 = 1920;
// defaults for creating world
const DEFAULT_TIME_SCALE: f64 = 50.;
const DEFAULT_WORLD_SCALE: f32 = 1.;

pub struct Application {
    // member variables for data on world
    world: WorldWorkerThreads,
    time_since_last_frame: Instant,

    // member variables for rendering
    camera_position: Point,
    world_scale: f32,
    time_scale: f64,
    batch: Batch,
}

impl Game for Application {
    type Input = KeyboardAndMouse; // No input data
    type LoadingScreen = (); // No loading screen

    fn load(_window: &Window) -> Task<Application> {
        Task::stage("Loading sprites...", Image::load(SPRITE_FILE)).map(|sprite| Application {
            world: WorldWorkerThreads::new(NUM_THREADS),
            time_scale: DEFAULT_TIME_SCALE,
            world_scale: DEFAULT_WORLD_SCALE,
            camera_position: Point::new((WIDTH / 2) as f32, (HEIGHT / 2) as f32),
            batch: Batch::new(sprite),
            time_since_last_frame: Instant::now(),
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
        let particles_lock = self.world.particles.read();
        let sprites = particles_lock.par_iter().map(|particle| Sprite {
            source: SPRITE_SOURCE,
            position: Point::new(particle.position.x as f32, particle.position.y as f32) * self.world_scale - Vector::new(HORIZONTAL_OFFSET, VERTICAL_OFFSET),
            scale: (SPRITE_SCALE, SPRITE_SCALE),
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
