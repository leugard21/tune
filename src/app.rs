// Application state module

use ratatui::widgets::ListState;

use crate::scanner::Track;

/// Main application state
pub struct App {
    /// List of discovered music tracks
    pub tracks: Vec<Track>,
    /// List state for tracking selection and scroll position
    pub list_state: ListState,
    /// Whether the application should continue running
    pub running: bool,
}

impl App {
    /// Create a new application with the given tracks
    pub fn new(tracks: Vec<Track>) -> Self {
        let mut list_state = ListState::default();
        // Select the first item if tracks exist
        if !tracks.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            tracks,
            list_state,
            running: true,
        }
    }

    /// Get the currently selected index
    pub fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    /// Move selection up by one item
    pub fn select_previous(&mut self) {
        let current = self.selected();
        if current > 0 {
            self.list_state.select(Some(current - 1));
        }
    }

    /// Move selection down by one item
    pub fn select_next(&mut self) {
        let current = self.selected();
        if !self.tracks.is_empty() && current < self.tracks.len() - 1 {
            self.list_state.select(Some(current + 1));
        }
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
