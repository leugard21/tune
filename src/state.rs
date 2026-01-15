use crate::app::{RepeatMode, SortMode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub volume: f32,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
    pub sort_mode: SortMode,
    pub last_track_path: Option<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            volume: 1.0,
            shuffle: false,
            repeat_mode: RepeatMode::Off,
            sort_mode: SortMode::Filename,
            last_track_path: None,
        }
    }
}

impl AppState {
    pub fn load() -> Self {
        if let Some(mut path) = dirs::data_dir() {
            path.push("tune");
            fs::create_dir_all(&path).ok();
            path.push("state.json");

            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(mut path) = dirs::data_dir() {
            path.push("tune");
            fs::create_dir_all(&path).ok();
            path.push("state.json");

            if let Ok(content) = serde_json::to_string_pretty(self) {
                fs::write(path, content).ok();
            }
        }
    }
}
