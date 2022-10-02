extern crate serde;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WindowSize {
    pub height: f32,
    pub width: f32,
}

#[derive(Serialize, Deserialize)]
pub struct TypingSpeed {
    pub slow: [f32; 2],
    pub normal: [f32; 2],
    pub fast: [f32; 2],
    pub very_fast: [f32; 2],
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub typing_speed: TypingSpeed,
    pub window_size: WindowSize,
}

#[cfg(any(not(debug_assertions), target_arch = "wasm32"))]
pub fn read_config() -> serde_json::Result<Config> {
    let config = include_str!("../assets/config.json");
    let config: Config = serde_json::from_str(config)?;
    return Ok(config);
}

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
pub fn read_config() -> std::io::Result<Config> {
    use std::fs::File;
    // Some JSON input data as a &str. Maybe this comes from the user.
    let f = File::open("assets/config.json")?;
    let p: Config = serde_json::from_reader(f)?;
    Ok(p)
}
