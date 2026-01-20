use rand::seq::{IteratorRandom, SliceRandom};
use ratatui::widgets::ListState;

use crate::player::{PlaybackState, Player};
use crate::scanner::Track;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl Default for RepeatMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortMode {
    Filename,
    Title,
    Artist,
}

impl Default for SortMode {
    fn default() -> Self {
        Self::Filename
    }
}

pub struct App {
    pub tracks: Vec<Track>,
    pub list_state: ListState,
    pub player: Player,
    pub playing_index: Option<usize>,
    pub running: bool,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub sort_mode: SortMode,
    pub show_help: bool,
    pub show_lyrics: bool,
    pub status_message: Option<(String, std::time::Instant)>,
    pub queue: Vec<usize>,
    pub queue_index: Option<usize>,
}

use crate::state::AppState;

impl App {
    pub fn new(tracks: Vec<Track>) -> Self {
        let state = AppState::load();

        let mut list_state = ListState::default();
        if !tracks.is_empty() {
            let initial_index = if let Some(path) = &state.last_track_path {
                tracks.iter().position(|t| &t.path == path).unwrap_or(0)
            } else {
                0
            };
            list_state.select(Some(initial_index));
        }

        let mut player = Player::new().expect("Failed to initialize audio player");
        player.set_volume(state.volume);

        let playing_index = if !tracks.is_empty() && state.last_track_path.is_some() {
            let idx = state
                .last_track_path
                .as_ref()
                .and_then(|path| tracks.iter().position(|t| &t.path == path));
            idx
        } else {
            None
        };

        let mut queue: Vec<usize> = (0..tracks.len()).collect();
        let mut queue_index = None;

        if !tracks.is_empty() {
            if let Some(path) = &state.last_track_path {
                let current_index = tracks.iter().position(|t| &t.path == path);

                if state.shuffle {
                    let mut rng = rand::thread_rng();
                    queue.shuffle(&mut rng);
                    if let Some(idx) = current_index {
                        queue_index = queue.iter().position(|&i| i == idx);
                    }
                } else {
                    queue_index = current_index;
                }
            }
        }

        Self {
            tracks,
            list_state,
            player,
            playing_index,
            running: true,
            repeat_mode: state.repeat_mode,
            shuffle: state.shuffle,
            sort_mode: state.sort_mode,
            show_help: false,
            show_lyrics: false,
            status_message: None,
            queue,
            queue_index,
        }
    }

    pub fn quit(&mut self) {
        self.player.stop();
        self.running = false;

        let last_track_path = self.playing_index.map(|i| self.tracks[i].path.clone());

        let state = AppState {
            volume: self.player.volume,
            shuffle: self.shuffle,
            repeat_mode: self.repeat_mode,
            sort_mode: self.sort_mode,
            last_track_path,
        };
        state.save();
    }

    pub fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn select_previous(&mut self) {
        let current = self.selected();
        if current > 0 {
            self.list_state.select(Some(current - 1));
        }
    }

    pub fn select_next(&mut self) {
        let current = self.selected();
        if !self.tracks.is_empty() && current < self.tracks.len() - 1 {
            self.list_state.select(Some(current + 1));
        }
    }

    pub fn play_selected(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let index = self.selected();
        let track = &self.tracks[index];

        match self.player.play(&track.path, &track.title) {
            Ok(_) => {
                self.playing_index = Some(index);
                if let Some(pos) = self.queue.iter().position(|&i| i == index) {
                    self.queue_index = Some(pos);
                }
            }
            Err(e) => {
                self.set_status(format!("Error: {}", e));
            }
        }
    }

    pub fn play_next(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let current_q_idx = self.queue_index.unwrap_or(0);
        let next_q_idx = current_q_idx + 1;

        let final_q_idx = if next_q_idx >= self.queue.len() {
            if self.repeat_mode == RepeatMode::All {
                0
            } else {
                return;
            }
        } else {
            next_q_idx
        };

        self.queue_index = Some(final_q_idx);
        let track_idx = self.queue[final_q_idx];

        self.list_state.select(Some(track_idx));
        self.play_selected();
    }

    pub fn toggle_pause(&mut self) {
        match self.player.state {
            PlaybackState::Playing | PlaybackState::Paused => {
                self.player.toggle_pause();
            }
            PlaybackState::Stopped => {
                if self.tracks.is_empty() {
                    return;
                }

                if self.shuffle {
                    let mut rng = rand::thread_rng();
                    let index = (0..self.tracks.len()).choose(&mut rng).unwrap_or(0);
                    self.list_state.select(Some(index));
                }

                self.play_selected();
            }
        }
    }

    pub fn stop(&mut self) {
        self.player.stop();
        self.playing_index = None;
    }

