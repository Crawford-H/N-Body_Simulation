use coffee::graphics::{Vector, Point};


const G: f32 = 6.67430e-11;

#[derive(Clone)]
pub struct Particle {
    pub id: u16,
    pub velocity: Vector,
    pub position: Point,
    pub mass: f32,
    pub acceleration: Vector,
}

impl Particle {
    pub fn new(position: (f32, f32), velocity: (f32, f32), mass: f32, id: u16) -> Particle {
        Particle { 
            velocity: Vector::new(velocity.0, velocity.1), 
            position: Point::new(position.0, position.1), 
            mass,
            acceleration: Vector::new(0., 0.),
            id,
        }
    }
}

/// Returns a Vector representing the acceleration of the particle on the left hand side imparted by the particle on the right hand side.
pub fn calculate_acceleration(lhs: &Particle, rhs: &Particle) -> Vector {
    let distance: Vector = lhs.position - rhs.position;
    let acceleration = (-1. * G * rhs.mass) / distance.magnitude_squared(); // acceleration = -G * M2 / r^2

    if acceleration.is_nan() {
        Vector::new(0., 0.) 
    } else {
        acceleration * (distance / distance.magnitude())
    }
}

/// Calculate the net acceleration on a given particle by summing the accelerations caused by every other particle in the vector.
pub fn net_acceleration(iterator: &Vec<Particle>, particle: &Particle) -> Vector {
    iterator.iter()
        .filter(|other| particle.id != other.id )
        .map(|other| calculate_acceleration(particle, other))
        .sum()
}

/// Returns a vector of particles containing the sun and planets in the solar system
pub fn solar_system(id: u16) -> Vec<Particle> {
    vec![
        Particle { // sun
            id: id, 
            velocity: Vector::new(0., 0.), 
            position: Point::new(0., 0.), 
            mass: 1.989e30, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { // mercury
            id: id + 1, 
            velocity: Vector::new(0., 47.36e3), 
            position: Point::new(57.909e9, 0.), 
            mass: 0.33011e24, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { // venus
            id: id + 2, 
            velocity: Vector::new(0., 35.02e3), 
            position: Point::new(108.209e9, 0.), 
            mass: 4.8675e24, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { // earth
            id: id + 3, 
            velocity: Vector::new(0., 29.78e3), 
            position: Point::new(149.596e9, 0.), 
            mass: 5.9724e24, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { // mars
            id: id + 4, 
            velocity: Vector::new(0., 24.07e3), 
            position: Point::new(227.923e9, 0.), 
            mass: 0.64171e24, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { // jupiter
            id: id + 5, 
            velocity: Vector::new(0., 13e3), 
            position: Point::new(778.57e9, 0.), 
            mass: 1898.19e24, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { //saturn
            id: id + 6, 
            velocity: Vector::new(0., 9.68e3), 
            position: Point::new(1433.529e9, 0.), 
            mass: 568.34e24, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { // uranus
            id: id + 7, 
            velocity: Vector::new(0., 6.80e3), 
            position: Point::new(2872.463e9, 0.), 
            mass: 86.813e24, 
            acceleration: Vector::new(0., 0.) 
        },
        Particle { //neptune
            id: id + 8, 
            velocity: Vector::new(0., 5.43e3), 
            position: Point::new(4495.060e9, 0.), 
            mass: 102.413e24, 
            acceleration: Vector::new(0., 0.) 
        },
    ]
}
