use glam::DVec2;

#[derive(Clone, Debug)]
pub struct Particle {
    pub id: usize,
    pub velocity: DVec2,
    pub position: DVec2,
    pub mass: f64,
}

fn acceleration(lhs: &Particle, rhs: &Particle) -> DVec2 {
    let distance = lhs.position.distance(rhs.position);
    let acceleration_magnitude = (-6.67430e-11 * rhs.mass) / (distance * distance);
    if acceleration_magnitude.is_nan() {
        DVec2::new(0., 0.)
    } else {
        acceleration_magnitude * ((lhs.position - rhs.position) / distance)
    }
}

/// Calculates the net acceleartiong on a particle caused by each particle in a vector
pub fn net_acceleration(particle: &Particle, particles: &[Particle]) -> DVec2 {
    particles
        .iter()
        .filter(|other| particle.id != other.id)
        .map(|other| acceleration(particle, other))
        .sum()
}
