use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::config::{Config, SUPPORTED_EXTENSIONS};

#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub duration: u64,
    pub lyrics: Option<String>,
}

impl Track {
    pub fn from_path(path: PathBuf) -> Self {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut title = filename.clone();
        let mut artist = String::from("Unknown Artist");
        let mut duration = 0;
        let mut lyrics = None;

        if let Ok(tagged_file) = Probe::open(&path).and_then(|p| p.read()) {
            duration = tagged_file.properties().duration().as_secs();

            if let Some(tag) = tagged_file.primary_tag() {
                if let Some(t) = tag.title() {
                    title = t.to_string();
                }
                if let Some(a) = tag.artist() {
                    artist = a.to_string();
                }
                lyrics = tag.get_string(&ItemKey::Lyrics).map(|s| s.to_string());
            }
        }

        Self {
            path: path.to_path_buf(),
            title,
            artist,
            duration,
            lyrics,
        }
    }

    pub fn display_name(&self) -> String {
        if self.artist != "Unknown Artist" {
            format!("{} - {}", self.artist, self.title)
        } else {
            self.title.clone()
        }
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

    tracks.sort_by(|a, b| {
        a.artist
            .to_lowercase()
            .cmp(&b.artist.to_lowercase())
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });

    tracks
}
