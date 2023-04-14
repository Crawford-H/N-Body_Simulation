use std::sync::Arc;
use std::time::Instant;
use std::thread;

use coffee::ui::slider::State;
use coffee::ui::{Renderer, UserInterface, Element, Column, Justify, Text, Slider, slider, Align, Row};
use coffee::{load::Task, Game, Timer};
use coffee::graphics::{Window, Color, Frame, Batch, Image, Sprite, Rectangle, Vector, Transformation, Point};
use coffee::input::{KeyboardAndMouse, mouse, keyboard};
use rayon::prelude::*;
use rand::Rng;

use crate::particle::{Particle, solar_system, net_acceleration};
use crate::config::Config;

pub enum UpdateParticleAlgorithm {
    Sequential,
    Threading,
    Rayon,
}

#[derive()]
pub struct Application {
    entities: Vec<Particle>, // vector to store locations of particles
    mass: f32,

    // variables for rendering particles
    particle_sprite_quad: Rectangle<u16>,
    batch: Batch,
    camera_transform: Transformation,
    zoom: f32,
    camera_position: Point,
    scale: f32,

    // variables for tracking time
    time: Instant,
    time_scale: f32,

    // config to store various constants
    config: Config, 
    algorithm: UpdateParticleAlgorithm,

    // ui variables
    time_scale_slider: State,
    mass_slider: State,
    x_velocity_slider: State,
    y_velocity_slider: State,
    velocity: (f32, f32),
}

impl Application {
    fn update_acceleration(&mut self) {
        match self.algorithm {
            UpdateParticleAlgorithm::Sequential => self.update_acceleration_series(),
            UpdateParticleAlgorithm::Threading => self.update_acceleration_threads(),
            UpdateParticleAlgorithm::Rayon => self.update_acceleration_par_for(),
        }
    }

    fn update_acceleration_par_for(&mut self) {
        let entities_clone = self.entities.clone();

        self.entities.par_iter_mut().for_each(move |particle| {
            particle.acceleration = net_acceleration(entities_clone.iter(), particle)
        });
    }

    fn update_acceleration_series(&mut self) {
        let entities_clone = self.entities.clone();
        
        for particle in self.entities.iter_mut() {
            particle.acceleration = net_acceleration(entities_clone.iter(), particle)
        };
    }

    fn update_acceleration_threads(&mut self) {
        let entities = Arc::new(self.entities.clone());
        let num_threads = self.config.num_threads;
        
        let mut handles: Vec<thread::JoinHandle<Vec<Particle>>> = vec![];
        for i in 0..self.config.num_threads {
            let entities = Arc::clone(&entities);
            handles.push(thread::spawn(move || {
                entities
                    .iter()
                    .skip(i)
                    .step_by(num_threads)
                    .map(|particle| {
                        let mut temp = particle.clone();
                        temp.acceleration = net_acceleration(entities.iter(), particle);
                        temp
                    })
                    .collect()
            }));
        }

        self.entities = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .flatten()
            .collect();
    }

    fn update_position(&mut self, dt: f32) {
        self.entities.par_iter_mut().for_each(|particle| {
            particle.velocity += particle.acceleration * dt * self.time_scale;
            particle.position += particle.velocity * dt * self.time_scale;
        });
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
        self.time_scale = 500000.;
        // generate the sun
        self.entities.extend(solar_system(self.entities.len() as u16));
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
                mass: 100.,
                entities: Vec::new(),
                batch: Batch::new(sprites),
                particle_sprite_quad: Rectangle { x: 0, y: 0, height: config.sprite_height, width: config.sprite_width },
                camera_position: Point::new(config.screen_width as f32 / 2., config.screen_height as f32 / 2.),
                camera_transform: Transformation::identity(),
                zoom: 1.,
                scale: 1.,
                time_scale: 100.,
                config,
                algorithm: UpdateParticleAlgorithm::Rayon,
                time: Instant::now(),
                time_scale_slider: slider::State::new(),
                mass_slider: slider::State::new(),
                x_velocity_slider: slider::State::new(),
                y_velocity_slider: slider::State::new(),
                velocity: (0., 0.),
        })
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
        
        let mut target = frame.as_target();
        let mut camera = target.transform(self.camera_transform);
        
        self.update_acceleration();
        self.update_position(self.time.elapsed().as_secs_f32());
        self.time = Instant::now();

        let sprite_offset = Vector::new(self.config.sprite_width as f32 * self.config.sprite_scale / 2., self.config.sprite_height as f32 * self.config.sprite_scale / 2.);
        let sprites = self.entities
            .par_iter()
            .map(|particle| {
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
            self.entities.push(Particle::new((x_position, y_position), self.velocity, self.mass, self.entities.len() as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key2) {
            self.entities.push(Particle::new((x_position, y_position), self.velocity, 1.0e10, self.entities.len() as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key3) {
            self.generate_random_particles(4000);
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key4) {
            self.generate_solar_system();
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key5) {
            println!("Rayon");
            self.algorithm = UpdateParticleAlgorithm::Rayon;
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key6) {
            println!("Threads");
            self.algorithm = UpdateParticleAlgorithm::Threading;
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key7) {
            println!("Sequential");
            self.algorithm = UpdateParticleAlgorithm::Sequential;
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

pub enum Message {
    _AlgorithmChanged(UpdateParticleAlgorithm),
    TimeScaleChanged(f32),
    MassChanged(f32),
    XVelocityChanged(f32),
    YVelocityChanged(f32),
    _ScaleChanged(f32),
}

impl UserInterface for Application {
    type Message = Message;
    type Renderer = Renderer;

    fn react(&mut self, message: Message, _window: &mut Window) {
        match message {
            Message::_AlgorithmChanged(val) => self.algorithm = val,
            Message::TimeScaleChanged(val) => self.time_scale = val,
            Message::MassChanged(val) => self.mass = val,
            Message::_ScaleChanged(val) => self.scale = val,
            Message::XVelocityChanged(val) => self.velocity.0 = val,
            Message::YVelocityChanged(val) => self.velocity.1 = val,
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
                .push(Text::new(format!("Number of particles: {}", self.entities.len()).as_str()))
                .push(Text::new(format!("Time Scale: {} virtual seconds / real seconds", self.time_scale).as_str()).size(20))
                .push(Slider::new(&mut self.time_scale_slider, 0.1..=5.0e3, self.time_scale, Message::TimeScaleChanged))
                .push(Text::new(format!("Spawned Particle Mass: {} kg", self.mass).as_str()).size(20))
                .push(Slider::new(&mut self.mass_slider, 0.1..=1.0e6, self.mass, Message::MassChanged))
            )
            .push(Column::new()
                .push(Text::new(format!("Velocity: {:?} m/s", self.velocity).as_str()))
                .push(Text::new("X"))
                .push(Slider::new(&mut self.x_velocity_slider, -1.0..=1.0, self.velocity.0, Message::XVelocityChanged))
                .push(Text::new("Y"))
                .push(Slider::new(&mut self.y_velocity_slider, -1.0..=1.0, self.velocity.1, Message::YVelocityChanged))
            )
            .into()
    }
}
