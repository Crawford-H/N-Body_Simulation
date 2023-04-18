use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, Barrier, Mutex, Condvar};
use std::time::Instant;
use std::thread::{self, JoinHandle};

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
    // Sequential,
    Threading,
    // ParallelFor,
}

pub struct Application {
    entities: Arc<RwLock<Vec<Particle>>>, // vector to store locations of particles
    handles: Vec<JoinHandle<()>>,
    cond: Arc<(Mutex<bool>, Condvar)>,
    dt: Arc<RwLock<f32>>,
    flag: Arc<AtomicBool>,
    
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
    mass: f32,
    time_scale_slider: State,
    mass_slider: State,
    x_velocity_slider: State,
    y_velocity_slider: State,
    velocity: (f32, f32),
    increment_threads: button::State,
    decrement_threads: button::State,
}

impl Application {
    fn init_threads(&mut self) -> Vec<JoinHandle<()>> {
        let num_threads = self.config.num_threads;
        let mut handles = Vec::new();
        let barrier = Arc::new(Barrier::new(num_threads));

        for i in 0..self.config.num_threads {
            let entities = Arc::clone(&self.entities);
            let barrier = Arc::clone(&barrier);
            let condition = Arc::clone(&self.cond);
            let dt_lock = Arc::clone(&self.dt);
            let flag = Arc::clone(&self.flag);

            handles.push(thread::spawn(move || {
                // wait until unparked to calculate frame
                loop {
                    while !flag.load(Ordering::Acquire) {
                        // println!("Parking thread: {}", flag.load(Ordering::Acquire));
                        // thread::park();
                        // println!("Thread unparked");
                    }
    
                    // once unparked, we can calculate the frame with the dt
                    let read = entities.read().unwrap();
                    let accelerations: Vec<Vector> = read.iter()
                        .skip(i)
                        .step_by(num_threads)
                        .map(|particle| {
                            net_acceleration(read.iter(), particle)
                        })
                        .collect();
                    drop(read); // need to drop read lock to prevent a deadlock
    
                    let dt_guard = dt_lock.read().unwrap();
                    let dt = *dt_guard;
                    drop(dt_guard);
                    let mut write = entities.write().unwrap();
                    write.iter_mut()
                        .skip(i)
                        .step_by(num_threads)
                        .zip(accelerations)
                        .for_each(|(particle, acceleration)| {
                            particle.acceleration = acceleration;
                            particle.velocity += acceleration * dt;
                            particle.position += particle.velocity * dt;
                        });
                    drop(write);
    
                    // wait until each thread has calculated the frame
                    let is_leader = barrier.wait();
                    flag.store(false, Ordering::Release);
                    
                    // message main thread to continue loop
                    if is_leader.is_leader() {
                        // println!("Finished generating frame, setting cond variable");
                        let (lock, cvar) = &*condition;
                        let mut pending = lock.lock().unwrap();
                        *pending = false;
                        cvar.notify_one();
                    }
                }
                
            }));
        }
        handles
    }

    /// Generate particles randomly in different position with different velocities.
    fn generate_random_particles(&mut self, num_particles: i32) {
        let generator = &mut rand::thread_rng();
        let len = self.entities.read().unwrap().len();
        let mut entities = self.entities.write().unwrap();
        for _ in 0..num_particles {
            entities.push(Particle { 
                velocity: Vector::new(generator.gen_range(-1.0e-1..1.0e-1), generator.gen_range(-1.0e-1..1.0e-1)), 
                position: Point::new(generator.gen_range(-1.0e3..1.0e3), generator.gen_range(-1.0e3..1.0e3)), 
                mass: generator.gen_range(10.0e1..1.0e8), 
                acceleration: Vector::new(0., 0.),
                id: len as u16, 
            })
        }
    }

    /// Adds particles that represent the solar system to the entities list.
    fn generate_solar_system(&mut self) {
        let len = self.entities.read().unwrap().len();
        let mut entities = self.entities.write().unwrap();
        self.scale = 1500. / 4495.060e9;
        self.time_scale = 500000.;
        entities.clear();
        entities.extend(solar_system(len as u16));
    }
}

impl Game for Application {
    type Input = KeyboardAndMouse;
    type LoadingScreen = ();
    const TICKS_PER_SECOND: u16 = 128;
    const DEBUG_KEY: Option<keyboard::KeyCode> = Some(keyboard::KeyCode::F12);

