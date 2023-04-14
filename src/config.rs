use dotenv::dotenv;


pub struct Config {
    pub sprite_width: u16,
    pub sprite_height: u16,
    pub sprite_scale: f32,
    pub star_sprite_path: String,
    pub screen_width: u16,
    pub screen_height: u16,
    pub num_threads: usize,
}

impl Config {
    pub fn new() -> Config {
        dotenv().ok();

        Config {
            sprite_width: std::env::var("SPRITE_WIDTH").expect("Environment variable SPRITE_WIDTH missing").parse().unwrap(),
            sprite_height: std::env::var("SPRITE_HEIGHT").expect("Environment variable SPRITE_HEIGHT missing").parse().unwrap(),
            sprite_scale: std::env::var("SPRITE_SCALE").expect("Environment variable SPRITE_SCALE missing").parse().unwrap(),
            star_sprite_path: std::env::var("STAR_SPRITE").expect("Environment variable STAR_SPRITE missing"),
            screen_width: std::env::var("SCREEN_WIDTH").expect("Environment variable SPRITE_SCALE missing").parse().unwrap(),
            screen_height: std::env::var("SCREEN_HEIGHT").expect("Environment variable SPRITE_SCALE missing").parse().unwrap(),
            num_threads: std::env::var("NUM_THREADS").expect("Environment variable SPRITE_SCALE missing").parse().unwrap(),
        }
    }
}