//! Mouse event handling.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::{
    AddCommandType, App, InputMode, Popup, PopupFocus, Screen, Section, TxBuilderStep,
};
use crate::ui::common::{centered_rect_min, screen_hints};
use crate::ui::popups::actions_menu_area;
use crate::wallet::{Network, WalletCmd};

use super::explorer::explorer_enter;

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let col = mouse.column;
            let row = mouse.row;

            // Double-click detection
            let is_double_click = if let Some((lc, lr, lt)) = app.last_click {
                lc == col && lr == row && lt.elapsed() < std::time::Duration::from_millis(400)
            } else {
                false
            };
            app.last_click = Some((col, row, std::time::Instant::now()));

            // Handle popup clicks: dismiss on click outside, handle options inside
            if app.popup.is_some() {
                handle_popup_click(app, col, row);
                return;
            }

            // Handle context menu: click inside selects/executes, click outside dismisses
            if app.context_menu.is_some() {
                let cm = app.context_menu_area;
                if row >= cm.y && row < cm.y + cm.height && col >= cm.x && col < cm.x + cm.width {
                    // Click inside the menu — determine which item (1px border on each side)
                    let inner_y = cm.y + 1; // skip top border
                    if row >= inner_y {
                        let item_idx = (row - inner_y) as usize;
                        if let Some(ref menu) = app.context_menu
                            && item_idx < menu.actions.len()
                        {
                            let action = menu.actions[item_idx];
                            let section = menu.section;
                            app.context_menu = None;
                            super::context_menu::execute_context_action(app, section, action);
                            return;
                        }
                    }
                }
                app.context_menu = None;
                return;
            }

            // Network tag click → open network popup
            let nt = app.net_tag_area;
            if row >= nt.y && row < nt.y + nt.height && col >= nt.x && col < nt.x + nt.width {
                app.open_popup(Popup::SwitchNetwork);
                return;
            }

            // Search bar click
            let sb = app.search_bar_area;
            if row >= sb.y && row < sb.y + sb.height && col >= sb.x && col < sb.x + sb.width {
                if !app.search_focused {
                    app.search_focused = true;
                    app.input_mode = InputMode::Editing;
                }
                return;
            }

            // Unfocus search if clicking elsewhere
            if app.search_focused {
                app.search_focused = false;
                app.input_mode = InputMode::Normal;
            }

            // Section overlay: route to old screen-based handlers
            if let Some(section) = app.section_open {
                if is_double_click {
                    // Double-click in overlay opens context menu
                    let is_own = app.exploring.is_none();
                    let actions = crate::app::actions_for(section, is_own);
                    if !actions.is_empty() {
                        app.context_menu = Some(crate::app::ContextMenu {
                            section,
                            actions,
                            selected: 0,
                            anchor_row: row,
                            anchor_col: col,
                        });
                    }
                } else {
                    handle_overlay_click(app, col, row);
                }
                return;
            }

            // Explorer overlay
            if app.exploring.is_some() {
                let ca = app.content_area;
                if row >= ca.y && row < ca.y + ca.height && col >= ca.x && col < ca.x + ca.width {
                    // Content is inside a bordered "Result" block (+1 for top border).
                    // Search results also have a header row + margin (+2 more).
                    let border_offset = if !app.explorer.search_results.is_empty() {
                        3 // border + header + margin
                    } else {
                        1 // border only
                    };
                    let inner_y = ca.y + border_offset;
                    if row >= inner_y {
                        let click_line = app.explorer.lookup_offset + (row - inner_y) as usize;
                        explorer_click_select(app, click_line);
                        if is_double_click {
                            explorer_enter(app);
                        }
                    }
                }
                return;
            }

            // Tx Builder overlay
            if app.tx_builder_open {
                handle_tx_builder_click(app, col, row);
                return;
            }

            // Grid box click
            if let Some(section) = hit_test_grid_box(app, col, row) {
                app.focused_section = section;

                // Click on the expand icon (▶) in the title bar opens the overlay
                if let Some((_, rect)) = app.grid_box_areas.iter().find(|(s, _)| *s == section)
                    && row == rect.y
                    && col >= rect.x + rect.width.saturating_sub(4)
                {
                    let count = super::grid::section_item_count(app, section);
                    if count > 0 {
                        app.section_open = Some(section);
                    }
                    return;
                }

                if is_double_click {
                    // Double-click opens context menu
                    super::grid::open_context_menu_at(app, section, row, col);
                } else {
                    // Single click: select the item in the box
                    click_select_in_box(app, section, col, row);
                }
                return;
            }

            // Check for hint bar clicks
            for (hint_rect, action_id) in &app.hint_areas.clone() {
                if col >= hint_rect.x && col < hint_rect.x + hint_rect.width && row == hint_rect.y {
                    handle_hint_click(app, action_id);
                    return;
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if app.popup.is_some() {
                app.popup_scroll = app.popup_scroll.saturating_sub(1);
            } else if app.section_open.is_some() || app.tx_builder_open {
                scroll_selection(app, -1);
            } else {
                super::grid::scroll_grid_box(app, mouse.column, mouse.row, -1);
            }
        }
        MouseEventKind::ScrollDown => {
            if app.popup.is_some() {
                app.popup_scroll = app.popup_scroll.saturating_add(1);
            } else if app.section_open.is_some() || app.tx_builder_open {
                scroll_selection(app, 1);
            } else {
                super::grid::scroll_grid_box(app, mouse.column, mouse.row, 1);
            }
        }
        MouseEventKind::Moved => {}
        _ => {}
    }
}

