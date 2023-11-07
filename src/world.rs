use std::sync::{Arc, Barrier, atomic::Ordering};
use std::thread::{self, JoinHandle};

use rayon::prelude::*;
use atomic_float::AtomicF64;
use glam::DVec2;
use parking_lot::RwLock;

use crate::particle::Particle;

pub trait World {
    /// Updates the particles with a given delta time.
    fn update(&mut self, dt: f64);
    /// Add a new [`Particle`] to the world.
    fn create_particle(&mut self, position: DVec2, velocity: DVec2, mass: f64);
    /// Returns a copy of the Particles 
    fn get_particles(&mut self) -> Vec<Particle>;
}

/// Stores the entities in the world as a vector of Particles and 
/// handles updating velocities and positions of the particles.
/// 
/// The positions of the particles are calculated using a parallel
/// iterator from the rayon library.
pub struct RayonWorld {
    pub particles: Vec<Particle>,
}

impl World for RayonWorld {
    fn update(&mut self, dt: f64) {
        let particles_clone = self.particles.clone();
        self.particles.par_iter_mut().for_each(|particle| {
            let acceleration = particle.net_acceleration(&particles_clone) * dt;
            particle.velocity += acceleration * dt;
            particle.position += particle.velocity * dt;
        });
    }

    fn create_particle(&mut self, position: glam::DVec2, velocity: glam::DVec2, mass: f64) {
        self.particles.push(Particle { 
            id: self.particles.len(), 
            velocity, 
            position, 
            mass 
        });
    }

    fn get_particles(&mut self) -> Vec<Particle> {
        self.particles.clone()
    }
}

/// Stores the entities in the world as a vector of Particles and 
/// handles updating velocities and positions of the particles.
/// 
/// The positions of the particles are calculated using a simple for loop.
pub struct SequentialWorld {
    pub particles: Vec<Particle>,
}

impl World for SequentialWorld {
    fn update(&mut self, dt: f64) {
        let particles_clone = self.particles.clone();
        for particle in self.particles.iter_mut() {
            let acceleration = particle.net_acceleration(&particles_clone) * dt;
            particle.velocity += acceleration * dt;
            particle.position += particle.velocity * dt;
        }
    }

    fn create_particle(&mut self, position: glam::DVec2, velocity: glam::DVec2, mass: f64) {
        self.particles.push(Particle { 
            id: self.particles.len(), 
            velocity, 
            position, 
            mass 
        });
    }

    fn get_particles(&mut self) -> Vec<Particle> {
        self.particles.clone()
    }
}

pub struct WorkerThreadsWorld {
    pub particles: Arc<RwLock<Vec<Particle>>>,
    pub particle_count: usize,
    dt: Arc<AtomicF64>,
    barrier: Arc<Barrier>,
    threads: Vec<JoinHandle<()>>,
    num_threads: usize,
}

impl World for WorkerThreadsWorld {
    fn update(&mut self, dt: f64) {
        // update the delta time for threads to use
        self.dt.store(dt, Ordering::Release);
        
        // main thread starts processing which starts worker threads also as barrier will be unlocked.
        process_particles(
            &self.barrier,
            &self.particles,
            &self.dt,
            0,
            self.num_threads,
        );
    }

    fn create_particle(&mut self, position: DVec2, velocity: DVec2, mass: f64) {
        self.particles.write().push(Particle {
            id: self.particle_count,
            velocity,
            position,
            mass,
        });
        self.particle_count += 1;
    }

    fn get_particles(&mut self) -> Vec<Particle> {
        self.particles.read().clone()
    }
}

impl WorkerThreadsWorld {
    /// Creates a new [`World`] with a given amount of worker threads.
    pub fn new(num_threads: usize, particles: Vec<Particle>) -> Self {
        let mut world = WorkerThreadsWorld {
            particles: Arc::new(RwLock::new(particles)),
            threads: Vec::new(),
            dt: Arc::new(AtomicF64::new(0.)),
            particle_count: 0,
            barrier: Arc::new(Barrier::new(num_threads)),
            num_threads,
        };
        world.init_worker_threads(num_threads);
        world
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
    dt: &Arc<AtomicF64>,
    thread_id: usize,
    num_threads: usize,
) {
    // wait until all threads ready to process particles, this will be locked until the main thread calls this function which will happen when the update method is called
    let _ = barrier.wait();

    let dt_copy = dt.load(Ordering::Acquire); // get the dt to calculate new velocities and positions

    // calculate accelerations of particles
    let particles_read = particles.read().clone();
    let velocities: Vec<DVec2> = particles_read
        .iter()
        .skip(thread_id)
        .step_by(num_threads)
        .map(|particle| particle.net_acceleration(&particles_read) * dt_copy)
        .collect();

    // update particle velocities and position with accelerations calculated
    particles.write()
        .iter_mut()
        .skip(thread_id)
        .step_by(num_threads)
        .zip(velocities)
        .for_each(|(particle, velocity)| {
            particle.velocity += velocity;
            particle.position += particle.velocity * dt_copy;
        });

    // wait until each thread is finished updating particle positions
    let _ = barrier.wait();
}
