//! Event handling — dispatches keyboard and mouse events to the appropriate handler.

mod context_menu;
mod explorer;
pub(crate) mod grid;
mod input;
mod mouse;
pub(crate) mod nav;
mod popup;
mod screen;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, InputMode, Popup, Screen, Section};
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
    // Ctrl+C always quits immediately
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.running = false;
        return;
    }

    // Konami code tracker (theme toggle)
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

    // Layer 1: Popup overlay (highest priority)
    if app.popup.is_some() {
        popup::handle_popup_key(app, key);
        return;
    }

    // Layer 2: Context menu
    if app.context_menu.is_some() {
        context_menu::handle_context_menu_key(app, key);
        return;
    }

    // Layer 3: Section overlay (full-screen section view)
    if let Some(section) = app.section_open {
        handle_section_overlay_key(app, key, section);
        return;
    }

    // Layer 3.5: Explorer overlay (when exploring an external entity)
    if app.exploring.is_some() {
        handle_explorer_overlay_key(app, key);
        return;
    }

    // Layer 4: Tx Builder overlay
    if app.tx_builder_open {
        handle_tx_builder_overlay_key(app, key);
        return;
    }

    // Layer 5: Global shortcuts (skip when search bar is focused)
    if !app.search_focused {
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
                app.request_refresh();
                return;
            }
            KeyCode::Char('n') => {
                app.open_popup(Popup::SwitchNetwork);
                return;
            }
            KeyCode::Char('E') => {
                app.load_error_log();
                app.open_popup(Popup::ErrorLog);
                return;
            }
            _ => {}
        }
    }

    // Layer 6: Grid view
    grid::handle_grid_key(app, key);
}

/// Handle keys when a section is open as a full overlay.
fn handle_section_overlay_key(app: &mut App, key: KeyEvent, section: Section) {
    // Esc closes the overlay
    if key.code == KeyCode::Esc
        && app.input_mode != InputMode::Editing
        && !has_active_filter(app, section)
    {
        app.section_open = None;
        return;
    }

    // Global shortcuts available inside overlays
    if app.input_mode != InputMode::Editing {
        match key.code {
            KeyCode::Char('q') => {
                app.open_popup(Popup::ConfirmQuit);
                return;
            }
            KeyCode::Char('?') => {
                app.open_popup(Popup::Help);
                return;
            }
            KeyCode::Char('n') => {
                app.open_popup(Popup::SwitchNetwork);
                return;
            }
            KeyCode::Char('E') => {
                app.load_error_log();
                app.open_popup(Popup::ErrorLog);
                return;
            }
            _ => {}
        }
    }

    // Context menu trigger
    if key.code == KeyCode::Char('/') && app.input_mode != InputMode::Editing {
        // Open context menu inside the overlay
        let is_own = app.exploring.is_none();
        let actions = crate::app::actions_for(section, is_own);
        if !actions.is_empty() {
            let anchor_row = app.content_area.y + app.content_area.height / 2;
            let anchor_col = app.content_area.x + app.content_area.width / 2;
            app.context_menu = Some(crate::app::ContextMenu {
                section,
                actions,
                selected: 0,
                anchor_row,
                anchor_col,
            });
        }
        return;
    }

    // Set the legacy screen for existing handlers
    app.screen = section.to_screen();

    // Route to existing screen-specific handlers
    if app.input_mode == InputMode::Editing {
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

    // Filter mode inside overlay
    let filter_active = has_active_filter(app, section);
    if filter_active {
        match section {
            Section::Coins => screen::handle_coins_key(app, key),
            Section::Objects => screen::handle_objects_key(app, key),
            Section::Transactions => screen::handle_transactions_key(app, key),
            _ => {}
        }
        return;
    }

    // Normal mode: route to screen handler
    match section {
        Section::Coins => screen::handle_coins_key(app, key),
        Section::Objects => screen::handle_objects_key(app, key),
        Section::Staking => screen::handle_staking_key(app, key),
        Section::Transactions => screen::handle_transactions_key(app, key),
        Section::Packages => screen::handle_packages_key(app, key),
    }
}

/// Handle keys when the Tx Builder overlay is open.
fn handle_tx_builder_overlay_key(app: &mut App, key: KeyEvent) {
    app.screen = Screen::TxBuilder;

    // Esc in normal mode closes the overlay
    if key.code == KeyCode::Esc && app.input_mode != InputMode::Editing {
        if app.tx.commands.is_empty() {
            app.tx_builder_open = false;
        } else {
            app.open_popup(Popup::ConfirmClearTx);
        }
        return;
    }

    screen::handle_tx_key(app, key);
}

/// Handle keys when the explorer overlay is showing (exploring an external entity).
fn handle_explorer_overlay_key(app: &mut App, key: KeyEvent) {
    // Global shortcuts available inside the explorer
    if app.input_mode != InputMode::Editing {
        match key.code {
            KeyCode::Char('q') => {
                app.open_popup(Popup::ConfirmQuit);
                return;
            }
            KeyCode::Char('?') => {
                app.open_popup(Popup::Help);
                return;
            }
            KeyCode::Char('n') => {
                app.open_popup(Popup::SwitchNetwork);
                return;
            }
            KeyCode::Char('E') => {
                app.load_error_log();
                app.open_popup(Popup::ErrorLog);
                return;
            }
            KeyCode::Esc => {
                // Close explorer and return to grid
                app.exploring = None;
                app.search_buffer.clear();
                app.request_refresh();
                return;
            }
            _ => {}
        }
    }

    // Route to the existing explorer key handler
    app.screen = Screen::Explorer;
    explorer::handle_explorer_key(app, key);
}

fn has_active_filter(app: &App, section: Section) -> bool {
    match section {
        Section::Coins => app.coins_filter.is_some(),
        Section::Objects => app.objects_filter.is_some(),
        Section::Transactions => app.transactions_filter.is_some(),
        _ => false,
    }
}

pub(crate) fn submit_transaction(app: &mut App) {
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

pub(crate) fn trigger_dry_run(app: &mut App) {
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