/// Hit-test which grid box contains (col, row).
fn hit_test_grid_box(app: &App, col: u16, row: u16) -> Option<Section> {
    for &(section, rect) in &app.grid_box_areas {
        if col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
        {
            return Some(section);
        }
    }
    None
}

/// Select an item within a box based on click row.
fn click_select_in_box(app: &mut App, section: Section, _col: u16, row: u16) {
    // Find box area
    let Some((_, rect)) = app.grid_box_areas.iter().find(|(s, _)| *s == section) else {
        return;
    };
    let rect = *rect;

    // Items start at rect.y + 1 (border top)
    let data_start = rect.y + 1;
    if row < data_start || row >= rect.y + rect.height.saturating_sub(1) {
        return;
    }

    let clicked_row = (row - data_start) as usize;

    match section {
        Section::Coins => {
            let idx = app.coins_offset + clicked_row;
            if idx < app.coins.len() {
                app.coins_selected = idx;
            }
        }
        Section::Objects => {
            let idx = app.objects_offset + clicked_row;
            if idx < app.objects.len() {
                app.objects_selected = idx;
            }
        }
        Section::Staking => {
            let idx = app.stakes_offset + clicked_row;
            if idx < app.stakes.len() {
                app.stakes_selected = idx;
            }
        }
        Section::Transactions => {
            let idx = app.transactions_offset + clicked_row;
            if idx < app.transactions.len() {
                app.transactions_selected = idx;
            }
        }
        Section::Packages => {
            let pkg_count = app.package_indices().len();
            let idx = app.packages_offset + clicked_row;
            if idx < pkg_count {
                app.packages_selected = idx;
            }
        }
    }
}

/// Handle clicks when a section overlay is open (routes to old screen-based click logic).
fn handle_overlay_click(app: &mut App, col: u16, row: u16) {
    let cy = app.content_area.y;
    if row < cy {
        return;
    }

    match app.screen {
        Screen::Coins => {
            let data_start = cy + 3 + 1 + 1 + 1;
            if row >= data_start {
                let idx = app.coins_offset + (row - data_start) as usize;
                if idx < app.coins.len() {
                    app.coins_selected = idx;
                    if is_icon_click(app, col) {
                        app.activate_selected_coin();
                    }
                }
            }
        }
        Screen::Objects => {
            let data_start = cy + 1 + 1 + 1;
            if row >= data_start {
                let idx = app.objects_offset + (row - data_start) as usize;
                if idx < app.objects.len() {
                    app.objects_selected = idx;
                    if is_icon_click(app, col) {
                        app.activate_selected_object();
                    }
                }
            }
        }
        Screen::Transactions => {
            let data_start = cy + 1 + 1;
            if row >= data_start {
                let idx = app.transactions_offset + (row - data_start) as usize;
                if idx < app.transactions.len() {
                    app.transactions_selected = idx;
                    if is_icon_click(app, col) {
                        app.activate_selected_transaction();
                    }
                }
            }
        }
        Screen::Staking => {
            let data_start = cy + 1 + 1;
            if row >= data_start {
                let idx = app.stakes_offset + (row - data_start) as usize;
                if idx < app.stakes.len() {
                    app.stakes_selected = idx;
                }
            }
        }
        Screen::Packages => {
            let data_start = cy + 1 + 1 + 1;
            if row >= data_start {
                let packages = app.package_indices();
                let idx = app.packages_offset + (row - data_start) as usize;
                if idx < packages.len() {
                    app.packages_selected = idx;
                    if is_icon_click(app, col) {
                        app.activate_selected_package();
                    }
                }
            }
        }
        _ => {}
    }
}

