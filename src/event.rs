use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::App;

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(33))? {
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

    if app.show_lyrics {
        match code {
            KeyCode::Char('l') | KeyCode::Esc => {
                app.toggle_lyrics();
                return;
            }
            _ => {}
        }
    }

    match code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('h') => app.toggle_help(),
        KeyCode::Char('l') => app.toggle_lyrics(),
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

        KeyCode::Char('m') => app.toggle_mute(),

        KeyCode::Char('[') => app.play_previous_track(),
        KeyCode::Char(']') => app.play_next_track(),

        KeyCode::Char('J') => app.seek_by(-10),
        KeyCode::Char('K') => app.seek_by(10),

        KeyCode::Char('0') => app.seek_percentage(0),
        KeyCode::Char('1') => app.seek_percentage(10),
        KeyCode::Char('2') => app.seek_percentage(20),
        KeyCode::Char('3') => app.seek_percentage(30),
        KeyCode::Char('4') => app.seek_percentage(40),
        KeyCode::Char('5') => app.seek_percentage(50),
        KeyCode::Char('6') => app.seek_percentage(60),
        KeyCode::Char('7') => app.seek_percentage(70),
        KeyCode::Char('8') => app.seek_percentage(80),
        KeyCode::Char('9') => app.seek_percentage(90),

        _ => {}
    }
}
