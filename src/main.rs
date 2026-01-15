mod app;
mod config;
mod event;
mod player;
mod scanner;
mod state;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;
use config::Config;
use scanner::scan_music_directory;

fn main() -> io::Result<()> {
    let config = Config::new();

    let tracks = scan_music_directory(&config);

    let mut app = App::new(tracks);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(frame, app))?;

        event::handle_events(app)?;

        app.check_playback();
    }

    Ok(())
}
