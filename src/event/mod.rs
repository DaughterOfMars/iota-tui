//! Event handling — dispatches keyboard and mouse events to the appropriate handler.

mod explorer;
mod input;
mod mouse;
pub(crate) mod nav;
mod popup;
mod screen;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, InputMode, Popup, Screen};
use crate::wallet::WalletCmd;

pub fn handle_event(app: &mut App, ev: Event) {
    match ev {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => mouse::handle_mouse(app, mouse),
        Event::Resize(_, _) => {}
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.running = false;
        return;
    }

    // Toggle sidebar collapsed/expanded
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b') {
        app.sidebar_collapsed = !app.sidebar_collapsed;
        crate::wallet::save_sidebar_collapsed(app.sidebar_collapsed);
        return;
    }

    {
        const SEQ: [u8; 10] = [38, 38, 40, 40, 37, 39, 37, 39, 98, 97];
        let code: Option<u8> = match key.code {
            KeyCode::Up => Some(38),
            KeyCode::Down => Some(40),
            KeyCode::Left => Some(37),
            KeyCode::Right => Some(39),
            KeyCode::Char(c) => Some(c as u8),
            _ => None,
        };
        if let Some(c) = code {
            if c == SEQ[app.nav_idx] {
                app.nav_idx += 1;
                if app.nav_idx == SEQ.len() {
                    app.nav_idx = 0;
                    app.color_phase = if app.color_phase > 0 { 0 } else { 1 };
                    crate::wallet::save_theme(app.color_phase > 0);
                }
            } else {
                app.nav_idx = if c == SEQ[0] { 1 } else { 0 };
            }
        }
    }

    if app.popup.is_some() {
        popup::handle_popup_key(app, key);
        return;
    }

    if app.input_mode == InputMode::Editing {
        // Screens that handle their own editing (Enter/Esc to submit/cancel)
        match app.screen {
            Screen::Explorer => {
                explorer::handle_explorer_key(app, key);
                return;
            }
            Screen::TxBuilder => {
                screen::handle_tx_key(app, key);
                return;
            }
            _ => {
                input::handle_input_key(app, key);
                return;
            }
        }
    }

    match key.code {
        KeyCode::Char('q') => {
            app.open_popup(Popup::ConfirmQuit);
            return;
        }
        KeyCode::Char('?') => {
            app.open_popup(Popup::Help);
            return;
        }
        KeyCode::Char('r') => {
            if app.screen == Screen::Explorer {
                app.refresh_explorer();
            } else {
                app.request_refresh();
            }
            return;
        }
        KeyCode::Char('n') => {
            app.open_popup(Popup::SwitchNetwork);
            return;
        }
        KeyCode::Char('.') => {
            app.action_menu_selected = 0;
            app.open_popup(Popup::ActionsMenu);
            return;
        }
        KeyCode::Char('E') => {
            app.load_error_log();
            app.open_popup(Popup::ErrorLog);
            return;
        }
        KeyCode::Char('1') => {
            app.navigate(Screen::Coins);
            return;
        }
        KeyCode::Char('2') => {
            app.navigate(Screen::Objects);
            return;
        }
        KeyCode::Char('3') => {
            app.navigate(Screen::Transactions);
            return;
        }
        KeyCode::Char('4') => {
            app.navigate(Screen::Packages);
            return;
        }
        KeyCode::Char('5') => {
            app.navigate(Screen::AddressBook);
            return;
        }
        KeyCode::Char('6') => {
            app.navigate(Screen::Keys);
            return;
        }
        KeyCode::Char('7') => {
            app.navigate(Screen::TxBuilder);
            return;
        }
        KeyCode::Char('8') => {
            app.navigate(Screen::Explorer);
            return;
        }
        KeyCode::Tab => {
            let idx = app.screen.index();
            let next = (idx + 1) % Screen::ALL.len();
            app.navigate(Screen::ALL[next]);
            return;
        }
        KeyCode::BackTab => {
            let idx = app.screen.index();
            let next = if idx == 0 {
                Screen::ALL.len() - 1
            } else {
                idx - 1
            };
            app.navigate(Screen::ALL[next]);
            return;
        }
        _ => {}
    }

    match app.screen {
        Screen::Coins => screen::handle_coins_key(app, key),
        Screen::Objects => screen::handle_objects_key(app, key),
        Screen::Transactions => screen::handle_transactions_key(app, key),
        Screen::Packages => screen::handle_packages_key(app, key),
        Screen::AddressBook => screen::handle_address_key(app, key),
        Screen::Keys => screen::handle_keys_key(app, key),
        Screen::TxBuilder => screen::handle_tx_key(app, key),
        Screen::Explorer => explorer::handle_explorer_key(app, key),
    }
}

fn submit_transaction(app: &mut App) {
    if app.keys.is_empty() || app.tx.commands.is_empty() {
        return;
    }
    if app.validate_balance().is_err() {
        return;
    }

    let gas_budget: u64 = app.tx.gas_budget.parse().unwrap_or(10_000_000);

    app.send_cmd(WalletCmd::ExecutePtb {
        sender_idx: app.tx.sender,
        commands: app.tx.commands.clone(),
        gas_budget,
    });
}

fn trigger_dry_run(app: &mut App) {
    if !app.tx.dry_run_dirty || app.keys.is_empty() || app.tx.commands.is_empty() {
        return;
    }
    app.tx.dry_run = None;
    app.tx.dry_running = true;
    app.tx.dry_run_dirty = false;
    app.send_cmd(WalletCmd::DryRun {
        sender_idx: app.tx.sender,
        commands: app.tx.commands.clone(),
    });
}
