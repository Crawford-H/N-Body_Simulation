use std::sync::Arc;
use std::time::Instant;
use std::thread;

use coffee::ui::slider::State;
use coffee::ui::{Renderer, UserInterface, Element, Column, Justify, Text, Slider, slider, Align, Row, Button, button};
use coffee::{load::Task, Game, Timer};
use coffee::graphics::{Window, Color, Frame, Batch, Image, Sprite, Rectangle, Vector, Transformation, Point};
use coffee::input::{KeyboardAndMouse, mouse, keyboard};
use rayon::prelude::*;
use rand::Rng;

use crate::benchmark::{Benchmark, BenchmarkStatus};
use crate::particle::{Particle, solar_system, net_acceleration};
use crate::config::Config;

pub enum UpdateParticleAlgorithm {
    Sequential,
    Threading,
    ParallelFor,
}

#[derive()]
pub struct Application {
    entities: Vec<Particle>, // vector to store locations of particles
    mass: f32,

    // variables for rendering particles
    particle_sprite_quad: Rectangle<u16>,
    batch: Batch,
    camera_transform: Transformation,
    camera_position: Point,
    scale: f32,

    // variables for tracking time
    time: Instant,
    time_scale: f32,
    benchmark: Benchmark,

    // config to store various constants
    config: Config, 
    algorithm: UpdateParticleAlgorithm,

    // ui variables
    time_scale_slider: State,
    mass_slider: State,
    x_velocity_slider: State,
    y_velocity_slider: State,
    velocity: (f32, f32),
    increment_threads: button::State,
    decrement_threads: button::State,
}

impl Application {
    /// Update position of particles depending on the algorithm selected
    fn update_entity_position(&mut self, dt: f32) {
        match self.algorithm {
            UpdateParticleAlgorithm::Sequential => self.update_position_series(dt),
            UpdateParticleAlgorithm::Threading => self.update_position_threads(dt),
            UpdateParticleAlgorithm::ParallelFor => self.update_position_par_for(dt),
        }
    }

    /// Updated positon of particles using the Rayon library which allows for parallel iteration using the same syntax as it would be in series.
    fn update_position_par_for(&mut self, dt: f32) {
        let entities_clone = self.entities.clone();
        let time_scale = self.time_scale;

        self.entities.par_iter_mut().for_each(move |particle| {
            particle.acceleration = net_acceleration(entities_clone.iter(), particle);
            particle.velocity += particle.acceleration * dt * time_scale;
            particle.position += particle.velocity * dt * time_scale;
        });
    }

    /// Calculate the position of the particles in series.
    fn update_position_series(&mut self, dt: f32) {
        let entities_clone = self.entities.clone();
        
        for particle in self.entities.iter_mut() {
            particle.acceleration = net_acceleration(entities_clone.iter(), particle);
            particle.velocity += particle.acceleration * dt * self.time_scale;
            particle.position += particle.velocity * dt * self.time_scale;
        };
    }

    /// Update the position of the particles in parallel using the standard library threads.
    fn update_position_threads(&mut self, dt: f32) {
        let entities = Arc::new(self.entities.clone());
        let num_threads = self.config.num_threads;
        let time_scale = self.time_scale;
        
        // create threads then caculate positions of a subset of the entities
        let mut handles: Vec<thread::JoinHandle<Vec<Particle>>> = vec![];
        for i in 0..num_threads {
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

        // get the return values from the threads then set the updated list as the positions
        self.entities = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .flatten()
            .map(|particle| {
                let mut temp = particle.clone();
                temp.velocity += particle.acceleration * dt * time_scale;
                temp.position += particle.velocity * dt * time_scale;
                temp
            })
            .collect();
    }

    /// Generate particles randomly in different position with different velocities.
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

    /// Adds particles that represent the solar system to the entities list.
    fn generate_solar_system(&mut self) {
        self.scale = 1500. / 4495.060e9;
        self.time_scale = 500000.;
        self.entities.clear();
        self.entities.extend(solar_system(self.entities.len() as u16));
    }
}

impl Game for Application {
    type Input = KeyboardAndMouse;
    type LoadingScreen = ();
    const TICKS_PER_SECOND: u16 = 64;
    const DEBUG_KEY: Option<keyboard::KeyCode> = Some(keyboard::KeyCode::F12);

