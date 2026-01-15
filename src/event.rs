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
    if app.show_help {
        match code {
            KeyCode::Char('h') | KeyCode::Esc => {
                app.toggle_help();
                return;
            }
            _ => {}
        }
    }

    match code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('h') => app.toggle_help(),
        KeyCode::Char('o') => app.cycle_sort_mode(),

        KeyCode::Char(' ') => app.toggle_pause(),
        KeyCode::Char('s') => app.stop(),

        KeyCode::Down | KeyCode::Char('j') => {
            if !app.show_help {
                app.select_next()
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.show_help {
                app.select_previous()
            }
        }
        KeyCode::Enter => {
            if !app.show_help {
                app.play_selected()
            }
        }

        KeyCode::Char('+') | KeyCode::Char('=') => app.change_volume(true),
        KeyCode::Char('-') => app.change_volume(false),

        KeyCode::Right => app.seek_forward(),
        KeyCode::Left => app.seek_backward(),

        KeyCode::Char('r') => app.check_repeat_mode(),
        KeyCode::Char('z') => app.toggle_shuffle(),

        _ => {}
    }
}
