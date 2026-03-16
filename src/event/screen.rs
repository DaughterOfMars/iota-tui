//! Per-screen keyboard event handlers.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

use crate::app::{App, ExplorerView, InputMode, Popup, TxBuilderStep};
use crate::wallet::WalletCmd;

use super::input::handle_input_key;

pub fn handle_coins_key(app: &mut App, key: KeyEvent) {
    let len = app.coins.len();
    match key.code {
        KeyCode::Up => {
            if app.coins_selected > 0 {
                app.coins_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.coins_selected + 1 < len {
                app.coins_selected += 1;
            }
        }
        KeyCode::Home => {
            app.coins_selected = 0;
        }
        KeyCode::End => {
            if len > 0 {
                app.coins_selected = len - 1;
            }
        }
        KeyCode::PageUp => {
            app.coins_selected = app.coins_selected.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.coins_selected = (app.coins_selected + 10).min(len.saturating_sub(1));
        }
        KeyCode::Enter => {
            if !app.coins.is_empty() {
                app.open_popup(Popup::Detail);
            }
        }
        KeyCode::Char('f') => {
            if let Some(key) = app.active_key()
                && let Ok(addr) = iota_sdk::types::Address::from_hex(&key.address)
            {
                app.send_cmd(WalletCmd::RequestFaucet(addr));
                app.set_status("Requesting faucet...");
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.coins_selected, &mut app.coins_offset, 20);
}

pub fn handle_objects_key(app: &mut App, key: KeyEvent) {
    let len = app.objects.len();
    match key.code {
        KeyCode::Up => {
            if app.objects_selected > 0 {
                app.objects_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.objects_selected + 1 < len {
                app.objects_selected += 1;
            }
        }
        KeyCode::Home => {
            app.objects_selected = 0;
        }
        KeyCode::End => {
            if len > 0 {
                app.objects_selected = len - 1;
            }
        }
        KeyCode::PageUp => {
            app.objects_selected = app.objects_selected.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.objects_selected = (app.objects_selected + 10).min(len.saturating_sub(1));
        }
        KeyCode::Enter => {
            if !app.objects.is_empty() {
                app.open_popup(Popup::Detail);
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.objects_selected, &mut app.objects_offset, 20);
}

pub fn handle_transactions_key(app: &mut App, key: KeyEvent) {
    let len = app.transactions.len();
    match key.code {
        KeyCode::Up => {
            if app.transactions_selected > 0 {
                app.transactions_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.transactions_selected + 1 < len {
                app.transactions_selected += 1;
            }
        }
        KeyCode::Home => {
            app.transactions_selected = 0;
        }
        KeyCode::End => {
            if len > 0 {
                app.transactions_selected = len - 1;
            }
        }
        KeyCode::PageUp => {
            app.transactions_selected = app.transactions_selected.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.transactions_selected = (app.transactions_selected + 10).min(len.saturating_sub(1));
        }
        KeyCode::Enter => {
            if !app.transactions.is_empty() {
                app.open_popup(Popup::Detail);
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.transactions_selected, &mut app.transactions_offset, 20);
}

pub fn handle_address_key(app: &mut App, key: KeyEvent) {
    let combined_len = app.key_entry_count() + app.address_book.len();
    match key.code {
        KeyCode::Up => {
            if app.address_selected > 0 {
                app.address_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.address_selected + 1 < combined_len {
                app.address_selected += 1;
            }
        }
        KeyCode::Home => {
            app.address_selected = 0;
        }
        KeyCode::End => {
            if combined_len > 0 {
                app.address_selected = combined_len - 1;
            }
        }
        KeyCode::Enter => {
            if combined_len > 0 {
                app.open_popup(Popup::Detail);
            }
        }
        KeyCode::Char('a') => {
            app.address_edit_field = 0;
            app.address_edit_buffers = [String::new(), String::new(), String::new()];
            app.open_popup(Popup::AddAddress);
            app.start_input("");
        }
        KeyCode::Char('e') => {
            if let Some(user_idx) = app.user_address_index(app.address_selected) {
                if let Some(entry) = app.address_book.get(user_idx) {
                    let label = entry.label.clone();
                    let address = entry.address.clone();
                    let notes = entry.notes.clone();
                    app.address_edit_field = 0;
                    app.address_edit_buffers = [label.clone(), address, notes];
                    app.open_popup(Popup::EditAddress);
                    app.start_input(&label);
                }
            } else {
                app.set_status("Key entries are read-only");
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if let Some(user_idx) = app.user_address_index(app.address_selected) {
                if user_idx < app.address_book.len() {
                    app.open_popup(Popup::ConfirmDeleteAddress);
                }
            } else {
                app.set_status("Key entries cannot be deleted here");
            }
        }
        KeyCode::Char('l') => {
            app.open_popup(Popup::LookupIotaName);
            app.start_input("");
        }
        _ => {}
    }
    App::scroll_into_view(app.address_selected, &mut app.address_offset, 20);
}

pub fn handle_keys_key(app: &mut App, key: KeyEvent) {
    let len = app.keys.len();
    match key.code {
        KeyCode::Up => {
            if app.keys_selected > 0 {
                app.keys_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.keys_selected + 1 < len {
                app.keys_selected += 1;
            }
        }
        KeyCode::Home => {
            app.keys_selected = 0;
        }
        KeyCode::End => {
            if len > 0 {
                app.keys_selected = len - 1;
            }
        }
        KeyCode::Enter => {
            let idx = app.keys_selected;
            for (i, k) in app.keys.iter_mut().enumerate() {
                k.is_active = i == idx;
            }
            app.send_cmd(WalletCmd::SetActiveKey(idx));
            app.set_status("Active key changed");
            app.request_refresh();
        }
        KeyCode::Char('g') => {
            app.open_popup(Popup::GenerateKey);
        }
        KeyCode::Char('i') => {
            app.open_popup(Popup::ImportKey);
            app.start_input("");
        }
        KeyCode::Char('e') => {
            if let Some(key_display) = app.keys.get(app.keys_selected) {
                let current = key_display.alias.clone();
                app.open_popup(Popup::RenameKey);
                app.start_input(&current);
            }
        }
        KeyCode::Char('p') => {
            app.keys_show_private = !app.keys_show_private;
        }
        KeyCode::Char(' ') => {
            if let Some(key) = app.keys.get_mut(app.keys_selected) {
                key.visible = !key.visible;
                app.request_refresh();
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if !app.keys.is_empty() {
                app.open_popup(Popup::ConfirmDeleteKey);
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.keys_selected, &mut app.keys_offset, 20);
}

pub fn handle_tx_key(app: &mut App, key: KeyEvent) {
    // Global tx builder keybind: clear/reset (when not editing)
    if app.input_mode != InputMode::Editing && key.code == KeyCode::Char('c') {
        app.reset_tx_builder();
        app.set_status("Transaction cleared");
        return;
    }
    match app.tx_step {
        TxBuilderStep::SelectSender => match key.code {
            KeyCode::Up => {
                if app.tx_sender > 0 {
                    app.tx_sender -= 1;
                    app.tx_dry_run_dirty = true;
                }
            }
            KeyCode::Down => {
                if app.tx_sender + 1 < app.keys.len() {
                    app.tx_sender += 1;
                    app.tx_dry_run_dirty = true;
                }
            }
            KeyCode::Enter | KeyCode::Right => {
                app.tx_step = TxBuilderStep::EditCommands;
            }
            _ => {}
        },
        TxBuilderStep::EditCommands => match key.code {
            KeyCode::Left => {
                app.tx_step = TxBuilderStep::SelectSender;
            }
            KeyCode::Right => {
                app.tx_step = TxBuilderStep::SetGas;
            }
            KeyCode::Char('a') => {
                app.open_popup(Popup::AddCommand);
            }
            KeyCode::Up => {
                if app.tx_cmd_selected > 0 {
                    app.tx_cmd_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.tx_cmd_selected + 1 < app.tx_commands.len() {
                    app.tx_cmd_selected += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if !app.tx_commands.is_empty() {
                    app.tx_commands.remove(app.tx_cmd_selected);
                    app.tx_dry_run_dirty = true;
                    if app.tx_cmd_selected >= app.tx_commands.len() && app.tx_cmd_selected > 0 {
                        app.tx_cmd_selected -= 1;
                    }
                }
            }
            _ => {}
        },
        TxBuilderStep::SetGas => {
            if app.input_mode == InputMode::Editing {
                match key.code {
                    KeyCode::Enter => {
                        app.tx_gas_budget = app.stop_input();
                        app.tx_gas_edited = true;
                    }
                    KeyCode::Esc => {
                        app.stop_input();
                    }
                    _ => handle_input_key(app, key),
                }
            } else {
                match key.code {
                    KeyCode::Left => {
                        app.tx_step = TxBuilderStep::EditCommands;
                    }
                    KeyCode::Right => {
                        app.tx_step = TxBuilderStep::Review;
                        super::trigger_dry_run(app);
                    }
                    KeyCode::Enter | KeyCode::Char('e') => {
                        app.start_input(&app.tx_gas_budget.clone());
                    }
                    _ => {}
                }
            }
        }
        TxBuilderStep::Review => match key.code {
            KeyCode::Left => {
                app.tx_step = TxBuilderStep::SetGas;
            }
            KeyCode::Enter => {
                super::submit_transaction(app);
            }
            _ => {}
        },
    }
}

pub fn handle_explorer_key(app: &mut App, key: KeyEvent) {
    // When editing (lookup input), handle text input
    if app.input_mode == InputMode::Editing {
        match key.code {
            KeyCode::Enter => {
                let query = app.stop_input();
                if !query.is_empty() {
                    if app.explorer_search_mode {
                        app.send_cmd(WalletCmd::SearchObjectsByType(query));
                        app.set_status("Searching objects by type...");
                    } else {
                        app.send_cmd(WalletCmd::LookupAddress(query));
                        app.set_status("Looking up...");
                    }
                }
            }
            KeyCode::Esc => {
                app.stop_input();
            }
            _ => handle_input_key(app, key),
        }
        return;
    }

    // Sub-view navigation with Left/Right
    match key.code {
        KeyCode::Left => {
            let idx = app.explorer_view.index();
            if idx > 0 {
                app.explorer_view = ExplorerView::ALL[idx - 1];
                app.refresh_explorer();
            }
            return;
        }
        KeyCode::Right => {
            let idx = app.explorer_view.index();
            if idx + 1 < ExplorerView::ALL.len() {
                app.explorer_view = ExplorerView::ALL[idx + 1];
                app.refresh_explorer();
            }
            return;
        }
        _ => {}
    }

    match app.explorer_view {
        ExplorerView::Overview => {}
        ExplorerView::Checkpoints => {
            let len = app.explorer_checkpoints.len();
            match key.code {
                KeyCode::Up => {
                    if app.explorer_checkpoints_selected > 0 {
                        app.explorer_checkpoints_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.explorer_checkpoints_selected + 1 < len {
                        app.explorer_checkpoints_selected += 1;
                    }
                }
                KeyCode::Home => app.explorer_checkpoints_selected = 0,
                KeyCode::End => {
                    if len > 0 {
                        app.explorer_checkpoints_selected = len - 1;
                    }
                }
                KeyCode::Enter => {
                    if !app.explorer_checkpoints.is_empty() {
                        app.open_popup(Popup::Detail);
                    }
                }
                _ => {}
            }
            App::scroll_into_view(
                app.explorer_checkpoints_selected,
                &mut app.explorer_checkpoints_offset,
                20,
            );
        }
        ExplorerView::Validators => {
            let len = app.explorer_validators.len();
            match key.code {
                KeyCode::Up => {
                    if app.explorer_validators_selected > 0 {
                        app.explorer_validators_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.explorer_validators_selected + 1 < len {
                        app.explorer_validators_selected += 1;
                    }
                }
                KeyCode::Home => app.explorer_validators_selected = 0,
                KeyCode::End => {
                    if len > 0 {
                        app.explorer_validators_selected = len - 1;
                    }
                }
                KeyCode::Enter => {
                    if !app.explorer_validators.is_empty() {
                        app.open_popup(Popup::Detail);
                    }
                }
                _ => {}
            }
            App::scroll_into_view(
                app.explorer_validators_selected,
                &mut app.explorer_validators_offset,
                20,
            );
        }
        ExplorerView::Lookup => {
            match key.code {
                KeyCode::Enter => {
                    app.explorer_search_mode = false;
                    app.start_input("");
                }
                KeyCode::Char('s') => {
                    app.explorer_search_mode = true;
                    app.start_input("");
                }
                KeyCode::Esc => {
                    app.explorer_lookup_result = None;
                    app.explorer_search_results.clear();
                    app.explorer_search_selected = 0;
                }
                KeyCode::Up => {
                    if !app.explorer_search_results.is_empty() && app.explorer_search_selected > 0 {
                        app.explorer_search_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.explorer_search_selected + 1 < app.explorer_search_results.len() {
                        app.explorer_search_selected += 1;
                    }
                }
                _ => {}
            }
            if !app.explorer_search_results.is_empty() {
                App::scroll_into_view(
                    app.explorer_search_selected,
                    &mut app.explorer_search_offset,
                    20,
                );
            }
        }
    }
}
