use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, HighlightSpacing, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::player::PlaybackState;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(7),
            Constraint::Length(3),
        ])
        .split(frame.area());

    if app.show_lyrics {
        render_lyrics(frame, app, chunks[0]);
    } else {
        render_playlist(frame, app, chunks[0]);
    }
    render_now_playing(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    if app.show_help {
        let area = centered_rect(70, 70, frame.area());
        render_help_overlay(frame, area);
    }

    if let Some((msg, _)) = &app.status_message {
        let area = centered_rect(50, 15, frame.area());
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
                    .fg(Color::Rgb(100, 255, 100))
                    .add_modifier(Modifier::BOLD),
                (true, false) => Style::default()
                    .fg(Color::Rgb(255, 200, 100))
                    .add_modifier(Modifier::BOLD),
                (false, true) => Style::default().fg(Color::Rgb(150, 255, 150)),
                (false, false) => Style::default().fg(Color::Rgb(200, 200, 200)),
            };

            let prefix = if is_playing { "▶ " } else { "  " };
            let content = format!("{}{}", prefix, track.display_name());

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Playlist ")
                .border_style(Style::default().fg(Color::Rgb(100, 150, 255)))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .highlight_symbol("▸ ")
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn parse_lrc(lrc: &str) -> Vec<(std::time::Duration, String)> {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }

        let parts: Vec<&str> = line.split(']').collect();
        if parts.len() < 2 {
            continue;
        }

        let time_part = parts[0].trim_start_matches('[');
        let text = parts[1..].join("]").trim().to_string();

        let time_split: Vec<&str> = time_part.split(':').collect();
        if time_split.len() == 2 {
            let mins: u64 = time_split[0].parse().unwrap_or(0);
            let secs_split: Vec<&str> = time_split[1].split('.').collect();
            let secs: u64 = secs_split[0].parse().unwrap_or(0);
            let millis: u64 = if secs_split.len() > 1 {
                let ms_str = secs_split[1];
                let ms_val: u64 = ms_str.parse().unwrap_or(0);
                match ms_str.len() {
                    1 => ms_val * 100,
                    2 => ms_val * 10,
                    _ => ms_val,
                }
            } else {
                0
            };
            let duration =
                std::time::Duration::from_millis(mins * 60 * 1000 + secs * 1000 + millis);
            lines.push((duration, text));
        }
    }
    lines
}

