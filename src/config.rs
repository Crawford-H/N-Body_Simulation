use coffee::graphics::Rectangle;
use dotenv::dotenv;

#[derive(Clone, Debug)]
pub struct Config {
    // sprite parameters
    pub sprite_file: String,
    pub sprite_width: f32,
    pub sprite_height: f32,
    pub sprite_scale: f32,
    pub sprite_source: Rectangle<u16>,
    pub horizontal_offset: f32,
    pub vertical_offset: f32,
    // rendering and processing parameters
    pub num_threads: usize,
    pub screen_height: u32,
    pub screen_width: u32,
    // world parameters
    pub default_time_scale: f64,
    pub default_world_scale: f32,
}

impl Config {
    pub fn new() -> Config {
        dotenv().ok();
        let sprite_file = std::env::var("SPRITE_FILE").expect("Environment variable SPRITE_FILE missing").parse().unwrap();
        let sprite_width = std::env::var("SPRITE_WIDTH").expect("Environment variable SPRITE_WIDTH missing").parse().unwrap();
        let sprite_height = std::env::var("SPRITE_HEIGHT").expect("Environment variable SPRITE_HEIGHT missing").parse().unwrap();
        let sprite_scale = std::env::var("SPRITE_SCALE").expect("Environment variable SPRITE_SCALE missing").parse().unwrap();
        let num_threads = std::env::var("NUM_THREADS").expect("Environment variable NUM_THREADS missing").parse().unwrap();
        let screen_height = std::env::var("SCREEN_HEIGHT").expect("Environment variable SCREEN_HEIGHT missing").parse().unwrap();
        let screen_width = std::env::var("SCREEN_WIDTH").expect("Environment variable SCREEN_WIDTH missing").parse().unwrap();
        let default_time_scale = std::env::var("DEFAULT_TIME_SCALE").expect("Environment variable DEFAULT_TIME_SCALE missing").parse().unwrap();
        let default_world_scale = std::env::var("DEFAULT_WORLD_SCALE").expect("Environment variable DEFAULT_WORLD_SCALE missing").parse().unwrap();
        
        Config { 
            sprite_file,
            sprite_width,
            sprite_height,
            sprite_scale,
            sprite_source: Rectangle { x: 0, y: 0, width: sprite_height as u16, height: sprite_width as u16 }, 
            horizontal_offset: sprite_width * sprite_scale / 2., 
            vertical_offset: sprite_height * sprite_scale / 2.,
            num_threads,
            screen_height,
            screen_width,
            default_time_scale,
            default_world_scale, 
        }   
    }
}
