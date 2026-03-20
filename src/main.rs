//! IOTA Wallet TUI — a terminal interface for the IOTA network.

mod app;
mod event;
mod ui;
mod wallet;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use wallet::{WalletBackend, WalletCmd};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal).await;

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

async fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> color_eyre::Result<()> {
    // Create channels
    let (cmd_tx, cmd_rx) = mpsc::channel::<WalletCmd>(32);
    let (event_tx, mut event_rx) = mpsc::channel::<wallet::WalletEvent>(32);

    // Spawn wallet backend
    let backend = WalletBackend::new(cmd_rx, event_tx);
    let initial_keys = backend.stored_keys().to_vec();
    tokio::spawn(backend.run());

    let mut app = app::App::new(cmd_tx.clone(), initial_keys);

    // Connect to last used network (defaults to testnet)
    let saved_network = wallet::load_network();
    let _ = cmd_tx.send(WalletCmd::Connect(saved_network)).await;

    let mut event_stream = EventStream::new();
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(100));

    loop {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        tokio::select! {
            // Terminal events (keyboard/mouse)
            maybe_event = event_stream.next() => {
                if let Some(Ok(ev)) = maybe_event {
                    event::handle_event(&mut app, ev);
                }
            }
            // Wallet backend responses
            maybe_wallet = event_rx.recv() => {
                if let Some(wallet_event) = maybe_wallet {
                    app.handle_wallet_event(wallet_event);
                }
            }
            // Tick for periodic redraws
            _ = tick_interval.tick() => {
                if app.color_phase > 0 {
                    app.color_phase = app.color_phase.wrapping_add(1);
                }
                // Auto-dismiss toast after 2 seconds
                if let Some((_, ref instant)) = app.clipboard_toast
                    && instant.elapsed() >= std::time::Duration::from_secs(2)
                {
                    app.clipboard_toast = None;
                }
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
