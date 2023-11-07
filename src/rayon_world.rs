use rayon::prelude::*;

use crate::{world::World, particle::Particle};

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