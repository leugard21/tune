use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::player::PlaybackState;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_playlist(frame, app, chunks[0]);
    render_now_playing(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    if app.show_help {
        let area = centered_rect(60, 60, frame.area());
        render_help_overlay(frame, area);
    }

    if let Some((msg, _)) = &app.status_message {
        let area = centered_rect(40, 10, frame.area());
        render_status_overlay(frame, msg, area);
    }
}

fn render_playlist(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let selected = app.selected();
    let playing_index = app.playing_index;

    let items: Vec<ListItem> = app
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let is_playing = playing_index == Some(index);
            let is_selected = index == selected;

            let style = match (is_selected, is_playing) {
                (true, true) => Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                (true, false) => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                (false, true) => Style::default().fg(Color::Green),
                (false, false) => Style::default().fg(Color::White),
            };

            let prefix = if is_playing { "[>] " } else { "    " };
            let content = format!("{}{}", prefix, track.display_name());

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Playlist ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_now_playing(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)])
        .margin(1)
        .split(area);

    let state = app.player.state;
    let position = app.player.position();

    let (state_label, state_color) = match state {
        PlaybackState::Playing => ("Playing", Color::Green),
        PlaybackState::Paused => ("Paused", Color::Yellow),
        PlaybackState::Stopped => ("Stopped", Color::Gray),
    };

    let track_name = app
        .player
        .current_track
        .as_deref()
        .unwrap_or("No track selected");

    let info_text = format!("[{}] {}", state_label, track_name);

    let info = Paragraph::new(info_text).style(Style::default().fg(state_color));

    let elapsed_secs = position.as_secs();
    let elapsed_mins = elapsed_secs / 60;
    let elapsed_secs = elapsed_secs % 60;

    let total_duration_secs = if let Some(index) = app.playing_index {
        app.tracks[index].duration
    } else {
        0
    };
    let total_mins = total_duration_secs / 60;
    let total_secs = total_duration_secs % 60;

    let vol_percent = (app.player.volume * 100.0) as u8;

    let stats_text = format!(
        "{:02}:{:02} / {:02}:{:02} | Vol: {}%",
        elapsed_mins, elapsed_secs, total_mins, total_secs, vol_percent
    );

    let stats_display = Paragraph::new(stats_text).style(Style::default().fg(Color::Cyan));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Now Playing ")
        .border_style(Style::default().fg(Color::Magenta));

    frame.render_widget(block, area);
    frame.render_widget(info, chunks[0]);
    frame.render_widget(stats_display, chunks[1]);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let track_count = app.tracks.len();
    let repeat_str = match app.repeat_mode {
        crate::app::RepeatMode::Off => "",
        crate::app::RepeatMode::All => "[Repeat: All] ",
        crate::app::RepeatMode::One => "[Repeat: One] ",
    };

    let shuffle_str = if app.shuffle { "[Shuffle: ON] " } else { "" };
    let sort_str = match app.sort_mode {
        crate::app::SortMode::Filename => "[Sort: File] ",
        crate::app::SortMode::Title => "[Sort: Title] ",
        crate::app::SortMode::Artist => "[Sort: Artist] ",
    };

    let status_text = if track_count == 0 {
        String::from("No tracks found")
    } else {
        format!(
            "{}{}{}Track {}/{} | [h] Help | [q] Quit",
            sort_str,
            shuffle_str,
            repeat_str,
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

fn render_help_overlay(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help_text = vec![
        "      Controls",
        "--------------------",
        " k / Up    : Selection Up",
        " j / Down  : Selection Down",
        " Enter     : Play Track",
        " Space     : Pause / Resume",
        " s         : Stop",
        " + / =     : Volume Up",
        " -         : Volume Down",
        " Left      : Seek Back 5s",
        " Right     : Seek Forward 5s",
        " z         : Toggle Shuffle",
        " r         : Cycle Repeat",
        " o         : Cycle Sort",
        " h / Esc   : Close Help",
        " q         : Quit",
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    let paragraph = Paragraph::new(help_text.join("\n"))
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

fn render_status_overlay(frame: &mut Frame, msg: &str, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Message ")
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(msg)
        .block(block)
        .style(Style::default().fg(Color::Yellow))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
