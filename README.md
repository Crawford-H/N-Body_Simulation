# 2D Physics Simulation For CSCI4060u
This is a 2D physics simulation written in Rust. This program simulates the force of gravity between each particle in the system. The particles move depending on how much time has elapsed, the mass of the particles, and the velocity of the particles. This simulation does not take into account other forces such as dark matter but uses the formula $a = -Gm_2 / r^2$ to calculate the acceleration of the particles, then uses the elapsed time to calculate the new velocity, then the new position.

## How to Run
1. Install Rust Cargo
1. If your computer is not compatiible with OpenGL, go into the Cargo.toml file and on line 10 change opengl to a platform your system supports. The platforms supported are `opengl`, `vulkan`, `dx12`, `dx11`, and `metal`.
1. Next open a terminal window in the base directory for the project and run `cargo run`

## Key Bindings
* Change the algorithm used for calculating each particle's position with <kbd>tab</kbd>.
* Move camera with <kbd>w</kbd>, <kbd>a</kbd>, <kbd>s</kbd>, and <kbd>d</kbd>
* Runs a benchmark on the algorithm calculating physics with <kbd>1</kbd>. The results are printed in the console.
* Spawn a very heavy particle with <kbd>2</kbd>.
* Use <kbd>3</kbd> to generate a large number of particles randomly.
* Use <kbd>4</kbd> to generate the solar system.
* Use <kbd>Left Click</kbd> to spawn particles depending on setting provided in the User Interface.
