use std::sync::{Arc, Barrier, RwLock};
use std::thread::{self, JoinHandle};

use glam::DVec2;

use crate::particle::{net_acceleration, Particle};

pub struct World {
    pub particles: Arc<RwLock<Vec<Particle>>>,
    pub particle_count: usize,
    dt: Arc<RwLock<f64>>,
    barrier: Arc<Barrier>,
    threads: Vec<JoinHandle<()>>,
    num_threads: usize,
}

impl World {
    /// Creates a new [`World`] with a given amount of worker threads.
    pub fn new(num_threads: usize) -> Self {
        let mut world = World {
            particles: Arc::new(RwLock::new(Vec::new())),
            threads: Vec::new(),
            dt: Arc::new(RwLock::new(0.)),
            particle_count: 0,
            barrier: Arc::new(Barrier::new(num_threads)),
            num_threads,
        };
        world.init_worker_threads(num_threads);
        world
    }

    /// Updates the particles with a given delta time.
    pub fn update(&mut self, dt: f64) {
        // update the delta time for threads to use
        {
            *self.dt.write().unwrap() = dt;
        }
        // main thread starts processing which starts worker threads also as barrier will be unlocked.
        process_particles(
            &self.barrier,
            &self.particles,
            &self.dt,
            0,
            self.num_threads,
        );
    }

    /// Add a new [`Particle`] to the world.
    pub fn create_particle(&mut self, position: DVec2, velocity: DVec2, mass: f64) {
        self.particles.write().unwrap().push(Particle {
            id: self.particle_count,
            velocity,
            position,
            mass,
        });
        self.particle_count += 1;
        println!("Particle {} added to world", self.particle_count);
    }

    /// Generates worker threads to calculate positions and velocities of particles
    fn init_worker_threads(&mut self, num_threads: usize) {
        for thread_id in 1..num_threads {
            // clone pointers required for threads
            let barrier = Arc::clone(&self.barrier);
            let dt = Arc::clone(&self.dt);
            let particles = Arc::clone(&self.particles);
            // create worker threads which will just loop processing particles
            self.threads.push(thread::spawn(move || loop {
                process_particles(&barrier, &particles, &dt, thread_id, num_threads);
            }))
        }
    }
}

fn process_particles(
    barrier: &Arc<Barrier>,
    particles: &Arc<RwLock<Vec<Particle>>>,
    dt: &Arc<RwLock<f64>>,
    thread_id: usize,
    num_threads: usize,
) {
    // wait until all threads ready to process particles, this will be locked until the main thread calls this function which will happen when the update method is called
    let _ = barrier.wait();

    let dt_copy = { *dt.read().unwrap() }; // get the dt to calculate new velocities and positions

    // calculate accelerations of particles
    let particles_read_lock = particles.read().unwrap();
    let velocities: Vec<DVec2> = particles_read_lock
        .iter()
        .skip(thread_id)
        .step_by(num_threads)
        .map(|particle| net_acceleration(particle, &particles_read_lock) * dt_copy)
        .collect();
    drop(particles_read_lock);

    // update particle velocities and position with accelerations calculated
    let mut particles_write_lock = particles.write().unwrap();
    particles_write_lock
        .iter_mut()
        .skip(thread_id)
        .step_by(num_threads)
        .zip(velocities)
        .for_each(|(particle, velocities)| {
            particle.velocity += velocities;
            particle.position += particle.velocity * dt_copy;
        });
    drop(particles_write_lock);

    // wait until each thread is finished updating particle positions
    let _ = barrier.wait();
}
