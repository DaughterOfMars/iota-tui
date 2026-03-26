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
                    // If lookup result is showing, handle tree navigation
                    if let Some(ref mut result) = app.explorer.lookup_result {
                        let sections = result.sections_mut();
                        if app.explorer.lookup_depth == 0 {
                            // On a heading: toggle collapsed
                            if let Some(s) = sections.get_mut(app.explorer.lookup_section) {
                                s.collapsed = !s.collapsed;
                            }
                        } else if let Some(section) = sections.get(app.explorer.lookup_section) {
                            // On a field: follow action
                            if let Some(field) = section.fields.get(app.explorer.lookup_field_idx) {
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
                        }
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
                    app.explorer.lookup_section = 0;
                    app.explorer.lookup_depth = 0;
                    app.explorer.lookup_field_idx = 0;
                    app.explorer.search_has_next = false;
                    app.explorer.search_cursor = None;
                    app.explorer.search_cursors.clear();
                }
                KeyCode::Up => {
                    if !app.explorer.search_results.is_empty() {
                        if app.explorer.search_selected > 0 {
                            app.explorer.search_selected -= 1;
                        }
                    } else if app.explorer.lookup_result.is_some() {
                        lookup_cursor_up(app);
                    }
                }
                KeyCode::Down => {
                    if !app.explorer.search_results.is_empty() {
                        if app.explorer.search_selected + 1 < app.explorer.search_results.len() {
                            app.explorer.search_selected += 1;
                        }
                    } else if app.explorer.lookup_result.is_some() {
                        lookup_cursor_down(app);
                    }
                }
                KeyCode::Left => {
                    if app.explorer.lookup_result.is_some() {
                        lookup_cursor_left(app);
                    }
                }
                KeyCode::Right => {
                    if app.explorer.lookup_result.is_some() {
                        lookup_cursor_right(app);
                    }
                }
                KeyCode::Home => {
                    app.explorer.search_selected = 0;
                    app.explorer.lookup_section = 0;
                    app.explorer.lookup_depth = 0;
                    app.explorer.lookup_field_idx = 0;
                }
                KeyCode::End => {
                    if !app.explorer.search_results.is_empty() {
                        app.explorer.search_selected =
                            app.explorer.search_results.len().saturating_sub(1);
                    } else if let Some(ref result) = app.explorer.lookup_result {
                        let n = result.sections().len();
                        if n > 0 {
                            app.explorer.lookup_section = n - 1;
                            app.explorer.lookup_depth = 0;
                            app.explorer.lookup_field_idx = 0;
                        }
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
                result.scroll_cursor_into_view(
                    app.explorer.lookup_section,
                    app.explorer.lookup_depth,
                    app.explorer.lookup_field_idx,
                    &mut app.explorer.lookup_offset,
                    app.explorer.visible_rows,
                );
            }
        }
    }
}

/// Move cursor up in the lookup tree.
fn lookup_cursor_up(app: &mut App) {
    let result = match app.explorer.lookup_result {
        Some(ref r) => r,
        None => return,
    };
    let sections = result.sections();
    if sections.is_empty() {
        return;
    }

    if app.explorer.lookup_depth == 1 {
        // Inside a section's fields
        if app.explorer.lookup_field_idx > 0 {
            app.explorer.lookup_field_idx -= 1;
        } else {
            // At first field — go back to heading
            app.explorer.lookup_depth = 0;
        }
    } else {
        // On a heading — move to previous section
        if app.explorer.lookup_section > 0 {
            app.explorer.lookup_section -= 1;
            // If previous section is expanded and has fields, land on its last field
            let prev = &sections[app.explorer.lookup_section];
            if !prev.collapsed && !prev.fields.is_empty() {
                app.explorer.lookup_depth = 1;
                app.explorer.lookup_field_idx = prev.fields.len() - 1;
            }
        }
    }
}

/// Move cursor down in the lookup tree.
fn lookup_cursor_down(app: &mut App) {
    let result = match app.explorer.lookup_result {
        Some(ref r) => r,
        None => return,
    };
    let sections = result.sections();
    if sections.is_empty() {
        return;
    }

    if app.explorer.lookup_depth == 1 {
        // Inside a section's fields
        let section = &sections[app.explorer.lookup_section];
        if app.explorer.lookup_field_idx + 1 < section.fields.len() {
            app.explorer.lookup_field_idx += 1;
        } else {
            // Past last field — move to next section heading
            if app.explorer.lookup_section + 1 < sections.len() {
                app.explorer.lookup_section += 1;
                app.explorer.lookup_depth = 0;
                app.explorer.lookup_field_idx = 0;
            }
        }
    } else {
        // On a heading
        let section = &sections[app.explorer.lookup_section];
        if !section.collapsed && !section.fields.is_empty() {
            // Expanded with fields — step into first field
            app.explorer.lookup_depth = 1;
            app.explorer.lookup_field_idx = 0;
        } else {
            // Collapsed or empty — move to next section heading
            if app.explorer.lookup_section + 1 < sections.len() {
                app.explorer.lookup_section += 1;
            }
        }
    }
}

/// Left: collapse section or jump to heading.
fn lookup_cursor_left(app: &mut App) {
    if app.explorer.lookup_depth == 1 {
        // On a field — jump back to section heading
        app.explorer.lookup_depth = 0;
        app.explorer.lookup_field_idx = 0;
    } else if let Some(ref mut result) = app.explorer.lookup_result {
        // On a heading — collapse it
        if let Some(s) = result.sections_mut().get_mut(app.explorer.lookup_section) {
            s.collapsed = true;
        }
    }
}

/// Right: expand section or step into fields.
fn lookup_cursor_right(app: &mut App) {
    if let Some(ref mut result) = app.explorer.lookup_result {
        let sections = result.sections_mut();
        if let Some(s) = sections.get_mut(app.explorer.lookup_section)
            && app.explorer.lookup_depth == 0
        {
            if s.collapsed {
                // Expand the section
                s.collapsed = false;
            } else if !s.fields.is_empty() {
                // Already expanded — step into first field
                app.explorer.lookup_depth = 1;
                app.explorer.lookup_field_idx = 0;
            }
        }
    }
}
