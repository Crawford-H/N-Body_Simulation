use coffee::graphics::{Vector, Point};



const G: f32 = 6.67430e-11;
const METERS_PER_UNIT: f32 = 10.;

#[derive(Clone)]
pub struct Particle {
    pub velocity: Vector,
    pub position: Point,
    pub mass: f32,
    pub acceleration: Vector,
}

impl Particle {

    pub fn new(position_x: f32, position_y: f32, mass: f32) -> Particle {
        Particle { 
            velocity: Vector::new(0., 0.), 
            position: Point::new(position_x, position_y), 
            mass,
            acceleration: Vector::new(0., 0.),
        }
    }

    pub fn calculate_acceleration(&mut self, other: &Particle) {
        let distance = Self::distance(&self.position,&other.position);
        
        let force = if distance > 0. {
            (self.mass * other.mass * G) / (distance * METERS_PER_UNIT).powi(2)
        } else {
            0.
        }.min(1_000_000.);

        self.acceleration.y += (force * (other.position.y - self.position.y)) / self.mass;
        self.acceleration.x += (force * (other.position.x - self.position.x)) / self.mass;
    }

    fn distance(x: &Point, y: &Point) -> f32 {
        ((y.x - x.x).powi(2) + (y.y - x.y).powi(2)).sqrt()
    }
}

