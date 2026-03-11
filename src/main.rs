mod app;
mod event;
mod ui;

use std::time::Duration;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> color_eyre::Result<()> {
    let mut app = app::App::new();

    loop {
        app.clear_expired_status();

        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        if let Some(ev) = event::poll_event(Duration::from_millis(100))? {
            event::handle_event(&mut app, ev);
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
