//! Explorer screen keyboard event handler.

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, ExplorerView, InputMode, LookupAction, Popup};
use crate::wallet::WalletCmd;

use super::input::handle_input_key;
use super::nav::ListNav;

/// Determine whether a query string is a lookup (address/object/tx digest)
/// or a type search. Lookups start with `0x` (hex IDs) or look like base58
/// transaction digests (alphanumeric, 32-44 chars, no `::` separator).
fn is_lookup_query(query: &str) -> bool {
    if query.starts_with("0x") {
        return true;
    }
    // Type strings contain `::` (e.g. `0x2::coin::Coin<0x2::iota::IOTA>`)
    if query.contains("::") {
        return false;
    }
    // Base58 transaction digests: alphanumeric, typically 32-44 chars
    let len = query.len();
    (32..=44).contains(&len) && query.chars().all(|c| c.is_alphanumeric())
}

pub fn handle_explorer_key(app: &mut App, key: KeyEvent) {
    // When editing (lookup input), handle text input
    if app.input_mode == InputMode::Editing {
        match key.code {
            KeyCode::Enter => {
                let query = app.stop_input();
                if !query.is_empty() {
                    if is_lookup_query(&query) {
                        app.explorer.search_mode = false;
                        app.explorer.lookup_query = Some(query.clone());
                        app.explorer.lookup_address = Some(query.clone());
                        app.send_cmd(WalletCmd::LookupAddress(query));
                    } else {
                        app.explorer.search_mode = true;
                        app.explorer.search_type = query.clone();
                        app.explorer.search_cursors.clear();
                        app.explorer.search_has_next = false;
                        app.explorer.search_cursor = None;
                        app.send_cmd(WalletCmd::SearchObjectsByType {
                            type_filter: query,
                            cursor: None,
                        });
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
            let idx = app.explorer.view.index();
            if idx > 0 {
                app.explorer.view = ExplorerView::ALL[idx - 1];
                app.refresh_explorer();
            }
            return;
        }
        KeyCode::Right => {
            let idx = app.explorer.view.index();
            if idx + 1 < ExplorerView::ALL.len() {
                app.explorer.view = ExplorerView::ALL[idx + 1];
                app.refresh_explorer();
            }
            return;
        }
        _ => {}
    }

    match app.explorer.view {
        ExplorerView::Overview => {}
        ExplorerView::Checkpoints => {
            // If filtering, handle text input
            if app.explorer.checkpoints_filter.is_some() {
                match key.code {
                    KeyCode::Esc => {
                        app.explorer.checkpoints_filter = None;
                        app.explorer.checkpoints_selected = 0;
                        app.explorer.checkpoints_offset = 0;
                    }
                    KeyCode::Enter => {
                        // Keep filter active but stop editing — just deselect input
                    }
                    KeyCode::Backspace => {
                        if let Some(ref mut q) = app.explorer.checkpoints_filter {
                            q.pop();
                            if q.is_empty() {
                                app.explorer.checkpoints_filter = None;
                            }
                        }
                        app.explorer.checkpoints_selected = 0;
                        app.explorer.checkpoints_offset = 0;
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        if let Some(ref mut q) = app.explorer.checkpoints_filter {
                            q.push(c);
                        }
                        app.explorer.checkpoints_selected = 0;
                        app.explorer.checkpoints_offset = 0;
                    }
                    KeyCode::Up => {
                        if app.explorer.checkpoints_selected > 0 {
                            app.explorer.checkpoints_selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let len = app.explorer.filtered_checkpoints().len();
                        if app.explorer.checkpoints_selected + 1 < len {
                            app.explorer.checkpoints_selected += 1;
                        }
                    }
                    _ => {}
                }
                let len = app.explorer.filtered_checkpoints().len();
                if app.explorer.checkpoints_selected >= len {
                    app.explorer.checkpoints_selected = len.saturating_sub(1);
                }
                App::scroll_into_view(
                    app.explorer.checkpoints_selected,
                    &mut app.explorer.checkpoints_offset,
                    app.explorer.visible_rows,
                );
                return;
            }

            let len = app.explorer.filtered_checkpoints().len();
            let mut nav = ListNav {
                selected: &mut app.explorer.checkpoints_selected,
                offset: &mut app.explorer.checkpoints_offset,
                len,
                visible_rows: app.explorer.visible_rows,
            };
            if nav.handle_key(key.code) {
                return;
            }
            match key.code {
                KeyCode::Enter => {
                    if !app.explorer.checkpoints.is_empty() {
                        app.open_popup(Popup::Detail);
                    }
                }
                KeyCode::Char('s') => {
                    app.explorer.checkpoints_sort_asc = !app.explorer.checkpoints_sort_asc;
                    app.explorer.checkpoints_selected = 0;
                    app.explorer.checkpoints_offset = 0;
                }
                KeyCode::Char('/') => {
                    app.explorer.checkpoints_filter = Some(String::new());
                    app.explorer.checkpoints_selected = 0;
                    app.explorer.checkpoints_offset = 0;
                }
                KeyCode::Char(']') if app.explorer.checkpoints_has_next => {
                    app.explorer
                        .checkpoints_cursors
                        .push(app.explorer.checkpoints_cursor.clone());
                    app.explorer.checkpoints_page += 1;
                    let cursor = app.explorer.checkpoints_cursor.clone();
                    app.send_cmd(WalletCmd::RefreshCheckpoints { cursor });
                }
                KeyCode::Char('[') if !app.explorer.checkpoints_cursors.is_empty() => {
                    let prev = app.explorer.checkpoints_cursors.pop().flatten();
                    app.explorer.checkpoints_page = app.explorer.checkpoints_page.saturating_sub(1);
                    app.send_cmd(WalletCmd::RefreshCheckpoints { cursor: prev });
                }
                _ => {}
            }
        }
        ExplorerView::Validators => {
            let mut nav = ListNav {
                selected: &mut app.explorer.validators_selected,
                offset: &mut app.explorer.validators_offset,
                len: app.explorer.validators.len(),
                visible_rows: app.explorer.visible_rows,
            };
            if nav.handle_key(key.code) {
                return;
            }
            if key.code == KeyCode::Enter
                && let Some(v) = app
                    .explorer
                    .validators
                    .get(app.explorer.validators_selected)
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
                        .explorer
                        .search_results
                        .get(app.explorer.search_selected)
                    {
                        let id = obj.object_id.clone();
                        app.explorer.search_results.clear();
                        app.explore_item(id);
                        return;
                    }
                    // If lookup result is showing, follow the selected field's action
                    if let Some(ref result) = app.explorer.lookup_result {
                        if let Some(field) = result.field_at(app.explorer.lookup_selected) {
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
                    app.start_input("");
                }
                KeyCode::Esc => {
                    app.explorer.lookup_result = None;
                    app.explorer.search_results.clear();
                    app.explorer.search_selected = 0;
                    app.explorer.lookup_selected = 0;
                    app.explorer.lookup_offset = 0;
                    app.explorer.search_has_next = false;
                    app.explorer.search_cursor = None;
                    app.explorer.search_cursors.clear();
                }
                KeyCode::Up => {
                    if !app.explorer.search_results.is_empty() {
                        if app.explorer.search_selected > 0 {
                            app.explorer.search_selected -= 1;
                        }
                    } else if app.explorer.lookup_result.is_some()
                        && app.explorer.lookup_selected > 0
                    {
                        app.explorer.lookup_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if !app.explorer.search_results.is_empty() {
                        if app.explorer.search_selected + 1 < app.explorer.search_results.len() {
                            app.explorer.search_selected += 1;
                        }
                    } else if let Some(ref result) = app.explorer.lookup_result {
                        let total = result.total_fields();
                        if app.explorer.lookup_selected + 1 < total {
                            app.explorer.lookup_selected += 1;
                        }
                    }
                }
                KeyCode::Home => {
                    app.explorer.search_selected = 0;
                    app.explorer.lookup_selected = 0;
                }
                KeyCode::End => {
                    if !app.explorer.search_results.is_empty() {
                        app.explorer.search_selected =
                            app.explorer.search_results.len().saturating_sub(1);
                    } else if let Some(ref result) = app.explorer.lookup_result {
                        app.explorer.lookup_selected = result.total_fields().saturating_sub(1);
                    }
                }
                KeyCode::Char(']')
                    if !app.explorer.search_results.is_empty() && app.explorer.search_has_next =>
                {
                    // Save current cursor for going back
                    app.explorer
                        .search_cursors
                        .push(app.explorer.search_cursor.clone());
                    // Fetch next page using end_cursor from last response
                    let cursor = app.explorer.search_cursor.clone();
                    let type_filter = app.explorer.search_type.clone();
                    app.send_cmd(WalletCmd::SearchObjectsByType {
                        type_filter,
                        cursor,
                    });
                }
                KeyCode::Char('[')
                    if !app.explorer.search_results.is_empty()
                        && !app.explorer.search_cursors.is_empty() =>
                {
                    // Pop the previous cursor to go back
                    let prev_cursor = app.explorer.search_cursors.pop().flatten();
                    let type_filter = app.explorer.search_type.clone();
                    app.send_cmd(WalletCmd::SearchObjectsByType {
                        type_filter,
                        cursor: prev_cursor,
                    });
                }
                // Address lookup pagination: next page
                KeyCode::Char(']')
                    if app.explorer.search_results.is_empty()
                        && app.explorer.lookup_address.is_some()
                        && matches!(
                            app.explorer.lookup_result,
                            Some(crate::app::LookupResult::Address { .. })
                        )
                        && (app.explorer.lookup_obj_has_next
                            || app.explorer.lookup_tx_has_next) =>
                {
                    app.explorer
                        .lookup_obj_cursors
                        .push(app.explorer.lookup_obj_cursor.clone());
                    app.explorer
                        .lookup_tx_cursors
                        .push(app.explorer.lookup_tx_cursor.clone());
                    app.explorer.lookup_obj_page += 1;
                    app.explorer.lookup_tx_page += 1;
                    let address = app.explorer.lookup_address.clone().unwrap();
                    let obj_cursor = app.explorer.lookup_obj_cursor.clone();
                    let tx_cursor = app.explorer.lookup_tx_cursor.clone();
                    app.send_cmd(WalletCmd::LookupAddressPage {
                        address,
                        obj_cursor,
                        tx_cursor,
                    });
                }
                // Address lookup pagination: prev page
                KeyCode::Char('[')
                    if app.explorer.search_results.is_empty()
                        && app.explorer.lookup_address.is_some()
                        && matches!(
                            app.explorer.lookup_result,
                            Some(crate::app::LookupResult::Address { .. })
                        )
                        && !app.explorer.lookup_obj_cursors.is_empty() =>
                {
                    let prev_obj = app.explorer.lookup_obj_cursors.pop().flatten();
                    let prev_tx = app.explorer.lookup_tx_cursors.pop().flatten();
                    app.explorer.lookup_obj_page = app.explorer.lookup_obj_page.saturating_sub(1);
                    app.explorer.lookup_tx_page = app.explorer.lookup_tx_page.saturating_sub(1);
                    let address = app.explorer.lookup_address.clone().unwrap();
                    app.send_cmd(WalletCmd::LookupAddressPage {
                        address,
                        obj_cursor: prev_obj,
                        tx_cursor: prev_tx,
                    });
                }
                _ => {}
            }
            if !app.explorer.search_results.is_empty() {
                App::scroll_into_view(
                    app.explorer.search_selected,
                    &mut app.explorer.search_offset,
                    app.explorer.visible_rows,
                );
            } else if let Some(ref result) = app.explorer.lookup_result {
                result.scroll_into_view(
                    app.explorer.lookup_selected,
                    &mut app.explorer.lookup_offset,
                    app.explorer.visible_rows,
                );
            }
        }
    }
}
