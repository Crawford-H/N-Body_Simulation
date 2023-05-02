use glam::DVec2;

const NEG_G: f64 = -6.67430e-11;

#[derive(Clone, Debug)]
pub struct Particle {
    pub id: usize,
    pub velocity: DVec2,
    pub position: DVec2,
    pub mass: f64,
}

impl Particle {
    pub fn acceleration(&self, rhs: &Particle) -> DVec2 {
        let r = self.position - rhs.position;
        NEG_G * rhs.mass * r * r.length().powi(3).recip() // a = (-GM/|r|^2) * (r / |r|) = (-GMr) / |r|^3
    }

    pub fn net_acceleration(&self, particles: &[Particle]) -> DVec2 {
        particles
            .iter()
            .filter(|other| self.id != other.id)
            .map(|other| self.acceleration(other))
            .sum()
    }
}
