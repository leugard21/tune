// User interface rendering module

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph},
};

use crate::app::App;

/// Render the application UI
pub fn render(frame: &mut Frame, app: &mut App) {
    // Create the main layout with playlist and status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Playlist area (takes remaining space)
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    // Render the track list
    render_playlist(frame, app, chunks[0]);

    // Render the status bar
    render_status_bar(frame, app, chunks[1]);
}

/// Render the playlist panel with stateful scrolling
fn render_playlist(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let selected = app.selected();

    // Create list items from tracks
    let items: Vec<ListItem> = app
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            // Style for selected vs unselected items
            let style = if index == selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(track.name.clone(), style)))
        })
        .collect();

    // Create the list widget with highlight configuration
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Playlist ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    // Use stateful rendering to enable automatic scrolling
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

/// Render the status bar at the bottom
fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Build status text
    let track_count = app.tracks.len();
    let status_text = if track_count == 0 {
        String::from("No tracks found")
    } else {
        format!(
            "Track {} of {} | [j/k] Navigate | [q] Quit",
            app.selected() + 1,
            track_count
        )
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Status ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(status, area);
}
