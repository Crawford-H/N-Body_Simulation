use coffee::{load::Task, Game, Timer};
use coffee::graphics::{Window, Color, Frame, Rectangle, Point, Shape, Mesh};
use coffee::input::{KeyboardAndMouse, mouse};

use glam::{Vec2, vec2};

pub struct Particle {
    velocity: Vec2,
    position: Point,
}

pub struct Application {
    particles: Vec<Particle>,
}

impl Game for Application {
    type Input = KeyboardAndMouse; // No input data
    type LoadingScreen = ();

    fn load(_window: &Window) -> Task<Application> {
        Task::succeed(|| Application { particles: Vec::new() })
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        // Clear the current frame
        frame.clear(Color::BLACK);

        for particle in self.particles.iter() {
            let mut mesh = Mesh::new_with_tolerance(0.1);
            let shape = Shape::Rectangle(Rectangle {
                x: particle.position.x,
                y: particle.position.y,
                width: 1.,
                height: 1.,
            });
            mesh.fill(shape, Color::RED);
            mesh.draw(&mut frame.as_target());
        }
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        if input.mouse().is_button_pressed(mouse::Button::Left) {
            self.particles.push(Particle { velocity: vec2(0., 0.), position: input.mouse().cursor_position() });
        }

        // for point in input.mouse().button_clicks(mouse::Button::Left).iter() {
        //     self.particles.push(Particle { velocity: vec2(0., 0.), position: *point });
        // }
    }
}