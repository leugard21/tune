use std::path::PathBuf;
use walkdir::WalkDir;

use crate::config::{Config, SUPPORTED_EXTENSIONS};

pub struct Track {
    pub path: PathBuf,
    pub name: String,
}

impl Track {
    pub fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self { path, name }
    }
}

pub fn scan_music_directory(config: &Config) -> Vec<Track> {
    let mut tracks = Vec::new();

    for entry in WalkDir::new(&config.music_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                tracks.push(Track::from_path(path.to_path_buf()));
            }
        }
    }

    tracks.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    tracks
}
