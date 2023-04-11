use coffee::{load::Task, Game, Timer};
use coffee::graphics::{Window, Color, Frame, Batch, Image, Sprite, Rectangle, Vector, Transformation, Point};
use coffee::input::{KeyboardAndMouse, mouse, keyboard};
use rayon::prelude::*;

use rand::Rng;

use crate::particle::{Particle, acceleration};
use crate::config::Config;

pub struct Application {
    entities: Vec<Particle>, // vector to store locations of particles

    // variables for rendering particles
    particle_sprite_quad: Rectangle<u16>,
    batch: Batch,
    camera_transform: Transformation,
    zoom: f32,
    camera_position: Point,
    scale: f32,
    time_scale: f32,

    // config to store various constants
    config: Config, 
}

impl Application {
    fn update_acceleration_par(&mut self) {
        let entities_clone = self.entities.clone();

        self.entities.par_iter_mut().for_each( move |particle| {
            particle.acceleration = entities_clone
                .iter()
                .filter(|other| particle.id != other.id )
                .fold(Vector::new(0., 0.), |acc, other|  acc + acceleration(particle, other) )
        });
    }

    fn _update_acceleration_series(&mut self) {
        let entities_clone = self.entities.clone();
        
        for particle in self.entities.iter_mut() {
            particle.acceleration = entities_clone
                .iter()
                .filter(|other| particle.id != other.id )
                .fold(Vector::new(0., 0.), |acc, other| acc + acceleration(particle, other) )
        };
    }

    fn generate_random_particles(&mut self, num_particles: i32) {
        let generator = &mut rand::thread_rng();
        for _ in 0..num_particles {
            self.entities.push(Particle { 
                velocity: Vector::new(generator.gen_range(-1.0e-1..1.0e-1), generator.gen_range(-1.0e-1..1.0e-1)), 
                position: Point::new(generator.gen_range(-1.0e3..1.0e3), generator.gen_range(-1.0e3..1.0e3)), 
                mass: generator.gen_range(10.0e1..1.0e8), 
                acceleration: Vector::new(0., 0.),
                id: self.entities.len() as u16, 
            })
        }
    }

    fn generate_solar_system(&mut self) {
        self.scale = 1500. / 4495.060e9;
        self.time_scale = 5000.;
        // generate the sun
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 0.), 
            position: Point::new(0., 0.), 
            mass: 1.989e30, 
            acceleration: Vector::new(0., 0.) 
        });

        // mercury
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 47.36e3), 
            position: Point::new(57.909e9, 0.), 
            mass: 0.33011e24, 
            acceleration: Vector::new(0., 0.) 
        });

        // venus
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 35.02e3), 
            position: Point::new(108.209e9, 0.), 
            mass: 4.8675e24, 
            acceleration: Vector::new(0., 0.) 
        });

        // earth
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 29.78e3), 
            position: Point::new(149.596e9, 0.), 
            mass: 5.9724e24, 
            acceleration: Vector::new(0., 0.) 
        });

        // mars
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 24.07e3), 
            position: Point::new(227.923e9, 0.), 
            mass: 0.64171e24, 
            acceleration: Vector::new(0., 0.) 
        });

        // jupiter
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 13e3), 
            position: Point::new(778.57e9, 0.), 
            mass: 1898.19e24, 
            acceleration: Vector::new(0., 0.) 
        });

        // saturn
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 9.68e3), 
            position: Point::new(1433.529e9, 0.), 
            mass: 568.34e24, 
            acceleration: Vector::new(0., 0.) 
        });

        // uranus
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 6.80e3), 
            position: Point::new(2872.463e9, 0.), 
            mass: 86.813e24, 
            acceleration: Vector::new(0., 0.) 
        });

        // neptune
        self.entities.push(Particle { 
            id: self.entities.len() as u16, 
            velocity: Vector::new(0., 5.43e3), 
            position: Point::new(4495.060e9, 0.), 
            mass: 102.413e24, 
            acceleration: Vector::new(0., 0.) 
        });
    }
}

impl Game for Application {
    type Input = KeyboardAndMouse;
    type LoadingScreen = ();
    const TICKS_PER_SECOND: u16 = 64;
    const DEBUG_KEY: Option<keyboard::KeyCode> = Some(keyboard::KeyCode::F12);

    fn load(_window: &Window) -> Task<Application> {
        let config = Config::new();
        Task::stage("Loading Sprites", Image::load(config.star_sprite_path.as_str())).map(|sprites| 
            Application {
                entities: Vec::new(),
                batch: Batch::new(sprites),
                particle_sprite_quad: Rectangle { x: 0, y: 0, height: config.sprite_height, width: config.sprite_width },
                camera_position: Point::new(config.screen_width as f32 / 2., config.screen_height as f32 / 2.),
                camera_transform: Transformation::identity(),
                zoom: 1.,
                scale: 1.,
                time_scale: 1.,
                config,
        })
    }

    fn draw(&mut self, frame: &mut Frame, timer: &Timer) {
        frame.clear(Color::BLACK);
        
        let mut target = frame.as_target();
        let mut camera = target.transform(self.camera_transform);
        
        Self::update_acceleration_par(self);
        self.entities.par_iter_mut().for_each(|particle| { 
            particle.velocity += particle.acceleration * timer.next_tick_proximity() * self.time_scale;
            particle.position += particle.velocity * timer.next_tick_proximity() * self.time_scale;
        });

        let sprite_offset = Vector::new(self.config.sprite_width as f32 * self.config.sprite_scale / 2., self.config.sprite_height as f32 * self.config.sprite_scale / 2.);
        let sprites = self.entities.par_iter().map(|particle| {
            // println!("Position: {} Screen Position: {} Veclocity: {}", particle.position, particle.position * self.scale, particle.velocity);
            Sprite {
                source: self.particle_sprite_quad,
                position: (particle.position * self.scale) - sprite_offset,
                scale: (self.config.sprite_scale, self.config.sprite_scale),
            }
        });
    
        self.batch.clear();
        self.batch.par_extend(sprites);
        self.batch.draw(&mut camera);
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        let cursor_position = input.mouse().cursor_position();
        let x_position = (cursor_position.x - self.camera_position.x) / self.scale;
        let y_position = (cursor_position.y - self.camera_position.y) / self.scale;
        if input.mouse().is_button_pressed(mouse::Button::Left) {
            self.entities.push(Particle::new(x_position, y_position, 1000., self.entities.len() as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key1) {
            self.entities.push(Particle::new(x_position, y_position, 1.0e6, self.entities.len() as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key2) {
            self.entities.push(Particle::new(x_position, y_position, 1.0e10, self.entities.len() as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key3) {
            Self::generate_random_particles(self, 4000);
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key4) {
            Self::generate_solar_system(self);
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
            self.time_scale = 1.;
            self.scale = 1.;
            self.entities = Vec::new();
        }
        if input.keyboard().was_key_released(keyboard::KeyCode::P) {
            println!("Number of particles = {}", self.entities.len());
        }

        self.camera_transform = Transformation::scale(self.zoom) * Transformation::translate(
            Vector::new(self.camera_position.x, self.camera_position.y));
    }
}
