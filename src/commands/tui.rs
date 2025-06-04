use crate::Result;
use crate::tui::{app::TuiApp, events, ui};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

pub fn execute() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and event handler
    let mut app = TuiApp::new();
    let events = events::start_event_handler();

    // Main loop
    let result = loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle events
        match events.recv() {
            Ok(events::AppEvent::Key(key)) => {
                if let Err(e) = app.handle_key(key) {
                    break Err(e);
                }
                if app.should_quit {
                    break Ok(());
                }
            }
            Ok(events::AppEvent::Tick) => {}
            Err(e) => {
                break Err(crate::DhdError::Io(io::Error::other(e.to_string())));
            }
        }
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
