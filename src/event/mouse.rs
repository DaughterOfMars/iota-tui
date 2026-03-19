//! Mouse event handling.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, ExplorerView, InputMode, LookupAction, Popup, Screen, TxBuilderStep};
use crate::wallet::WalletCmd;

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let col = mouse.column;
            let row = mouse.row;

            // Dismiss popups on click outside (simplistic)
            if app.popup.is_some() {
                app.popup = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
                return;
            }

            for (i, area) in app.tab_areas.iter().enumerate() {
                if col >= area.x
                    && col < area.x + area.width
                    && row >= area.y
                    && row < area.y + area.height
                    && let Some(&screen) = Screen::ALL.get(i)
                {
                    app.navigate(screen);
                    return;
                }
            }

            // Content area Y position (set each frame by the UI draw function)
            let cy = app.content_area_y;
            if row < cy {
                return;
            }

            match app.screen {
                // Coins: summary(3) + table with border(1)+header(1)+margin(1) = data at cy+6
                Screen::Coins => {
                    let data_start = cy + 3 + 1 + 1 + 1;
                    if row >= data_start {
                        let idx = app.coins_offset + (row - data_start) as usize;
                        if idx < app.coins.len() {
                            app.coins_selected = idx;
                            if is_icon_click(app, col) {
                                let id = app.coins[idx].object_id.clone();
                                app.explore_item(id);
                            }
                        }
                    }
                }
                // Objects: border(1)+header(1)+margin(1) = data at cy+3
                Screen::Objects => {
                    let data_start = cy + 1 + 1 + 1;
                    if row >= data_start {
                        let idx = app.objects_offset + (row - data_start) as usize;
                        if idx < app.objects.len() {
                            app.objects_selected = idx;
                            if is_icon_click(app, col) {
                                let id = app.objects[idx].object_id.clone();
                                app.explore_item(id);
                            }
                        }
                    }
                }
                // Transactions: border(1)+header(1), NO margin = data at cy+2
                Screen::Transactions => {
                    let data_start = cy + 1 + 1;
                    if row >= data_start {
                        let idx = app.transactions_offset + (row - data_start) as usize;
                        if idx < app.transactions.len() {
                            app.transactions_selected = idx;
                            if is_icon_click(app, col) {
                                let digest = app.transactions[idx].digest.clone();
                                app.explore_item(digest);
                            }
                        }
                    }
                }
                // Packages: border(1)+header(1)+margin(1) = data at cy+3
                Screen::Packages => {
                    let data_start = cy + 1 + 1 + 1;
                    if row >= data_start {
                        let packages = app.package_indices();
                        let idx = app.packages_offset + (row - data_start) as usize;
                        if idx < packages.len() {
                            app.packages_selected = idx;
                            if is_icon_click(app, col) {
                                let id = app.objects[packages[idx]].object_id.clone();
                                app.explore_item(id);
                            }
                        }
                    }
                }
                // AddressBook: border(1)+header(1)+margin(1) = data at cy+3
                Screen::AddressBook => {
                    let data_start = cy + 1 + 1 + 1;
                    if row >= data_start {
                        let idx = app.address_offset + (row - data_start) as usize;
                        let combined_len = app.key_entry_count() + app.address_book.len();
                        if idx < combined_len {
                            app.address_selected = idx;
                            if is_icon_click(app, col) {
                                let combined = app.combined_address_book();
                                if let Some(entry) = combined.get(idx) {
                                    let addr = entry.address.clone();
                                    app.explore_item(addr);
                                }
                            }
                        }
                    }
                }
                // Keys: border(1)+header(1)+margin(1) = data at cy+3
                Screen::Keys => {
                    let data_start = cy + 1 + 1 + 1;
                    if row >= data_start {
                        let idx = app.keys_offset + (row - data_start) as usize;
                        if idx < app.keys.len() {
                            app.keys_selected = idx;
                            if is_icon_click(app, col) {
                                // Activate key (same as Enter)
                                for (i, k) in app.keys.iter_mut().enumerate() {
                                    k.is_active = i == idx;
                                }
                                app.send_cmd(WalletCmd::SetActiveKey(idx));
                                app.request_refresh();
                            }
                        }
                    }
                }
                Screen::TxBuilder => {
                    // Step indicator: 3-row block at cy..cy+3, text on row cy+1
                    let step_end = cy + 3;
                    if row >= cy && row < step_end {
                        // Click on step indicator — determine which step from column
                        // Rendered: border(1) then per step: " N " (3) + " Title " (len+2) + " > " (3)
                        // Last step has no " > " separator
                        let mut x = 1u16; // inside left border
                        let last = TxBuilderStep::ALL.len() - 1;
                        for (si, step) in TxBuilderStep::ALL.iter().enumerate() {
                            // " N " = 3, " Title " = title.len() + 2
                            let w =
                                3 + step.title().len() as u16 + 2 + if si < last { 3 } else { 0 };
                            if col >= x && col < x + w {
                                app.tx.step = TxBuilderStep::ALL[si];
                                break;
                            }
                            x += w;
                        }
                    } else if row >= step_end {
                        // Content area below step indicator
                        match app.tx.step {
                            // SelectSender uses List: border(1) then items
                            TxBuilderStep::SelectSender => {
                                let data_start = step_end + 1;
                                if row >= data_start {
                                    let idx = (row - data_start) as usize;
                                    if idx < app.keys.len() {
                                        if app.tx.sender != idx {
                                            app.tx.dry_run_dirty = true;
                                        }
                                        app.tx.sender = idx;
                                    }
                                }
                            }
                            // EditCommands uses Table: border(1)+header(1)+margin(1)
                            TxBuilderStep::EditCommands => {
                                let data_start = step_end + 1 + 1 + 1;
                                if row >= data_start {
                                    let idx = (row - data_start) as usize;
                                    if idx < app.tx.commands.len() {
                                        app.tx.cmd_selected = idx;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Screen::Explorer => {
                    // Sub-tab bar: 3-row block at cy..cy+3, text on row cy+1
                    let sub_tab_end = cy + 3;
                    if row >= cy && row < sub_tab_end {
                        // Click on sub-tab text row
                        if row == cy + 1 {
                            let mut x = 2u16;
                            for &view in ExplorerView::ALL.iter() {
                                let w = view.title().len() as u16 + 3;
                                if col >= x && col < x + w {
                                    if app.explorer.view != view {
                                        app.explorer.view = view;
                                        app.refresh_explorer();
                                    }
                                    break;
                                }
                                x += w;
                            }
                        }
                    } else if row >= sub_tab_end {
                        // Content below sub-tabs; each sub-view has its own layout
                        match app.explorer.view {
                            // Checkpoints table: border(1)+header(1)+margin(1) = +3
                            // (may have a filter row +1 before the table)
                            ExplorerView::Checkpoints => {
                                let filter_rows = if app.explorer.checkpoints_filter.is_some() {
                                    1u16
                                } else {
                                    0
                                };
                                let data_start = sub_tab_end + filter_rows + 1 + 1 + 1;
                                if row >= data_start {
                                    let idx = app.explorer.checkpoints_offset
                                        + (row - data_start) as usize;
                                    if idx < app.explorer.checkpoints.len() {
                                        app.explorer.checkpoints_selected = idx;
                                        if is_icon_click(app, col)
                                            && !app.explorer.checkpoints.is_empty()
                                        {
                                            app.open_popup(Popup::Detail);
                                        }
                                    }
                                }
                            }
                            // Validators table: border(1)+header(1)+margin(1) = +3
                            ExplorerView::Validators => {
                                let data_start = sub_tab_end + 1 + 1 + 1;
                                if row >= data_start {
                                    let idx = app.explorer.validators_offset
                                        + (row - data_start) as usize;
                                    if idx < app.explorer.validators.len() {
                                        app.explorer.validators_selected = idx;
                                        if is_icon_click(app, col) {
                                            let addr = app.explorer.validators[idx].address.clone();
                                            app.explore_item(addr);
                                        }
                                    }
                                }
                            }
                            // Lookup: search input(3) + result block
                            ExplorerView::Lookup => {
                                let result_start = sub_tab_end + 3;
                                if !app.explorer.search_results.is_empty() {
                                    // Search results table: border(1)+header(1)+margin(1)
                                    let data_start = result_start + 1 + 1 + 1;
                                    if row >= data_start {
                                        let idx = app.explorer.search_offset
                                            + (row - data_start) as usize;
                                        if idx < app.explorer.search_results.len() {
                                            app.explorer.search_selected = idx;
                                            if is_icon_click(app, col) {
                                                let id = app.explorer.search_results[idx]
                                                    .object_id
                                                    .clone();
                                                app.explorer.search_results.clear();
                                                app.explore_item(id);
                                            }
                                        }
                                    }
                                } else if let Some(ref result) = app.explorer.lookup_result {
                                    // Lookup result: border(1) then content lines
                                    let data_start = result_start + 1;
                                    if row >= data_start {
                                        // explorer_lookup_offset is a line index
                                        let abs_line = app.explorer.lookup_offset
                                            + (row - data_start) as usize;
                                        // Convert line index to field index (skip headers)
                                        if let Some(field_idx) = result.line_to_field(abs_line)
                                            && field_idx < result.total_fields()
                                        {
                                            app.explorer.lookup_selected = field_idx;
                                            // Activate on click if field has an action
                                            if let Some(field) = result.field_at(field_idx) {
                                                match &field.action {
                                                    Some(LookupAction::Explore(val)) => {
                                                        let val = val.clone();
                                                        app.explore_item(val);
                                                    }
                                                    Some(LookupAction::TypeSearch(val)) => {
                                                        let val = val.clone();
                                                        app.explore_type(val);
                                                    }
                                                    None => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            ExplorerView::Overview => {}
                        }
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if app.popup.is_some() {
                app.popup_scroll = app.popup_scroll.saturating_sub(1);
            } else {
                scroll_selection(app, -1);
            }
        }
        MouseEventKind::ScrollDown => {
            if app.popup.is_some() {
                app.popup_scroll = app.popup_scroll.saturating_add(1);
            } else {
                scroll_selection(app, 1);
            }
        }
        _ => {}
    }
}

pub fn scroll_selection(app: &mut App, delta: i32) {
    match app.screen {
        Screen::Coins => {
            app.coins_selected = apply_delta(app.coins_selected, delta, app.coins.len());
            App::scroll_into_view(
                app.coins_selected,
                &mut app.coins_offset,
                app.content_visible_rows,
            );
        }
        Screen::Objects => {
            app.objects_selected = apply_delta(app.objects_selected, delta, app.objects.len());
            App::scroll_into_view(
                app.objects_selected,
                &mut app.objects_offset,
                app.content_visible_rows,
            );
        }
        Screen::Transactions => {
            app.transactions_selected =
                apply_delta(app.transactions_selected, delta, app.transactions.len());
            App::scroll_into_view(
                app.transactions_selected,
                &mut app.transactions_offset,
                app.content_visible_rows,
            );
        }
        Screen::AddressBook => {
            let combined_len = app.key_entry_count() + app.address_book.len();
            app.address_selected = apply_delta(app.address_selected, delta, combined_len);
            App::scroll_into_view(
                app.address_selected,
                &mut app.address_offset,
                app.content_visible_rows,
            );
        }
        Screen::Keys => {
            app.keys_selected = apply_delta(app.keys_selected, delta, app.keys.len());
            App::scroll_into_view(
                app.keys_selected,
                &mut app.keys_offset,
                app.content_visible_rows,
            );
        }
        Screen::TxBuilder => match app.tx.step {
            TxBuilderStep::SelectSender => {
                let old = app.tx.sender;
                app.tx.sender = apply_delta(app.tx.sender, delta, app.keys.len());
                if app.tx.sender != old {
                    app.tx.dry_run_dirty = true;
                }
            }
            TxBuilderStep::EditCommands => {
                app.tx.cmd_selected =
                    apply_delta(app.tx.cmd_selected, delta, app.tx.commands.len());
            }
            _ => {}
        },
        Screen::Packages => {
            let packages = app.package_indices();
            app.packages_selected = apply_delta(app.packages_selected, delta, packages.len());
            App::scroll_into_view(
                app.packages_selected,
                &mut app.packages_offset,
                app.content_visible_rows,
            );
        }
        Screen::Explorer => {
            // Explorer sub-view scroll: checkpoints, validators, search results
            use crate::app::ExplorerView;
            match app.explorer.view {
                ExplorerView::Checkpoints => {
                    app.explorer.checkpoints_selected = apply_delta(
                        app.explorer.checkpoints_selected,
                        delta,
                        app.explorer.checkpoints.len(),
                    );
                    App::scroll_into_view(
                        app.explorer.checkpoints_selected,
                        &mut app.explorer.checkpoints_offset,
                        app.explorer.visible_rows,
                    );
                }
                ExplorerView::Validators => {
                    app.explorer.validators_selected = apply_delta(
                        app.explorer.validators_selected,
                        delta,
                        app.explorer.validators.len(),
                    );
                    App::scroll_into_view(
                        app.explorer.validators_selected,
                        &mut app.explorer.validators_offset,
                        app.explorer.visible_rows,
                    );
                }
                ExplorerView::Lookup if !app.explorer.search_results.is_empty() => {
                    app.explorer.search_selected = apply_delta(
                        app.explorer.search_selected,
                        delta,
                        app.explorer.search_results.len(),
                    );
                    App::scroll_into_view(
                        app.explorer.search_selected,
                        &mut app.explorer.search_offset,
                        app.explorer.visible_rows,
                    );
                }
                ExplorerView::Lookup if app.explorer.lookup_result.is_some() => {
                    let result = app.explorer.lookup_result.as_ref().unwrap();
                    let total = result.total_fields();
                    app.explorer.lookup_selected =
                        apply_delta(app.explorer.lookup_selected, delta, total);
                    let visible = app.explorer.visible_rows;
                    app.explorer
                        .lookup_result
                        .as_ref()
                        .unwrap()
                        .scroll_into_view(
                            app.explorer.lookup_selected,
                            &mut app.explorer.lookup_offset,
                            visible,
                        );
                }
                _ => {}
            }
        }
    }
}

/// Check whether a click column falls in the icon (⏎) column.
/// The icon column is 2 chars wide, positioned as the last column inside the table border.
fn is_icon_click(app: &App, col: u16) -> bool {
    let area = app.content_area;
    // Right border is at area.x + area.width - 1, icon column is 2 chars before that
    let icon_start = area.x + area.width.saturating_sub(3);
    col >= icon_start && col < area.x + area.width.saturating_sub(1)
}

fn apply_delta(current: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = current as i32 + delta;
    new.clamp(0, (len as i32) - 1) as usize
}