fn render_lyrics(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Lyrics ")
        .border_style(Style::default().fg(Color::Rgb(255, 150, 100)))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let (track, raw_lyrics) = if let Some(index) = app.playing_index {
        let t = &app.tracks[index];
        (Some(t), t.lyrics.as_deref())
    } else {
        (None, None)
    };

    if track.is_none() {
        let paragraph = Paragraph::new("No track playing.")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let raw_lyrics = if let Some(l) = raw_lyrics {
        l
    } else {
        let paragraph = Paragraph::new("No lyrics found for this track.")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    };

    let parsed_lyrics = parse_lrc(raw_lyrics);

    if parsed_lyrics.is_empty() {
        let paragraph = Paragraph::new(raw_lyrics)
            .block(block)
            .style(Style::default().fg(Color::Rgb(240, 240, 240)))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    }

    let current_time = app.player.position();

    // Find active line index
    let active_idx = parsed_lyrics
        .iter()
        .position(|(time, _)| *time > current_time)
        .map(|idx| idx.saturating_sub(1))
        .unwrap_or(parsed_lyrics.len().saturating_sub(1));

    // Calculate viewable area
    let inner_area = block.inner(area);
    let height = inner_area.height as usize;
    let mid = height / 2;

    let start_idx = active_idx.saturating_sub(mid);
    let end_idx = (start_idx + height).min(parsed_lyrics.len());

    // Adjust start_idx if we're near the end to keep the view full
    let start_idx = if end_idx == parsed_lyrics.len() {
        end_idx.saturating_sub(height)
    } else {
        start_idx
    };

    let mut lines = Vec::new();
    for i in start_idx..end_idx {
        let (_, text) = &parsed_lyrics[i];
        let style = if i == active_idx {
            Style::default()
                .fg(Color::Rgb(255, 255, 100))
                .add_modifier(Modifier::BOLD)
        } else {
            // Distance-based dimming for "animation" feel
            let dist = (i as i32 - active_idx as i32).abs();
            let intensity = match dist {
                0 => 255,
                1 => 200,
                2 => 150,
                3 => 100,
                _ => 60,
            };
            Style::default().fg(Color::Rgb(intensity, intensity, intensity))
        };
        lines.push(Line::from(Span::styled(text, style)));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_now_playing(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(area);

    let state = app.player.state;
    let position = app.player.position();

    let (state_label, state_color, state_icon) = match state {
        PlaybackState::Playing => ("Playing", Color::Rgb(100, 255, 100), "▶"),
        PlaybackState::Paused => ("Paused", Color::Rgb(255, 200, 100), "⏸"),
        PlaybackState::Stopped => ("Stopped", Color::Rgb(150, 150, 150), "⏹"),
    };

    let track_name = app
        .player
        .current_track
        .as_deref()
        .unwrap_or("No track selected");

    let next_track_info = if let Some(q_idx) = app.queue_index {
        if q_idx + 1 < app.queue.len() {
            let next_idx = app.queue[q_idx + 1];
            format!("Next: {}", app.tracks[next_idx].title)
        } else if app.repeat_mode == crate::app::RepeatMode::All && !app.queue.is_empty() {
            let next_idx = app.queue[0];
            format!("Next: {}", app.tracks[next_idx].title)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let info_text = if next_track_info.is_empty() {
        format!("{} {} {}", state_icon, state_label, track_name)
    } else {
        format!(
            "{} {} {}\n{}",
            state_icon, state_label, track_name, next_track_info
        )
    };

    let info = Paragraph::new(info_text)
        .style(
            Style::default()
                .fg(state_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center);

    let elapsed_secs = position.as_secs();
    let total_duration_secs = if let Some(index) = app.playing_index {
        app.tracks[index].duration
    } else {
        0
    };

    let progress_ratio = if total_duration_secs > 0 {
        elapsed_secs as f64 / total_duration_secs as f64
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Rgb(100, 200, 255))
                .bg(Color::Rgb(40, 40, 40)),
        )
        .ratio(progress_ratio);

    let elapsed_mins = elapsed_secs / 60;
    let elapsed_sec = elapsed_secs % 60;
    let total_mins = total_duration_secs / 60;
    let total_sec = total_duration_secs % 60;

    let time_text = format!(
        "{:02}:{:02} / {:02}:{:02}",
        elapsed_mins, elapsed_sec, total_mins, total_sec
    );
    let time_display = Paragraph::new(time_text)
        .style(Style::default().fg(Color::Rgb(150, 150, 150)))
        .alignment(ratatui::layout::Alignment::Center);

    let vol_percent = (app.player.volume * 100.0) as u8;
    let vol_text = if app.player.muted {
        "Volume: Muted".to_string()
    } else {
        format!("Volume: {}%", vol_percent)
    };

    let vol_style = if app.player.muted {
        Style::default().fg(Color::Rgb(255, 100, 100))
    } else {
        Style::default().fg(Color::Rgb(150, 200, 255))
    };

    let vol_display = Paragraph::new(vol_text)
        .style(vol_style)
        .alignment(ratatui::layout::Alignment::Center);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Now Playing ")
        .border_style(Style::default().fg(Color::Rgb(255, 100, 200)))
        .border_type(ratatui::widgets::BorderType::Rounded);

    frame.render_widget(block, area);
    frame.render_widget(info, chunks[0]);
    frame.render_widget(gauge, chunks[1]);
    frame.render_widget(time_display, chunks[2]);
    frame.render_widget(vol_display, chunks[3]);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let track_count = app.tracks.len();
    let repeat_str = match app.repeat_mode {
        crate::app::RepeatMode::Off => "",
        crate::app::RepeatMode::All => "[Repeat: All] ",
        crate::app::RepeatMode::One => "[Repeat: One] ",
    };

    let shuffle_str = if app.shuffle { "[Shuffle] " } else { "" };
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
        .style(Style::default().fg(Color::Rgb(180, 180, 180)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Status ")
                .border_style(Style::default().fg(Color::Rgb(100, 100, 150)))
                .border_type(ratatui::widgets::BorderType::Rounded),
        );

    frame.render_widget(status, area);
}

fn render_help_overlay(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "      Controls",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Rgb(100, 200, 255)),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Navigation",
            Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .fg(Color::Rgb(150, 255, 150)),
        )]),
        Line::from(vec![
            Span::styled(
                " k / ↑      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Move selection up"),
        ]),
        Line::from(vec![
            Span::styled(
                " j / ↓      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Move selection down"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Playback",
            Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .fg(Color::Rgb(150, 255, 150)),
        )]),
        Line::from(vec![
            Span::styled(
                " Enter      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Play selected track"),
        ]),
        Line::from(vec![
            Span::styled(
                " Space      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Pause / Resume"),
        ]),
        Line::from(vec![
            Span::styled(
                " s          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Stop playback"),
        ]),
        Line::from(vec![
            Span::styled(
                " ← / →      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Seek 5s | Shift+←/→ Seek 10s"),
        ]),
        Line::from(vec![
            Span::styled(
                " 0 - 9      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Jump to 0% - 90%"),
        ]),
        Line::from(vec![
            Span::styled(
                " [ / ]      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Previous / Next Track"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Settings",
            Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .fg(Color::Rgb(150, 255, 150)),
        )]),
        Line::from(vec![
            Span::styled(
                " + / =      ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Increase volume"),
        ]),
        Line::from(vec![
            Span::styled(
                " -          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Decrease volume"),
        ]),
        Line::from(vec![
            Span::styled(
                " m          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Toggle mute"),
        ]),
        Line::from(vec![
            Span::styled(
                " z          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Toggle shuffle"),
        ]),
        Line::from(vec![
            Span::styled(
                " r          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Cycle repeat mode"),
        ]),
        Line::from(vec![
            Span::styled(
                " o          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Cycle sort mode"),
        ]),
        Line::from(vec![
            Span::styled(
                " l          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Toggle lyrics"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " h / Esc    ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Close help"),
        ]),
        Line::from(vec![
            Span::styled(
                " q          ",
                Style::default().fg(Color::Rgb(255, 200, 100)),
            ),
            Span::raw("Quit application"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .style(Style::default().bg(Color::Rgb(20, 20, 40)))
        .border_style(Style::default().fg(Color::Rgb(100, 150, 255)))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().fg(Color::Rgb(220, 220, 220)));

    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

fn render_status_overlay(frame: &mut Frame, msg: &str, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Message ")
        .border_style(Style::default().fg(Color::Rgb(255, 200, 100)))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(40, 30, 20)));

    let paragraph = Paragraph::new(msg)
        .block(block)
        .style(Style::default().fg(Color::Rgb(255, 220, 150)))
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
