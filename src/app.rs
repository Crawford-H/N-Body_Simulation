use coffee::{load::Task, Game, Timer};
use coffee::graphics::{Window, Color, Frame, Batch, Image, Sprite, Rectangle, Vector, Transformation, Point};
use coffee::input::{KeyboardAndMouse, mouse, keyboard};
use rayon::prelude::*;

use crate::particle::Particle;
use crate::config::Config;

pub struct Application {
    entities: Vec<Particle>, // vector to store locations of particles

    // variables for rendering particles
    particle_sprite_quad: Rectangle<u16>,
    batch: Batch,
    camera_transform: Transformation,
    zoom: f32,
    camera_position: Point,

    // config to store various constants
    config: Config, 
}

impl Application {
    fn update_acceleration_par(&mut self) {
        let entities_clone = self.entities.clone();
        self.entities.par_iter_mut().for_each(move |particle| {
            entities_clone.iter().for_each(|other_particle| {
                particle.calculate_acceleration(other_particle)
            });
        })
    }

    fn _update_acceleration_series(&mut self) {
        let entities_clone = self.entities.clone();
        self.entities.iter_mut().for_each(move |particle| {
            entities_clone.iter().for_each(|other_particle| {
                particle.calculate_acceleration(other_particle)
            });
        })
    }
}

impl Game for Application {
    type Input = KeyboardAndMouse; // No input data
    type LoadingScreen = ();

    const TICKS_PER_SECOND: u16 = 30;


    fn load(_window: &Window) -> Task<Application> {
        let config = Config::new();
        Task::stage("Loading Sprites", Image::load(config.star_sprite_path.as_str())).map(|sprites| 
            Application {
                entities: Vec::new(),
                batch: Batch::new(sprites),
                particle_sprite_quad: Rectangle { x: 0, y: 0, height: config.sprite_height, width: config.sprite_width },
                config,
                zoom: 1.,
                camera_position: Point::new(0.0, 0.0),
                camera_transform: Transformation::identity(),
        })
    }

    fn update(&mut self, _window: &Window) {
        Self::update_acceleration_par(self);
    }

    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        // Clear the current frame
        frame.clear(Color::BLACK);
        let mut target = frame.as_target();
        let mut camera = target.transform(self.camera_transform);

        self.entities.par_iter_mut().for_each(|particle| { 
            particle.velocity += particle.acceleration * timer.next_tick_proximity();
            particle.position += particle.velocity * timer.next_tick_proximity();
        });

        let sprite_offset = Vector::new(self.config.sprite_width as f32 * self.config.sprite_scale / 2., self.config.sprite_height as f32 * self.config.sprite_scale / 2.);
        let sprites = self.entities.iter().map(|particle| {
            Sprite {
                source: self.particle_sprite_quad,
                position: particle.position - sprite_offset,
                scale: (self.config.sprite_scale, self.config.sprite_scale),
            }
        });
    
        self.batch.clear();
        self.batch.extend(sprites);
        self.batch.draw(&mut camera);
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        if input.mouse().is_button_pressed(mouse::Button::Left) {
            let cursor_position = input.mouse().cursor_position();
            self.entities.push(Particle::new(cursor_position.x - self.camera_position.x, cursor_position.y - self.camera_position.y, 1000.));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key1) {
            let cursor_position = input.mouse().cursor_position();
            self.entities.push(Particle::new(cursor_position.x - self.camera_position.x, cursor_position.y - self.camera_position.y, 1.0e6));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key2) {
            let cursor_position = input.mouse().cursor_position();
            self.entities.push(Particle::new(cursor_position.x - self.camera_position.x, cursor_position.y - self.camera_position.y, 1.0e8));
        }

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
        if input.keyboard().was_key_released(keyboard::KeyCode::R) {
            self.entities = Vec::new();
        }
        if input.keyboard().was_key_released(keyboard::KeyCode::P) {
            println!("Number of particles = {}", self.entities.len());
        }

        // let scroll_wheel = input.mouse().wheel_movement().vertical;
        // let _cursor_position = input.mouse().cursor_position();
        // if scroll_wheel != 0. {
        //     self.zoom += scroll_wheel * 0.05;
        // }
        self.camera_transform = Transformation::scale(self.zoom) * Transformation::translate(
            Vector::new(self.camera_position.x, self.camera_position.y));
    }
}
