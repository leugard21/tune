use std::path::PathBuf;

pub const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg"];

pub struct Config {
    pub music_dir: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let music_dir = dirs::audio_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Music")))
            .unwrap_or_else(|| PathBuf::from("."));

        Self { music_dir }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