    /// Called once at the beginning of the program. Loads the sprite for the particles the sets the initial values.
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
                scale: 1.,
                time_scale: 100.,
                config,
                algorithm: UpdateParticleAlgorithm::ParallelFor,
                time: Instant::now(),
                time_scale_slider: slider::State::new(),
                mass_slider: slider::State::new(),
                x_velocity_slider: slider::State::new(),
                y_velocity_slider: slider::State::new(),
                velocity: (0., 0.),
                benchmark: Benchmark::new(1000),
                increment_threads: button::State::new(),
                decrement_threads: button::State::new(),
        })
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
        
        // update camera position
        let mut target = frame.as_target();
        self.camera_transform = Transformation::translate(Vector::new(self.camera_position.x, self.camera_position.y));
        let mut camera = target.transform(self.camera_transform);
        
        // update particles and benchmark times
        let benchmark_time = Instant::now();
        self.update_entity_position(self.time.elapsed().as_secs_f32());
        self.time = Instant::now();

        match self.benchmark.status {
            BenchmarkStatus::Running => self.benchmark.increase_elapsed_time(benchmark_time.elapsed().as_secs_f64()),
            BenchmarkStatus::Finished => self.benchmark.status = BenchmarkStatus::Finished,
            BenchmarkStatus::Paused => {},
        }

        // create sprites for rendering with updated positions
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
    
        // draw sprites
        self.batch.clear();
        self.batch.par_extend(sprites);
        self.batch.draw(&mut camera);
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        // store calculations for cursor positions
        let cursor_position = input.mouse().cursor_position();
        let x_position = (cursor_position.x - self.camera_position.x) / self.scale;
        let y_position = (cursor_position.y - self.camera_position.y) / self.scale;

        if input.mouse().is_button_pressed(mouse::Button::Left) {
            self.entities.push(Particle::new((x_position, y_position), self.velocity, self.mass, self.entities.len() as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key1) {  // start a benchmark
            self.benchmark = Benchmark::new(1000);
            self.benchmark.start();
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key2) { // create a large particle
            self.entities.push(Particle::new((x_position, y_position), self.velocity, 1.0e12, self.entities.len() as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key3) { // create 4000 random particles
            self.generate_random_particles(4000);
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key4) { // generate the solar system
            self.generate_solar_system();
        }

        // change the algorithm used to calculate physics
        if input.keyboard().was_key_released(keyboard::KeyCode::Tab) {
            match self.algorithm {
                UpdateParticleAlgorithm::Sequential => self.algorithm = UpdateParticleAlgorithm::ParallelFor,
                UpdateParticleAlgorithm::Threading => self.algorithm = UpdateParticleAlgorithm::Sequential,
                UpdateParticleAlgorithm::ParallelFor => self.algorithm = UpdateParticleAlgorithm::Threading,
            }
        }

        // camera movements
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

        // reset values
        if input.keyboard().was_key_released(keyboard::KeyCode::R) {
            self.time_scale = 1.;
            self.scale = 1.;
            self.velocity = (0., 0.);
            self.entities = Vec::new();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    TimeScaleChanged(f32),
    MassChanged(f32),
    XVelocityChanged(f32),
    YVelocityChanged(f32),
    IncrementThreads,
    DecrementThreads,
}

impl UserInterface for Application {
    type Message = Message;
    type Renderer = Renderer;

    fn react(&mut self, message: Message, _window: &mut Window) {
        match message {
            Message::TimeScaleChanged(val) => self.time_scale = val,
            Message::MassChanged(val) => self.mass = val,
            Message::XVelocityChanged(val) => self.velocity.0 = val,
            Message::YVelocityChanged(val) => self.velocity.1 = val,
            Message::IncrementThreads => {self.config.num_threads += 1; println!("{}", self.config.num_threads)},
            Message::DecrementThreads => {self.config.num_threads -= if self.config.num_threads == 0 { 0 } else { 1 }; println!("{}", self.config.num_threads)},
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
                .push(Text::new(format!("Scale: {} meter(s) / pixel", 1. / self.scale).as_str()))
                .push(Text::new(format!("Number of particles: {}", self.entities.len()).as_str()))
                .push(Text::new(format!("Time Scale: {:.5} seconds / actual seconds", self.time_scale).as_str()).size(20))
                .push(Slider::new(&mut self.time_scale_slider, 0.1..=1.0e3, self.time_scale, Message::TimeScaleChanged))
                .push(Text::new(format!("Spawned Particle Mass: {} kg", self.mass).as_str()).size(20))
                .push(Slider::new(&mut self.mass_slider, 0.1..=1.0e6, self.mass, Message::MassChanged))
            )
            .push(Column::new()
                .padding(10)
                .push(Text::new(format!("Velocity: {:?} m/s", self.velocity).as_str()))
                .push(Text::new("X"))
                .push(Slider::new(&mut self.x_velocity_slider, -1.0..=1.0, self.velocity.0, Message::XVelocityChanged))
                .push(Text::new("Y"))
                .push(Slider::new(&mut self.y_velocity_slider, -1.0..=1.0, self.velocity.1, Message::YVelocityChanged))
            )
            .push(Column::new()
                .padding(10)
                .push(Text::new(format!("Algorithm: {}", match self.algorithm {
                    UpdateParticleAlgorithm::ParallelFor=> "Parallel For Loop",
                    UpdateParticleAlgorithm::Sequential => "Multi-threaded",
                    UpdateParticleAlgorithm::Threading => "Series", }).as_str()))
                .push(Text::new(format!("Number of Threads: {}", self.config.num_threads).as_str()))
                .push(Row::new()
                    .padding(10)
                    .push(Button::new(&mut self.increment_threads, "+")
                        .on_press(Message::IncrementThreads)
                        .class(button::Class::Positive))
                    .push(Button::new(&mut self.decrement_threads, "-")
                        .on_press(Message::DecrementThreads)
                        .class(button::Class::Secondary))
                )
            )
            .into()
    }
}
