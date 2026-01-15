use rand::seq::SliceRandom;
use ratatui::widgets::ListState;

use crate::player::{PlaybackState, Player};
use crate::scanner::Track;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

pub struct App {
    pub tracks: Vec<Track>,
    pub list_state: ListState,
    pub player: Player,
    pub playing_index: Option<usize>,
    pub running: bool,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub search_query: String,
    pub is_searching: bool,
}

impl App {
    pub fn new(tracks: Vec<Track>) -> Self {
        let mut list_state = ListState::default();
        if !tracks.is_empty() {
            list_state.select(Some(0));
        }

        let player = Player::new().expect("Failed to initialize audio player");

        Self {
            tracks,
            list_state,
            player,
            playing_index: None,
            running: true,
            repeat_mode: RepeatMode::Off,
            shuffle: false,
            search_query: String::new(),
            is_searching: false,
        }
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

        if self.player.play(&track.path, &track.display_name()).is_ok() {
            self.playing_index = Some(index);
        }
    }

    pub fn play_next(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let next_index = if self.repeat_mode == RepeatMode::One {
            self.playing_index.unwrap_or(0)
        } else if self.shuffle {
            let mut rng = rand::thread_rng();
            let current = self.playing_index.unwrap_or(0);
            let mut indices: Vec<usize> = (0..self.tracks.len()).collect();
            if indices.len() > 1 {
                indices.retain(|&x| x != current);
            }
            *indices.choose(&mut rng).unwrap_or(&0)
        } else {
            let current_index = self.playing_index.unwrap_or(0);
            if current_index + 1 >= self.tracks.len() {
                if self.repeat_mode == RepeatMode::All {
                    0
                } else {
                    return;
                }
            } else {
                current_index + 1
            }
        };

        self.list_state.select(Some(next_index));
        self.play_selected();
    }

    pub fn toggle_pause(&mut self) {
        self.player.toggle_pause();
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

    pub fn seek_forward(&mut self) {
        let current = self.player.position();
        let new_pos = current + std::time::Duration::from_secs(5);
        self.player.seek(new_pos);
    }

    pub fn seek_backward(&mut self) {
        let current = self.player.position();
        if current.as_secs() > 5 {
            let new_pos = current - std::time::Duration::from_secs(5);
            self.player.seek(new_pos);
        } else {
            self.player.seek(std::time::Duration::ZERO);
        }
    }

    pub fn check_playback(&mut self) {
        if self.player.is_finished() {
            let current_index = self.playing_index.unwrap_or(0);
            let is_last_track = current_index + 1 >= self.tracks.len();

            if !self.shuffle && self.repeat_mode == RepeatMode::Off && is_last_track {
                self.playing_index = None;
                self.player.state = PlaybackState::Stopped;
            } else {
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
    }

    pub fn quit(&mut self) {
        self.player.stop();
        self.running = false;
    }
}