    /// Called once at the beginning of the program. Loads the sprite for the particles the sets the initial values.
    fn load(_window: &Window) -> Task<Application> {
        let config = Config::new();
        Task::stage("Loading Sprites", Image::load(config.star_sprite_path.as_str())).map(|sprites| {
            let mut app = Application {
                handles: Vec::new(),
                cond: Arc::new((Mutex::new(true), Condvar::new())),
                flag: Arc::new(AtomicBool::new(true)),
                dt: Arc::new(RwLock::new(0.)),
                mass: 100.,
                entities: Arc::new(RwLock::new(Vec::new())),
                batch: Batch::new(sprites),
                particle_sprite_quad: Rectangle { x: 0, y: 0, height: config.sprite_height, width: config.sprite_width },
                camera_position: Point::new(config.screen_width as f32 / 2., config.screen_height as f32 / 2.),
                camera_transform: Transformation::identity(),
                scale: 1.,
                time_scale: 100.,
                config,
                algorithm: UpdateParticleAlgorithm::Threading,
                time: Instant::now(),
                time_scale_slider: slider::State::new(),
                mass_slider: slider::State::new(),
                x_velocity_slider: slider::State::new(),
                y_velocity_slider: slider::State::new(),
                velocity: (0., 0.),
                benchmark: Benchmark::new(1000),
                increment_threads: button::State::new(),
                decrement_threads: button::State::new(),
            };
            app.init_threads();
            app
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

        // update dt
        {   
            let mut dt = self.dt.write().unwrap();
            *dt = self.time.elapsed().as_secs_f32() * self.time_scale;
        }
        // unpark threads
        self.flag.store(true, Ordering::Release);
        // for handle in self.handles.iter() {
        //     handle.thread().unpark();
        // }
        
        // wait until threads done calculating frame
        let pair = Arc::clone(&self.cond);
        let (lock, cvar) = &*pair;
        let _guard = cvar.wait_while(lock.lock().unwrap(), |pending| { *pending }).unwrap();
        
        self.time = Instant::now();
        match self.benchmark.status {
            BenchmarkStatus::Running => self.benchmark.increase_elapsed_time(benchmark_time.elapsed().as_secs_f64()),
            BenchmarkStatus::Finished => self.benchmark.status = BenchmarkStatus::Finished,
            BenchmarkStatus::Paused => {},
        }

        // create sprites for rendering with updated positions
        let sprite_offset = Vector::new(self.config.sprite_width as f32 * self.config.sprite_scale / 2., self.config.sprite_height as f32 * self.config.sprite_scale / 2.);
        let lock = self.entities.read().unwrap();
        let mapper = |particle: &Particle| {
            Sprite {
                source: self.particle_sprite_quad,
                position: (particle.position * self.scale) - sprite_offset,
                scale: (self.config.sprite_scale, self.config.sprite_scale),
        }};
        let sprites = lock.par_iter().map(mapper);
    
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
            let len = self.entities.read().unwrap().len();
            let mut entities = self.entities.write().unwrap();
            entities.push(Particle::new((x_position, y_position), self.velocity, self.mass, len as u16));
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key1) {  // start a benchmark
            self.benchmark = Benchmark::new(1000);
            self.benchmark.start();
        }

        if input.keyboard().was_key_released(keyboard::KeyCode::Key2) { // create a large particle
            let len = self.entities.read().unwrap().len();
            let mut entities = self.entities.write().unwrap();
            entities.push(Particle::new((x_position, y_position), self.velocity, 1.0e12, len as u16));
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
                // UpdateParticleAlgorithm::Sequential => self.algorithm = UpdateParticleAlgorithm::ParallelFor,
                UpdateParticleAlgorithm::Threading => self.algorithm = UpdateParticleAlgorithm::Threading,
                // UpdateParticleAlgorithm::ParallelFor => self.algorithm = UpdateParticleAlgorithm::Threading,
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
            let mut lock = self.entities.write().unwrap();
            (*lock).clear();
            drop(lock);
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
        let entities = self.entities.read().unwrap();

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
                .push(Text::new(format!("Number of particles: {}", entities.len()).as_str()))
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
                    // UpdateParticleAlgorithm::ParallelFor=> "Parallel For Loop",
                    // UpdateParticleAlgorithm::Sequential => "Series",
                    UpdateParticleAlgorithm::Threading => "Multi-threaded", }).as_str()))
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
