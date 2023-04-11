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
    pub fn new(position_x: f32, position_y: f32, mass: f32, id: u16) -> Particle {
        Particle { 
            velocity: Vector::new(0., 0.), 
            position: Point::new(position_x, position_y), 
            mass,
            acceleration: Vector::new(0., 0.),
            id,
        }
    }
}

pub fn acceleration(lhs: &Particle, rhs: &Particle) -> Vector {
    let distance: Vector = lhs.position - rhs.position;
    let acceleration = (-1. * G * rhs.mass) / distance.magnitude_squared();

    if acceleration.is_nan() {
        Vector::new(0., 0.) 
    } else {
        acceleration * (distance / distance.magnitude())
    }
}

