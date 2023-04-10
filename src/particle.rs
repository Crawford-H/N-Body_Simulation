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

    pub fn calculate_acceleration(&mut self, other: &Particle) {
        if self.id == other.id { return; }

        let distance: Vector = self.position - other.position;
        let acceleration = (-1. * G * other.mass) / distance.magnitude_squared();

        if acceleration.is_nan() { return; } 

        self.acceleration += acceleration * (distance / distance.magnitude());
    }
}