    pub fn change_volume(&mut self, increase: bool) {
        if increase {
            self.player.increase_volume();
        } else {
            self.player.decrease_volume();
        }
    }

    pub fn toggle_mute(&mut self) {
        self.player.toggle_mute();
    }

    pub fn seek_forward(&mut self) {
        self.seek_by(5);
    }

    pub fn seek_backward(&mut self) {
        self.seek_by(-5);
    }

    pub fn seek_by(&mut self, seconds: i64) {
        let current = self.player.position();
        if seconds > 0 {
            let new_pos = current + std::time::Duration::from_secs(seconds as u64);
            self.player.seek(new_pos);
        } else {
            let sub = seconds.abs() as u64;
            if current.as_secs() > sub {
                let new_pos = current - std::time::Duration::from_secs(sub);
                self.player.seek(new_pos);
            } else {
                self.player.seek(std::time::Duration::ZERO);
            }
        }
    }

    pub fn seek_percentage(&mut self, percent: u8) {
        if let Some(index) = self.playing_index {
            let duration = self.tracks[index].duration;
            let secs = (duration as f64 * (percent as f64 / 100.0)) as u64;
            self.player.seek(std::time::Duration::from_secs(secs));
        }
    }

    pub fn play_previous_track(&mut self) {
        let position = self.player.position();
        if position.as_secs() > 3 {
            self.player.seek(std::time::Duration::ZERO);
        } else {
            let current_q_idx = self.queue_index.unwrap_or(0);
            let prev_q_idx = if current_q_idx > 0 {
                current_q_idx - 1
            } else {
                self.queue.len() - 1
            };

            self.queue_index = Some(prev_q_idx);
            let track_idx = self.queue[prev_q_idx];
            self.list_state.select(Some(track_idx));
            self.play_selected();
        }
    }

    pub fn play_next_track(&mut self) {
        self.play_next();
    }

    pub fn check_playback(&mut self) {
        if self.player.is_finished() {
            let current_index = self.playing_index.unwrap_or(0);
            let is_last_track = current_index + 1 >= self.tracks.len();

            if !self.shuffle && self.repeat_mode == RepeatMode::Off && is_last_track {
                self.playing_index = None;
                self.player.state = PlaybackState::Stopped;
            } else {
                if self.repeat_mode == RepeatMode::One {
                    self.player.seek(std::time::Duration::ZERO);
                    self.player.state = PlaybackState::Playing;
                    return;
                }
                self.play_next();
            }
        }
    }

    pub fn check_repeat_mode(&mut self) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;

        if self.shuffle {
            let mut rng = rand::thread_rng();
            self.queue.shuffle(&mut rng);

            if let Some(current_idx) = self.playing_index {
                if let Some(pos) = self.queue.iter().position(|&i| i == current_idx) {
                    self.queue_index = Some(pos);
                }
            }
        } else {
            self.queue = (0..self.tracks.len()).collect();
            self.queue_index = self.playing_index;
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.show_lyrics = false;
        }
    }

    pub fn toggle_lyrics(&mut self) {
        self.show_lyrics = !self.show_lyrics;
        if self.show_lyrics {
            self.show_help = false;
        }
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Filename => SortMode::Title,
            SortMode::Title => SortMode::Artist,
            SortMode::Artist => SortMode::Filename,
        };
        self.sort_tracks();
    }

    pub fn sort_tracks(&mut self) {
        let current_track_path = self.playing_index.map(|i| self.tracks[i].path.clone());

        match self.sort_mode {
            SortMode::Filename => self.tracks.sort_by(|a, b| a.path.cmp(&b.path)),
            SortMode::Title => self
                .tracks
                .sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
            SortMode::Artist => self
                .tracks
                .sort_by(|a, b| a.artist.to_lowercase().cmp(&b.artist.to_lowercase())),
        }

        if let Some(path) = current_track_path {
            self.playing_index = self.tracks.iter().position(|t| t.path == path);
        }

        if self.shuffle {
            let mut rng = rand::thread_rng();
            let mut new_queue: Vec<usize> = (0..self.tracks.len()).collect();
            new_queue.shuffle(&mut rng);
            self.queue = new_queue;

            if let Some(current_idx) = self.playing_index {
                if let Some(pos) = self.queue.iter().position(|&i| i == current_idx) {
                    self.queue_index = Some(pos);
                }
            }
        } else {
            self.queue = (0..self.tracks.len()).collect();
            self.queue_index = self.playing_index;
        }

        if self.list_state.selected().is_none() && !self.tracks.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some((message, std::time::Instant::now()));
    }

    pub fn check_status_message(&mut self) {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed().as_secs() > 3 {
                self.status_message = None;
            }
        }
    }
}
