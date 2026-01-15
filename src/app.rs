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

    pub fn toggle_pause(&mut self) {
        self.player.toggle_pause();
    }

    pub fn stop(&mut self) {
        self.player.stop();
        self.playing_index = None;
    }

    pub fn check_playback(&mut self) {
        if self.player.is_finished() {
            self.playing_index = None;
            self.player.state = PlaybackState::Stopped;
        }
    }

    pub fn quit(&mut self) {
        self.player.stop();
        self.running = false;
    }
}
