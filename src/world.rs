use glam::DVec2;

use crate::particle::Particle;

pub trait World {
    /// Updates the particles with a given delta time.
    fn update(&mut self, dt: f64);
    /// Add a new [`Particle`] to the world.
    fn create_particle(&mut self, position: DVec2, velocity: DVec2, mass: f64);
    /// Returns a copy of the Particles 
    fn get_particles(&mut self) -> Vec<Particle>;
}
