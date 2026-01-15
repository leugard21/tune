use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::App;

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                handle_key(app, key.code);
            }
        }
        return Ok(true);
    }
    Ok(false)
}

fn handle_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char(' ') => app.toggle_pause(),
        KeyCode::Char('s') => app.stop(),

        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => app.play_selected(),

        KeyCode::Char('+') | KeyCode::Char('=') => app.change_volume(true),
        KeyCode::Char('-') => app.change_volume(false),

        KeyCode::Right => app.seek_forward(),
        KeyCode::Left => app.seek_backward(),

        KeyCode::Char('r') => app.check_repeat_mode(),
        KeyCode::Char('z') => app.toggle_shuffle(),

        _ => {}
    }
}
