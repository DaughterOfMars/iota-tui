//! Per-screen keyboard event handlers.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

use crate::app::{App, InputMode, Popup, TxBuilderStep};
use crate::wallet::WalletCmd;

use super::input::handle_input_key;
use super::nav::ListNav;

pub fn handle_coins_key(app: &mut App, key: KeyEvent) {
    // Filter mode
    if app.coins_filter.is_some() {
        let filtered = app.filtered_coins();
        match key.code {
            KeyCode::Esc => {
                app.coins_filter = None;
                app.coins_selected = 0;
                app.coins_offset = 0;
            }
            KeyCode::Backspace => {
                if let Some(ref mut q) = app.coins_filter {
                    q.pop();
                    if q.is_empty() {
                        app.coins_filter = None;
                    }
                }
                app.coins_selected = 0;
                app.coins_offset = 0;
            }
            KeyCode::Char(c) => {
                if let Some(ref mut q) = app.coins_filter {
                    q.push(c);
                }
                app.coins_selected = 0;
                app.coins_offset = 0;
            }
            KeyCode::Up => {
                if app.coins_selected > 0 {
                    app.coins_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.coins_selected + 1 < filtered.len() {
                    app.coins_selected += 1;
                }
            }
            KeyCode::Enter => {
                // Activate using the filtered index
                if let Some(&real_idx) = filtered.get(app.coins_selected) {
                    let old = app.coins_selected;
                    app.coins_selected = real_idx;
                    app.activate_selected_coin();
                    app.coins_selected = old;
                }
            }
            _ => {}
        }
        let filtered = app.filtered_coins();
        if app.coins_selected >= filtered.len() {
            app.coins_selected = filtered.len().saturating_sub(1);
        }
        App::scroll_into_view(
            app.coins_selected,
            &mut app.coins_offset,
            app.content_visible_rows,
        );
        return;
    }

    let mut nav = ListNav {
        selected: &mut app.coins_selected,
        offset: &mut app.coins_offset,
        len: app.coins.len(),
        visible_rows: app.content_visible_rows,
    };
    if nav.handle_key(key.code) {
        return;
    }
    match key.code {
        KeyCode::Enter => {
            app.activate_selected_coin();
        }
        KeyCode::Char('t') => {
            if let Some(coin) = app.coins.get(app.coins_selected) {
                let ct = coin.coin_type.clone();
                app.explore_type(ct);
            }
        }
        KeyCode::Char('f') => {
            if let Some(key) = app.active_key()
                && let Ok(addr) = iota_sdk::types::Address::from_hex(&key.address)
            {
                app.send_cmd(WalletCmd::RequestFaucet(addr));
            }
        }
        KeyCode::Char('/') => {
            app.coins_filter = Some(String::new());
            app.coins_selected = 0;
            app.coins_offset = 0;
        }
        _ => {}
    }
}

pub fn handle_objects_key(app: &mut App, key: KeyEvent) {
    // Filter mode
    if app.objects_filter.is_some() {
        let filtered = app.filtered_objects();
        match key.code {
            KeyCode::Esc => {
                app.objects_filter = None;
                app.objects_selected = 0;
                app.objects_offset = 0;
            }
            KeyCode::Backspace => {
                if let Some(ref mut q) = app.objects_filter {
                    q.pop();
                    if q.is_empty() {
                        app.objects_filter = None;
                    }
                }
                app.objects_selected = 0;
                app.objects_offset = 0;
            }
            KeyCode::Char(c) => {
                if let Some(ref mut q) = app.objects_filter {
                    q.push(c);
                }
                app.objects_selected = 0;
                app.objects_offset = 0;
            }
            KeyCode::Up => {
                if app.objects_selected > 0 {
                    app.objects_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.objects_selected + 1 < filtered.len() {
                    app.objects_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(&real_idx) = filtered.get(app.objects_selected) {
                    let old = app.objects_selected;
                    app.objects_selected = real_idx;
                    app.activate_selected_object();
                    app.objects_selected = old;
                }
            }
            _ => {}
        }
        let filtered = app.filtered_objects();
        if app.objects_selected >= filtered.len() {
            app.objects_selected = filtered.len().saturating_sub(1);
        }
        App::scroll_into_view(
            app.objects_selected,
            &mut app.objects_offset,
            app.content_visible_rows,
        );
        return;
    }

    let mut nav = ListNav {
        selected: &mut app.objects_selected,
        offset: &mut app.objects_offset,
        len: app.objects.len(),
        visible_rows: app.content_visible_rows,
    };
    if nav.handle_key(key.code) {
        return;
    }
    match key.code {
        KeyCode::Enter => {
            app.activate_selected_object();
        }
        KeyCode::Char('t') => {
            if let Some(obj) = app.objects.get(app.objects_selected) {
                let tn = obj.type_name.clone();
                app.explore_type(tn);
            }
        }
        KeyCode::Char('/') => {
            app.objects_filter = Some(String::new());
            app.objects_selected = 0;
            app.objects_offset = 0;
        }
        _ => {}
    }
}

pub fn handle_transactions_key(app: &mut App, key: KeyEvent) {
    // Filter mode
    if app.transactions_filter.is_some() {
        let filtered = app.filtered_transactions();
        match key.code {
            KeyCode::Esc => {
                app.transactions_filter = None;
                app.transactions_selected = 0;
                app.transactions_offset = 0;
            }
            KeyCode::Backspace => {
                if let Some(ref mut q) = app.transactions_filter {
                    q.pop();
                    if q.is_empty() {
                        app.transactions_filter = None;
                    }
                }
                app.transactions_selected = 0;
                app.transactions_offset = 0;
            }
            KeyCode::Char(c) => {
                if let Some(ref mut q) = app.transactions_filter {
                    q.push(c);
                }
                app.transactions_selected = 0;
                app.transactions_offset = 0;
            }
            KeyCode::Up => {
                if app.transactions_selected > 0 {
                    app.transactions_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.transactions_selected + 1 < filtered.len() {
                    app.transactions_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(&real_idx) = filtered.get(app.transactions_selected) {
                    let old = app.transactions_selected;
                    app.transactions_selected = real_idx;
                    app.activate_selected_transaction();
                    app.transactions_selected = old;
                }
            }
            _ => {}
        }
        let filtered = app.filtered_transactions();
        if app.transactions_selected >= filtered.len() {
            app.transactions_selected = filtered.len().saturating_sub(1);
        }
        App::scroll_into_view(
            app.transactions_selected,
            &mut app.transactions_offset,
            app.content_visible_rows,
        );
        return;
    }

    let mut nav = ListNav {
        selected: &mut app.transactions_selected,
        offset: &mut app.transactions_offset,
        len: app.transactions.len(),
        visible_rows: app.content_visible_rows,
    };
    if nav.handle_key(key.code) {
        return;
    }
    match key.code {
        KeyCode::Enter => {
            app.activate_selected_transaction();
        }
        KeyCode::Char('/') => {
            app.transactions_filter = Some(String::new());
            app.transactions_selected = 0;
            app.transactions_offset = 0;
        }
        _ => {}
    }
}

pub fn handle_packages_key(app: &mut App, key: KeyEvent) {
    let packages = app.package_indices();
    let mut nav = ListNav {
        selected: &mut app.packages_selected,
        offset: &mut app.packages_offset,
        len: packages.len(),
        visible_rows: app.content_visible_rows,
    };
    if nav.handle_key(key.code) {
        return;
    }
    if key.code == KeyCode::Enter {
        app.activate_selected_package();
    }
}

pub fn handle_address_key(app: &mut App, key: KeyEvent) {
    let combined_len = app.key_entry_count() + app.address_book.len();
    let mut nav = ListNav {
        selected: &mut app.address_selected,
        offset: &mut app.address_offset,
        len: combined_len,
        visible_rows: app.content_visible_rows,
    };
    if nav.handle_key(key.code) {
        return;
    }
    match key.code {
        KeyCode::Enter => {
            app.activate_selected_address();
        }
        KeyCode::Char('a') => {
            app.address_edit_field = 0;
            app.address_edit_buffers = [String::new(), String::new(), String::new()];
            app.open_popup(Popup::AddAddress);
            app.start_input("");
        }
        KeyCode::Char('e') => {
            if let Some(user_idx) = app.user_address_index(app.address_selected)
                && let Some(entry) = app.address_book.get(user_idx)
            {
                let label = entry.label.clone();
                let address = entry.address.clone();
                let notes = entry.notes.clone();
                app.address_edit_field = 0;
                app.address_edit_buffers = [label.clone(), address, notes];
                app.open_popup(Popup::EditAddress);
                app.start_input(&label);
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if let Some(user_idx) = app.user_address_index(app.address_selected)
                && user_idx < app.address_book.len()
            {
                app.open_popup(Popup::ConfirmDeleteAddress);
            }
        }
        KeyCode::Char('l') => {
            app.open_popup(Popup::LookupIotaName);
            app.start_input("");
        }
        _ => {}
    }
}

pub fn handle_keys_key(app: &mut App, key: KeyEvent) {
    let mut nav = ListNav {
        selected: &mut app.keys_selected,
        offset: &mut app.keys_offset,
        len: app.keys.len(),
        visible_rows: app.content_visible_rows,
    };
    if nav.handle_key(key.code) {
        return;
    }
    match key.code {
        KeyCode::Enter => {
            if let Some(key) = app.keys.get(app.keys_selected) {
                let addr = key.address.clone();
                app.explore_item(addr);
            }
        }
        KeyCode::Char('a') => {
            app.activate_selected_key();
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
}

pub fn handle_tx_key(app: &mut App, key: KeyEvent) {
    // Global tx builder keybind: clear/reset (when not editing)
    if app.input_mode != InputMode::Editing && key.code == KeyCode::Char('c') {
        if app.tx.commands.is_empty() {
            app.tx.reset();
        } else {
            app.open_popup(Popup::ConfirmClearTx);
        }
        return;
    }
    match app.tx.step {
        TxBuilderStep::SelectSender => match key.code {
            KeyCode::Up => {
                if app.tx.sender > 0 {
                    app.tx.sender -= 1;
                    app.tx.dry_run_dirty = true;
                }
            }
            KeyCode::Down => {
                if app.tx.sender + 1 < app.keys.len() {
                    app.tx.sender += 1;
                    app.tx.dry_run_dirty = true;
                }
            }
            KeyCode::Enter | KeyCode::Right => {
                app.tx.step = TxBuilderStep::EditCommands;
            }
            _ => {}
        },
        TxBuilderStep::EditCommands => match key.code {
            KeyCode::Left => {
                app.tx.step = TxBuilderStep::SelectSender;
            }
            KeyCode::Right => {
                app.tx.step = TxBuilderStep::SetGas;
            }
            KeyCode::Char('a') => {
                app.open_popup(Popup::AddCommand);
            }
            KeyCode::Up => {
                if app.tx.cmd_selected > 0 {
                    app.tx.cmd_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.tx.cmd_selected + 1 < app.tx.commands.len() {
                    app.tx.cmd_selected += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if !app.tx.commands.is_empty() {
                    app.tx.commands.remove(app.tx.cmd_selected);
                    app.tx.dry_run_dirty = true;
                    if app.tx.cmd_selected >= app.tx.commands.len() && app.tx.cmd_selected > 0 {
                        app.tx.cmd_selected -= 1;
                    }
                }
            }
            _ => {}
        },
        TxBuilderStep::SetGas => {
            if app.input_mode == InputMode::Editing {
                match key.code {
                    KeyCode::Enter => {
                        app.tx.gas_budget = app.stop_input();
                        app.tx.gas_edited = true;
                    }
                    KeyCode::Esc => {
                        app.stop_input();
                    }
                    _ => handle_input_key(app, key),
                }
            } else {
                match key.code {
                    KeyCode::Left => {
                        app.tx.step = TxBuilderStep::EditCommands;
                    }
                    KeyCode::Right => {
                        app.tx.step = TxBuilderStep::Review;
                        super::trigger_dry_run(app);
                    }
                    KeyCode::Enter | KeyCode::Char('e') => {
                        app.start_input(&app.tx.gas_budget.clone());
                    }
                    _ => {}
                }
            }
        }
        TxBuilderStep::Review => match key.code {
            KeyCode::Left => {
                app.tx.step = TxBuilderStep::SetGas;
            }
            KeyCode::Enter => {
                super::submit_transaction(app);
            }
            _ => {}
        },
    }
}