/// Handle clicks when the Tx Builder overlay is open.
fn handle_tx_builder_click(app: &mut App, col: u16, row: u16) {
    let cy = app.content_area.y;
    let step_end = cy + 3;
    if row >= cy && row < step_end {
        let mut x = 1u16;
        let last = TxBuilderStep::ALL.len() - 1;
        for (si, step) in TxBuilderStep::ALL.iter().enumerate() {
            let w = 3 + step.title().len() as u16 + 2 + if si < last { 3 } else { 0 };
            if col >= x && col < x + w {
                app.tx.step = TxBuilderStep::ALL[si];
                break;
            }
            x += w;
        }
    } else if row >= step_end {
        match app.tx.step {
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
        Screen::Staking => {
            app.stakes_selected = apply_delta(app.stakes_selected, delta, app.stakes.len());
            App::scroll_into_view(
                app.stakes_selected,
                &mut app.stakes_offset,
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
            if !app.explorer.search_results.is_empty() {
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
            } else if app.explorer.lookup_result.is_some() {
                let result = app.explorer.lookup_result.as_ref().unwrap();
                let total_lines = result.total_visible_lines();
                let new_offset = apply_delta(app.explorer.lookup_offset, delta, total_lines);
                app.explorer.lookup_offset = new_offset;
            }
        }
    }
}

/// Select the explorer item at a given line index (accounting for headings + fields).
fn explorer_click_select(app: &mut App, line_idx: usize) {
    let result = match app.explorer.lookup_result {
        Some(ref r) => r,
        None => {
            // Search results: simple index selection
            if !app.explorer.search_results.is_empty() {
                let idx = line_idx.min(app.explorer.search_results.len().saturating_sub(1));
                app.explorer.search_selected = idx;
            }
            return;
        }
    };

    let sections = result.sections();
    let mut current_line = 0;
    for (si, section) in sections.iter().enumerate() {
        if current_line == line_idx {
            // Clicked on this heading
            app.explorer.lookup_section = si;
            app.explorer.lookup_depth = 0;
            app.explorer.lookup_field_idx = 0;
            return;
        }
        current_line += 1;
        if !section.collapsed {
            for fi in 0..section.fields.len() {
                if current_line == line_idx {
                    app.explorer.lookup_section = si;
                    app.explorer.lookup_depth = 1;
                    app.explorer.lookup_field_idx = fi;
                    return;
                }
                current_line += 1;
            }
        }
    }
}

/// Handle a click on a status bar action hint.
pub(crate) fn handle_hint_click(app: &mut App, action_id: &str) {
    match action_id {
        "open_menu" => {
            app.action_menu_selected = 0;
            app.open_popup(Popup::ActionsMenu);
        }
        "explore" => match app.screen {
            Screen::Coins => app.activate_selected_coin(),
            Screen::Objects => app.activate_selected_object(),
            Screen::Transactions => app.activate_selected_transaction(),
            Screen::Staking => {
                if let Some(stake) = app.stakes.get(app.stakes_selected) {
                    let id = stake.object_id.clone();
                    app.explore_item(id);
                }
            }
            Screen::Packages => app.activate_selected_package(),
            Screen::AddressBook => app.activate_selected_address(),
            Screen::Keys => {
                if let Some(key) = app.keys.get(app.keys_selected) {
                    let addr = key.address.clone();
                    app.explore_item(addr);
                }
            }
            Screen::Explorer => {
                // Explorer "explore" triggers lookup submit (handled by screen key handler)
            }
            _ => {}
        },
        "help" => app.open_popup(Popup::Help),
        "refresh" => app.request_refresh(),
        "network" => app.open_popup(Popup::SwitchNetwork),
        "faucet" => {
            if let Some(key) = app.active_key()
                && let Ok(addr) = iota_sdk::types::Address::from_hex(&key.address)
            {
                app.send_cmd(WalletCmd::RequestFaucet(addr));
            }
        }
        "type_search" => {
            // Coins/Objects: explore the type of the selected item
            match app.screen {
                Screen::Coins => {
                    if let Some(coin) = app.coins.get(app.coins_selected) {
                        let ct = coin.coin_type.clone();
                        app.explore_type(ct);
                    }
                }
                Screen::Objects => {
                    if let Some(obj) = app.objects.get(app.objects_selected) {
                        let tn = obj.type_name.clone();
                        app.explore_type(tn);
                    }
                }
                _ => {}
            }
        }
        "copy" => {
            app.copy_selected();
        }
        "export" => {
            app.export_csv();
        }
        "filter" => match app.screen {
            Screen::Coins => {
                app.coins_filter = Some(String::new());
                app.coins_selected = 0;
                app.coins_offset = 0;
            }
            Screen::Objects => {
                app.objects_filter = Some(String::new());
                app.objects_selected = 0;
                app.objects_offset = 0;
            }
            Screen::Transactions => {
                app.transactions_filter = Some(String::new());
                app.transactions_selected = 0;
                app.transactions_offset = 0;
            }
            _ => {}
        },
        "addr_add" => {
            app.address_edit_field = 0;
            app.address_edit_buffers = [String::new(), String::new(), String::new()];
            app.open_popup(Popup::AddAddress);
            app.start_input("");
        }
        "addr_edit" => {
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
        "addr_delete" => {
            if let Some(user_idx) = app.user_address_index(app.address_selected)
                && user_idx < app.address_book.len()
            {
                app.open_popup(Popup::ConfirmDeleteAddress);
            }
        }
        "iota_name" => {
            app.open_popup(Popup::LookupIotaName);
            app.start_input("");
        }
        "key_activate" => app.activate_selected_key(),
        "key_visible" => {
            if let Some(key) = app.keys.get_mut(app.keys_selected) {
                key.visible = !key.visible;
                app.request_refresh();
            }
        }
        "key_gen" => app.open_popup(Popup::GenerateKey),
        "key_import" => {
            app.open_popup(Popup::ImportKey);
            app.start_input("");
        }
        "key_rename" => {
            if let Some(key_display) = app.keys.get(app.keys_selected) {
                let current = key_display.alias.clone();
                app.open_popup(Popup::RenameKey);
                app.start_input(&current);
            }
        }
        "key_delete" => {
            if !app.keys.is_empty() {
                app.open_popup(Popup::ConfirmDeleteKey);
            }
        }
        "tx_add" => app.open_popup(Popup::AddCommand),
        "tx_delete" => {
            if !app.tx.commands.is_empty() {
                app.tx.commands.remove(app.tx.cmd_selected);
                app.tx.dry_run_dirty = true;
                if app.tx.cmd_selected >= app.tx.commands.len() && app.tx.cmd_selected > 0 {
                    app.tx.cmd_selected -= 1;
                }
            }
        }
        "tx_clear" => {
            if app.tx.commands.is_empty() {
                app.tx.reset();
            } else {
                app.open_popup(Popup::ConfirmClearTx);
            }
        }
        "portfolio" => {
            if app.coins_summary_mode {
                app.coins_summary_mode = false;
            } else if app.show_multiple_owners() {
                app.coins_summary_mode = true;
                app.compute_portfolio_summary();
            }
        }
        "merge" => {
            app.merge_coins_for_selected();
        }
        "split" => {
            if !app.coins.is_empty() {
                app.open_popup(Popup::SplitCoin);
                app.start_input("2");
            }
        }
        "quick_transfer" => {
            if !app.coins.is_empty() {
                app.quick_transfer_field = 0;
                app.quick_transfer_buffers = [String::new(), String::new()];
                app.open_popup(Popup::QuickTransfer);
                app.start_input("");
            }
        }
        "unstake" => {
            if let Some(stake) = app.stakes.get(app.stakes_selected) {
                let staked_id = stake.object_id.clone();
                app.tx.reset();
                app.tx.commands.push(crate::app::PtbCommand::Unstake {
                    staked_iota_id: staked_id,
                });
                app.tx.step = crate::app::TxBuilderStep::Review;
                app.navigate(crate::app::Screen::TxBuilder);
                app.send_cmd(WalletCmd::DryRun {
                    sender_idx: app.tx.sender,
                    commands: app.tx.commands.clone(),
                });
                app.tx.dry_running = true;
                app.tx.dry_run_dirty = false;
            }
        }
        "pkg_browse" => match app.pkg_view {
            crate::app::PackageBrowserView::List => app.browse_selected_package(),
            crate::app::PackageBrowserView::Modules => app.browse_selected_module(),
            crate::app::PackageBrowserView::Functions => app.jump_to_move_call(),
        },
        "pkg_back" => match app.pkg_view {
            crate::app::PackageBrowserView::Functions => {
                app.pkg_view = crate::app::PackageBrowserView::Modules;
            }
            crate::app::PackageBrowserView::Modules => {
                app.pkg_view = crate::app::PackageBrowserView::List;
            }
            _ => {}
        },
        "explorer_search" => {
            app.start_input("");
        }
        _ => {}
    }
}

/// Handle a mouse click when a popup is open.
/// Clicks outside the popup area dismiss it; clicks inside may trigger options.
fn handle_popup_click(app: &mut App, col: u16, row: u16) {
    let area = app.frame_area;

    // Compute the popup area using the same params as the renderer
    let popup_area = match app.popup {
        Some(Popup::Help) => centered_rect_min(70, 80, 50, 24, area),
        Some(Popup::Detail) => centered_rect_min(65, 70, 50, 16, area),
        Some(Popup::AddAddress | Popup::EditAddress) => centered_rect_min(60, 60, 48, 14, area),
        Some(Popup::GenerateKey) => centered_rect_min(50, 40, 36, 11, area),
        Some(Popup::GenerateKeyAlias) => centered_rect_min(50, 30, 40, 8, area),
        Some(Popup::ImportKey) => centered_rect_min(60, 30, 48, 10, area),
        Some(Popup::AddCommand) => centered_rect_min(50, 50, 40, 16, area),
        Some(Popup::AddCommandForm) => centered_rect_min(65, 60, 52, 14, area),
        Some(Popup::RenameKey) => centered_rect_min(50, 30, 40, 8, area),
        Some(Popup::SwitchNetwork) => centered_rect_min(50, 50, 44, 16, area),
        Some(Popup::ConfirmDeleteAddress) => centered_rect_min(55, 40, 44, 10, area),
        Some(Popup::ConfirmDeleteKey) => centered_rect_min(55, 40, 44, 10, area),
        Some(Popup::ConfirmClearTx) => centered_rect_min(55, 40, 44, 10, area),
        Some(Popup::LookupIotaName) => centered_rect_min(60, 30, 48, 10, area),
        Some(Popup::ErrorLog) => centered_rect_min(80, 80, 60, 20, area),
        Some(Popup::ConfirmQuit) => centered_rect_min(50, 30, 40, 7, area),
        Some(Popup::SplitCoin) => centered_rect_min(50, 30, 40, 9, area),
        Some(Popup::MergeCoin) => {
            let h = (app.merge_candidates.len() as u16 + 6).min(area.height - 2);
            centered_rect_min(60, 60, 50, h, area)
        }
        Some(Popup::QuickTransfer) => centered_rect_min(60, 50, 48, 13, area),
        Some(Popup::ObjectTransfer) => centered_rect_min(60, 40, 48, 10, area),
        Some(Popup::ActionsMenu) => actions_menu_area(app, area),
        Some(Popup::Settings) => centered_rect_min(75, 75, 60, 20, area),
        Some(Popup::Welcome) => centered_rect_min(50, 30, 42, 9, area),
        None => return,
    };

    // Click outside popup → dismiss
    if col < popup_area.x
        || col >= popup_area.x + popup_area.width
        || row < popup_area.y
        || row >= popup_area.y + popup_area.height
    {
        app.popup = None;
        app.input_mode = InputMode::Normal;
        app.input_clear();
        return;
    }

    // inner_row: 0 = first line inside border (border is row 0 = popup_area.y)
    let inner_row = row.saturating_sub(popup_area.y + 1) as usize;

    match app.popup {
        Some(Popup::GenerateKey) => {
            // Lines: 0=blank, 1="Select...", 2=blank, 3=Ed25519, 4=Secp256k1, 5=Secp256r1
            let scheme = match inner_row {
                3 => Some("ed25519"),
                4 => Some("secp256k1"),
                5 => Some("secp256r1"),
                _ => None,
            };
            if let Some(scheme) = scheme {
                app.keys_gen_scheme = Some(scheme.to_string());
                let default_alias = format!("key-{}", app.keys.len());
                app.open_popup(Popup::GenerateKeyAlias);
                app.start_input(&default_alias);
            }
        }
        Some(Popup::SwitchNetwork) => {
            // Lines: 0=blank, 1="Select...", 2=blank, 3=Mainnet, 4=Testnet, 5=Devnet
            let network = match inner_row {
                3 => Some(Network::Mainnet),
                4 => Some(Network::Testnet),
                5 => Some(Network::Devnet),
                _ => None,
            };
            if let Some(net) = network {
                app.connected = false;
                app.network_name = format!("{}...", net.name());
                app.loading = true;
                app.send_cmd(WalletCmd::Connect(net));
                app.popup = None;
            }
        }
        Some(Popup::AddCommand) => {
            // Lines: 0=blank, 1="Select...", 2=blank, 3-9 = 7 command types
            let cmd_type = match inner_row {
                3 => Some(AddCommandType::TransferIota),
                4 => Some(AddCommandType::TransferObjects),
                5 => Some(AddCommandType::MoveCall),
                6 => Some(AddCommandType::SplitCoins),
                7 => Some(AddCommandType::MergeCoins),
                8 => Some(AddCommandType::Stake),
                9 => Some(AddCommandType::Unstake),
                _ => None,
            };
            if let Some(ct) = cmd_type {
                let field_count = match ct {
                    AddCommandType::TransferIota => 2,
                    AddCommandType::TransferObjects => 2,
                    AddCommandType::MoveCall => 5,
                    AddCommandType::SplitCoins => 2,
                    AddCommandType::MergeCoins => 2,
                    AddCommandType::Stake => 2,
                    AddCommandType::Unstake => 1,
                };
                app.tx.adding_cmd = Some(ct);
                app.tx.edit_field = 0;
                app.tx.edit_buffers = vec![String::new(); field_count];
                app.tx.multi_values.clear();
                app.open_popup(Popup::AddCommandForm);
                app.start_input("");
            }
        }
        Some(Popup::ConfirmDeleteAddress) => {
            // Lines: 0=blank, 1="Delete?", 2=blank, 3=label, 4=blank, 5=confirm/cancel
            if inner_row == 5 {
                let mid = popup_area.x + popup_area.width / 2;
                if col < mid {
                    // Confirm
                    if let Some(user_idx) = app.user_address_index(app.address_selected)
                        && user_idx < app.address_book.len()
                    {
                        app.address_book.remove(user_idx);
                        let combined_len = app.key_entry_count() + app.address_book.len();
                        if app.address_selected >= combined_len && app.address_selected > 0 {
                            app.address_selected -= 1;
                        }
                        crate::app::save_address_book(&app.address_book);
                    }
                }
                app.popup = None;
            }
        }
        Some(Popup::ConfirmDeleteKey) => {
            // Lines: 0=blank, 1="Delete?", 2=blank, 3=alias, 4=blank, 5=confirm/cancel
            if inner_row == 5 {
                let mid = popup_area.x + popup_area.width / 2;
                if col < mid {
                    // Confirm
                    let idx = app.keys_selected;
                    if idx < app.keys.len() {
                        let removed = app.keys.remove(idx);
                        app.send_cmd(WalletCmd::DeleteKey(idx));
                        if removed.is_active && !app.keys.is_empty() {
                            app.keys[0].is_active = true;
                            app.send_cmd(WalletCmd::SetActiveKey(0));
                            app.request_refresh();
                        }
                        if app.keys_selected >= app.keys.len() && app.keys_selected > 0 {
                            app.keys_selected -= 1;
                        }
                    }
                }
                app.popup = None;
            }
        }
        Some(Popup::ConfirmClearTx) => {
            // Lines: 0=blank, 1="Clear all?", 2=blank, 3=confirm/cancel
            if inner_row == 3 {
                let mid = popup_area.x + popup_area.width / 2;
                if col < mid {
                    app.tx.reset();
                }
                app.popup = None;
            }
        }
        Some(Popup::ConfirmQuit) => {
            // Lines: 0=blank, 1="Quit?", 2=blank, 3=confirm/cancel
            if inner_row == 3 {
                let mid = popup_area.x + popup_area.width / 2;
                if col < mid {
                    app.running = false;
                } else {
                    app.popup = None;
                }
            }
        }
        // Input popups: check for submit/cancel button clicks on the last inner line
        Some(
            Popup::AddAddress
            | Popup::EditAddress
            | Popup::GenerateKeyAlias
            | Popup::ImportKey
            | Popup::RenameKey
            | Popup::LookupIotaName
            | Popup::AddCommandForm
            | Popup::SplitCoin
            | Popup::QuickTransfer
            | Popup::ObjectTransfer,
        ) => {
            let button_row = (popup_area.height.saturating_sub(2)) as usize;
            if inner_row == button_row {
                let inner_col = col.saturating_sub(popup_area.x + 1) as usize;

                // Compute submit button region from the rendered text layout.
                // AddAddress/EditAddress have "  Tab: next  [ Save ]  [ Cancel ]"
                // Others have "  [ Label ]  [ Cancel ]"
                let (submit_start, submit_end) = match app.popup {
                    Some(Popup::AddAddress | Popup::EditAddress) => (13, 21),
                    Some(Popup::GenerateKeyAlias) => (2, 13),
                    Some(Popup::ImportKey | Popup::LookupIotaName) => (2, 12),
                    Some(Popup::RenameKey) => (2, 10),
                    Some(Popup::AddCommandForm) => (2, 9),
                    Some(Popup::SplitCoin) => (2, 11),
                    Some(Popup::QuickTransfer) => (13, 21),
                    Some(Popup::ObjectTransfer) => (13, 21),
                    _ => (2, 14),
                };
                let cancel_start = submit_end + 2;

                if (submit_start..submit_end).contains(&inner_col) {
                    submit_input_popup(app);
                } else if inner_col >= cancel_start {
                    cancel_input_popup(app);
                }
            } else {
                // Click on a field row → focus that field
                click_popup_field(app, inner_row);
            }
        }
        Some(Popup::ActionsMenu) => {
            // Each row inside the border maps directly to a clickable hint
            let hints = screen_hints(app.screen);
            let clickable: Vec<_> = hints.iter().filter(|(_, _, id)| !id.is_empty()).collect();
            if let Some((_, _, action_id)) = clickable.get(inner_row) {
                let id = *action_id;
                app.popup = None;
                handle_hint_click(app, id);
            }
        }
        // Scroll-only popups (Help, Detail, ErrorLog): click inside does nothing
        _ => {}
    }
}

/// Handle clicking on a field row inside an input popup.
/// Maps the inner_row to a field index and switches focus to it.
fn click_popup_field(app: &mut App, inner_row: usize) {
    match app.popup {
        Some(Popup::AddAddress | Popup::EditAddress) => {
            // Layout: row 0=blank, then per field: label, value, blank
            // Field i value is at inner_row 2 + i*3, label at 1 + i*3
            let field = match inner_row {
                1 | 2 => Some(0), // Label
                4 | 5 => Some(1), // Address
                7 | 8 => Some(2), // Notes
                _ => None,
            };
            if let Some(f) = field {
                if app.popup_focus != PopupFocus::Fields {
                    // Save current input to the current field buffer before switching
                    app.address_edit_buffers[app.address_edit_field] = app.input_buffer.clone();
                }
                app.popup_focus = PopupFocus::Fields;
                app.address_edit_field = f;
                let val = app.address_edit_buffers[f].clone();
                app.start_input(&val);
            }
        }
        Some(Popup::AddCommandForm) => {
            // Command form fields: each field takes 2 rows (label + input) + blank
            // Row 0=blank, field i: label at 1+i*3, value at 2+i*3, blank at 3+i*3
            let field_count = app.tx.edit_buffers.len();
            let field = (0..field_count).find(|&i| {
                let label_row = 1 + i * 3;
                inner_row == label_row || inner_row == label_row + 1
            });
            if let Some(f) = field {
                if app.popup_focus != PopupFocus::Fields {
                    app.tx.edit_buffers[app.tx.edit_field] = app.input_buffer.clone();
                }
                app.popup_focus = PopupFocus::Fields;
                app.tx.edit_field = f;
                let val = app.tx.edit_buffers[f].clone();
                app.start_input(&val);
                app.update_autocomplete();
            }
        }
        Some(
            Popup::GenerateKeyAlias
            | Popup::ImportKey
            | Popup::RenameKey
            | Popup::LookupIotaName
            | Popup::SplitCoin,
        ) => {
            // Single-field popups: input is at inner_row 3 (or nearby rows)
            if inner_row <= 4 {
                app.popup_focus = PopupFocus::Fields;
            }
        }
        Some(Popup::QuickTransfer) => {
            // Layout: row 0=blank, then per field: label, value, blank
            let field = match inner_row {
                1 | 2 => Some(0), // Recipient
                4 | 5 => Some(1), // Amount
                _ => None,
            };
            if let Some(f) = field {
                if app.popup_focus != PopupFocus::Fields {
                    app.quick_transfer_buffers[app.quick_transfer_field] = app.input_buffer.clone();
                }
                app.popup_focus = PopupFocus::Fields;
                app.quick_transfer_field = f;
                let val = app.quick_transfer_buffers[f].clone();
                app.start_input(&val);
            }
        }
        Some(Popup::ObjectTransfer) => {
            // Single field: Recipient at rows 3-4
            if inner_row <= 5 {
                app.popup_focus = PopupFocus::Fields;
            }
        }
        _ => {}
    }
}

/// Cancel an input popup (same as pressing Esc).
fn cancel_input_popup(app: &mut App) {
    match app.popup {
        Some(Popup::GenerateKeyAlias) => {
            app.keys_gen_scheme = None;
        }
        Some(Popup::AddCommandForm) => {
            app.tx.adding_cmd = None;
            app.autocomplete.clear();
            app.tx.multi_values.clear();
        }
        _ => {}
    }
    app.popup = None;
    app.input_mode = InputMode::Normal;
    app.input_clear();
}

/// Submit an input popup (same as pressing Enter).
fn submit_input_popup(app: &mut App) {
    match app.popup {
        Some(Popup::AddAddress | Popup::EditAddress) => {
            app.address_edit_buffers[app.address_edit_field] = app.input_buffer.clone();
            let [label, address, notes] = app.address_edit_buffers.clone();
            if !label.is_empty() && !address.is_empty() {
                if app.popup == Some(Popup::AddAddress) {
                    if !address.starts_with("0x") {
                        app.send_cmd(WalletCmd::LookupIotaName {
                            name: address,
                            label,
                            notes,
                        });
                    } else {
                        app.address_book.push(crate::app::AddressEntry {
                            label,
                            address,
                            notes,
                        });
                        crate::app::save_address_book(&app.address_book);
                    }
                } else if let Some(user_idx) = app.user_address_index(app.address_selected) {
                    if let Some(entry) = app.address_book.get_mut(user_idx) {
                        entry.label = label;
                        entry.address = address;
                        entry.notes = notes;
                    }
                    crate::app::save_address_book(&app.address_book);
                }
            }
            app.popup = None;
            app.input_mode = InputMode::Normal;
            app.input_clear();
        }
        Some(Popup::GenerateKeyAlias) => {
            let alias = app.stop_input();
            if let Some(scheme) = app.keys_gen_scheme.take() {
                let alias = if alias.is_empty() {
                    format!("key-{}", app.keys.len())
                } else {
                    alias
                };
                app.send_cmd(WalletCmd::GenerateKey {
                    scheme: scheme.clone(),
                    alias,
                });
            }
            app.popup = None;
        }
        Some(Popup::ImportKey) => {
            let val = app.stop_input();
            if !val.is_empty() {
                let alias = format!("imported-{}", app.keys.len());
                app.send_cmd(WalletCmd::ImportKey {
                    scheme: "ed25519".to_string(),
                    private_key_hex: val,
                    alias,
                });
            }
            app.popup = None;
        }
        Some(Popup::RenameKey) => {
            let new_alias = app.stop_input();
            if !new_alias.is_empty() {
                let idx = app.keys_selected;
                if let Some(k) = app.keys.get_mut(idx) {
                    k.alias = new_alias.clone();
                }
                app.send_cmd(WalletCmd::RenameKey { idx, new_alias });
            }
            app.popup = None;
        }
        Some(Popup::LookupIotaName) => {
            let name = app.stop_input();
            if !name.is_empty() {
                app.send_cmd(WalletCmd::LookupIotaName {
                    name,
                    label: String::new(),
                    notes: String::new(),
                });
            }
            app.popup = None;
        }
        Some(Popup::AddCommandForm) => {
            // Simulate Enter with empty buffer → submit form
            // This mirrors the keyboard handler in command_form.rs
            if app.autocomplete_idx.is_some() {
                app.accept_autocomplete();
            } else {
                app.tx.edit_buffers[app.tx.edit_field] = app.input_buffer.clone();
                // Try to build command
                use crate::event::popup::command_form_build_command;
                if let Some(cmd) = command_form_build_command(app) {
                    app.tx.commands.push(cmd);
                    app.tx.dry_run_dirty = true;
                    app.popup = None;
                    app.tx.adding_cmd = None;
                    app.input_mode = InputMode::Normal;
                    app.input_clear();
                    app.autocomplete.clear();
                    app.autocomplete_idx = None;
                    app.tx.multi_values.clear();
                }
            }
        }
        Some(Popup::SplitCoin) => {
            let val = app.stop_input();
            let n: usize = val.parse().unwrap_or(0);
            app.popup = None;
            app.split_selected_coin(n);
        }
        Some(Popup::QuickTransfer) => {
            app.quick_transfer_buffers[app.quick_transfer_field] = app.input_buffer.clone();
            app.stop_input();
            app.popup = None;
            app.finalize_quick_transfer();
        }
        Some(Popup::ObjectTransfer) => {
            app.stop_input();
            app.popup = None;
            app.finalize_object_transfer();
        }
        _ => {}
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
