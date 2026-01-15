// TUI Music Player - Main entry point

mod app;
mod config;
mod event;
mod scanner;
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
    // Load configuration
    let config = Config::new();

    // Scan for music files
    let tracks = scan_music_directory(&config);

    // Create application state
    let mut app = App::new(tracks);

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main application loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Propagate any errors from the main loop
    result
}

/// Run the main application loop
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    while app.running {
        // Draw the UI with mutable access to app for stateful widgets
        terminal.draw(|frame| ui::render(frame, app))?;

        // Handle events
        event::handle_events(app)?;
    }

    Ok(())
}
