//! Per-screen keyboard event handlers.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

use crate::app::{App, ExplorerView, InputMode, LookupAction, Popup, TxBuilderStep};
use crate::wallet::WalletCmd;

use super::input::handle_input_key;
use super::nav::ListNav;

pub fn handle_coins_key(app: &mut App, key: KeyEvent) {
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
            if let Some(coin) = app.coins.get(app.coins_selected) {
                let id = coin.object_id.clone();
                app.explore_item(id);
            }
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
                app.set_status("Requesting faucet...");
            }
        }
        _ => {}
    }
}

pub fn handle_objects_key(app: &mut App, key: KeyEvent) {
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
            if let Some(obj) = app.objects.get(app.objects_selected) {
                let id = obj.object_id.clone();
                app.explore_item(id);
            }
        }
        KeyCode::Char('t') => {
            if let Some(obj) = app.objects.get(app.objects_selected) {
                let tn = obj.type_name.clone();
                app.explore_type(tn);
            }
        }
        _ => {}
    }
}

pub fn handle_transactions_key(app: &mut App, key: KeyEvent) {
    let mut nav = ListNav {
        selected: &mut app.transactions_selected,
        offset: &mut app.transactions_offset,
        len: app.transactions.len(),
        visible_rows: app.content_visible_rows,
    };
    if nav.handle_key(key.code) {
        return;
    }
    if key.code == KeyCode::Enter
        && let Some(tx) = app.transactions.get(app.transactions_selected)
    {
        let digest = tx.digest.clone();
        app.explore_item(digest);
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
    if key.code == KeyCode::Enter
        && let Some(&obj_idx) = packages.get(app.packages_selected)
    {
        let id = app.objects[obj_idx].object_id.clone();
        app.explore_item(id);
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
            let combined = app.combined_address_book();
            if let Some(entry) = combined.get(app.address_selected) {
                let addr = entry.address.clone();
                app.explore_item(addr);
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
            let idx = app.keys_selected;
            for (i, k) in app.keys.iter_mut().enumerate() {
                k.is_active = i == idx;
            }
            app.send_cmd(WalletCmd::SetActiveKey(idx));
            app.set_status("Active key changed");
            app.request_refresh();
        }
        KeyCode::Char('x') => {
            if let Some(key) = app.keys.get(app.keys_selected) {
                let addr = key.address.clone();
                app.explore_item(addr);
            }
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
        if app.tx_commands.is_empty() {
            app.reset_tx_builder();
            app.set_status("Transaction cleared");
        } else {
            app.open_popup(Popup::ConfirmClearTx);
        }
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
                        app.explorer_search_type = query.clone();
                        app.explorer_search_cursors.clear();
                        app.explorer_search_has_next = false;
                        app.explorer_search_cursor = None;
                        app.send_cmd(WalletCmd::SearchObjectsByType {
                            type_filter: query,
                            cursor: None,
                        });
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
            // If filtering, handle text input
            if app.explorer_checkpoints_filter.is_some() {
                match key.code {
                    KeyCode::Esc => {
                        app.explorer_checkpoints_filter = None;
                        app.explorer_checkpoints_selected = 0;
                        app.explorer_checkpoints_offset = 0;
                    }
                    KeyCode::Enter => {
                        // Keep filter active but stop editing — just deselect input
                    }
                    KeyCode::Backspace => {
                        if let Some(ref mut q) = app.explorer_checkpoints_filter {
                            q.pop();
                            if q.is_empty() {
                                app.explorer_checkpoints_filter = None;
                            }
                        }
                        app.explorer_checkpoints_selected = 0;
                        app.explorer_checkpoints_offset = 0;
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        if let Some(ref mut q) = app.explorer_checkpoints_filter {
                            q.push(c);
                        }
                        app.explorer_checkpoints_selected = 0;
                        app.explorer_checkpoints_offset = 0;
                    }
                    KeyCode::Up => {
                        if app.explorer_checkpoints_selected > 0 {
                            app.explorer_checkpoints_selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let len = app.filtered_checkpoints().len();
                        if app.explorer_checkpoints_selected + 1 < len {
                            app.explorer_checkpoints_selected += 1;
                        }
                    }
                    _ => {}
                }
                let len = app.filtered_checkpoints().len();
                if app.explorer_checkpoints_selected >= len {
                    app.explorer_checkpoints_selected = len.saturating_sub(1);
                }
                App::scroll_into_view(
                    app.explorer_checkpoints_selected,
                    &mut app.explorer_checkpoints_offset,
                    app.explorer_visible_rows,
                );
                return;
            }

            let len = app.filtered_checkpoints().len();
            let mut nav = ListNav {
                selected: &mut app.explorer_checkpoints_selected,
                offset: &mut app.explorer_checkpoints_offset,
                len,
                visible_rows: app.explorer_visible_rows,
            };
            if nav.handle_key(key.code) {
                return;
            }
            match key.code {
                KeyCode::Enter => {
                    if !app.explorer_checkpoints.is_empty() {
                        app.open_popup(Popup::Detail);
                    }
                }
                KeyCode::Char('s') => {
                    app.explorer_checkpoints_sort_asc = !app.explorer_checkpoints_sort_asc;
                    app.explorer_checkpoints_selected = 0;
                    app.explorer_checkpoints_offset = 0;
                }
                KeyCode::Char('/') => {
                    app.explorer_checkpoints_filter = Some(String::new());
                    app.explorer_checkpoints_selected = 0;
                    app.explorer_checkpoints_offset = 0;
                }
                KeyCode::Char(']') if app.explorer_checkpoints_has_next => {
                    app.explorer_checkpoints_cursors
                        .push(app.explorer_checkpoints_cursor.clone());
                    app.explorer_checkpoints_page += 1;
                    let cursor = app.explorer_checkpoints_cursor.clone();
                    app.send_cmd(WalletCmd::RefreshCheckpoints { cursor });
                    app.set_status("Loading next page...");
                }
                KeyCode::Char('[') if !app.explorer_checkpoints_cursors.is_empty() => {
                    let prev = app.explorer_checkpoints_cursors.pop().flatten();
                    app.explorer_checkpoints_page = app.explorer_checkpoints_page.saturating_sub(1);
                    app.send_cmd(WalletCmd::RefreshCheckpoints { cursor: prev });
                    app.set_status("Loading previous page...");
                }
                _ => {}
            }
        }
        ExplorerView::Validators => {
            let mut nav = ListNav {
                selected: &mut app.explorer_validators_selected,
                offset: &mut app.explorer_validators_offset,
                len: app.explorer_validators.len(),
                visible_rows: app.explorer_visible_rows,
            };
            if nav.handle_key(key.code) {
                return;
            }
            if key.code == KeyCode::Enter
                && let Some(v) = app
                    .explorer_validators
                    .get(app.explorer_validators_selected)
            {
                let addr = v.address.clone();
                app.explore_item(addr);
            }
        }
        ExplorerView::Lookup => {
            match key.code {
                KeyCode::Enter => {
                    // If search results are showing, explore the selected one
                    if let Some(obj) = app
                        .explorer_search_results
                        .get(app.explorer_search_selected)
                    {
                        let id = obj.object_id.clone();
                        app.explorer_search_results.clear();
                        app.explore_item(id);
                        return;
                    }
                    // If lookup result is showing, follow the selected field's action
                    if let Some(ref result) = app.explorer_lookup_result {
                        if let Some(field) = result.field_at(app.explorer_lookup_selected) {
                            match &field.action {
                                Some(LookupAction::Explore(val)) => {
                                    let val = val.clone();
                                    app.explore_item(val);
                                    return;
                                }
                                Some(LookupAction::TypeSearch(val)) => {
                                    let val = val.clone();
                                    app.explore_type(val);
                                    return;
                                }
                                None => {}
                            }
                        }
                        // No action on this field — do nothing (don't open input)
                        return;
                    }
                    // No results at all — open lookup input
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
                    app.explorer_lookup_selected = 0;
                    app.explorer_lookup_offset = 0;
                    app.explorer_search_has_next = false;
                    app.explorer_search_cursor = None;
                    app.explorer_search_cursors.clear();
                }
                KeyCode::Up => {
                    if !app.explorer_search_results.is_empty() {
                        if app.explorer_search_selected > 0 {
                            app.explorer_search_selected -= 1;
                        }
                    } else if app.explorer_lookup_result.is_some()
                        && app.explorer_lookup_selected > 0
                    {
                        app.explorer_lookup_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if !app.explorer_search_results.is_empty() {
                        if app.explorer_search_selected + 1 < app.explorer_search_results.len() {
                            app.explorer_search_selected += 1;
                        }
                    } else if let Some(ref result) = app.explorer_lookup_result {
                        let total = result.total_fields();
                        if app.explorer_lookup_selected + 1 < total {
                            app.explorer_lookup_selected += 1;
                        }
                    }
                }
                KeyCode::Home => {
                    app.explorer_search_selected = 0;
                    app.explorer_lookup_selected = 0;
                }
                KeyCode::End => {
                    if !app.explorer_search_results.is_empty() {
                        app.explorer_search_selected =
                            app.explorer_search_results.len().saturating_sub(1);
                    } else if let Some(ref result) = app.explorer_lookup_result {
                        app.explorer_lookup_selected = result.total_fields().saturating_sub(1);
                    }
                }
                KeyCode::Char(']')
                    if !app.explorer_search_results.is_empty() && app.explorer_search_has_next =>
                {
                    // Save current cursor for going back
                    app.explorer_search_cursors
                        .push(app.explorer_search_cursor.clone());
                    // Fetch next page using end_cursor from last response
                    let cursor = app.explorer_search_cursor.clone();
                    let type_filter = app.explorer_search_type.clone();
                    app.send_cmd(WalletCmd::SearchObjectsByType {
                        type_filter,
                        cursor,
                    });
                    app.set_status("Loading next page...");
                }
                KeyCode::Char('[')
                    if !app.explorer_search_results.is_empty()
                        && !app.explorer_search_cursors.is_empty() =>
                {
                    // Pop the previous cursor to go back
                    let prev_cursor = app.explorer_search_cursors.pop().flatten();
                    let type_filter = app.explorer_search_type.clone();
                    app.send_cmd(WalletCmd::SearchObjectsByType {
                        type_filter,
                        cursor: prev_cursor,
                    });
                    app.set_status("Loading previous page...");
                }
                // Address lookup pagination: next page
                KeyCode::Char(']')
                    if app.explorer_search_results.is_empty()
                        && app.explorer_lookup_address.is_some()
                        && matches!(
                            app.explorer_lookup_result,
                            Some(crate::app::LookupResult::Address { .. })
                        )
                        && (app.explorer_lookup_obj_has_next
                            || app.explorer_lookup_tx_has_next) =>
                {
                    app.explorer_lookup_obj_cursors
                        .push(app.explorer_lookup_obj_cursor.clone());
                    app.explorer_lookup_tx_cursors
                        .push(app.explorer_lookup_tx_cursor.clone());
                    app.explorer_lookup_obj_page += 1;
                    app.explorer_lookup_tx_page += 1;
                    let address = app.explorer_lookup_address.clone().unwrap();
                    let obj_cursor = app.explorer_lookup_obj_cursor.clone();
                    let tx_cursor = app.explorer_lookup_tx_cursor.clone();
                    app.send_cmd(WalletCmd::LookupAddressPage {
                        address,
                        obj_cursor,
                        tx_cursor,
                    });
                    app.set_status("Loading next page...");
                }
                // Address lookup pagination: prev page
                KeyCode::Char('[')
                    if app.explorer_search_results.is_empty()
                        && app.explorer_lookup_address.is_some()
                        && matches!(
                            app.explorer_lookup_result,
                            Some(crate::app::LookupResult::Address { .. })
                        )
                        && !app.explorer_lookup_obj_cursors.is_empty() =>
                {
                    let prev_obj = app.explorer_lookup_obj_cursors.pop().flatten();
                    let prev_tx = app.explorer_lookup_tx_cursors.pop().flatten();
                    app.explorer_lookup_obj_page = app.explorer_lookup_obj_page.saturating_sub(1);
                    app.explorer_lookup_tx_page = app.explorer_lookup_tx_page.saturating_sub(1);
                    let address = app.explorer_lookup_address.clone().unwrap();
                    app.send_cmd(WalletCmd::LookupAddressPage {
                        address,
                        obj_cursor: prev_obj,
                        tx_cursor: prev_tx,
                    });
                    app.set_status("Loading previous page...");
                }
                _ => {}
            }
            if !app.explorer_search_results.is_empty() {
                App::scroll_into_view(
                    app.explorer_search_selected,
                    &mut app.explorer_search_offset,
                    app.explorer_visible_rows,
                );
            } else if let Some(ref result) = app.explorer_lookup_result {
                result.scroll_into_view(
                    app.explorer_lookup_selected,
                    &mut app.explorer_lookup_offset,
                    app.explorer_visible_rows,
                );
            }
        }
    }
}
