use ratatui::widgets::ListState;

use crate::player::{PlaybackState, Player};
use crate::scanner::Track;

pub struct App {
    pub tracks: Vec<Track>,
    pub list_state: ListState,
    pub player: Player,
    pub playing_index: Option<usize>,
    pub running: bool,
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

        if self.player.play(&track.path, &track.name).is_ok() {
            self.playing_index = Some(index);
        }
    }

    pub fn play_next(&mut self) {
        let current_index = self.playing_index.unwrap_or(0);
        let next_index = (current_index + 1) % self.tracks.len();

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
            if !self.tracks.is_empty() {
                self.play_next();
            } else {
                self.playing_index = None;
                self.player.state = PlaybackState::Stopped;
            }
        }
    }

    pub fn quit(&mut self) {
        self.player.stop();
        self.running = false;
    }
}
