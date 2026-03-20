//! Mouse event handling.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::{
    AddCommandType, App, ExplorerView, InputMode, LookupAction, Popup, PopupFocus, Screen,
    TxBuilderStep,
};
use crate::ui::common::{centered_rect_min, screen_hints};
use crate::ui::popups::actions_menu_area;
use crate::wallet::{Network, WalletCmd};

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let col = mouse.column;
            let row = mouse.row;

            // Handle popup clicks: dismiss on click outside, handle options inside
            if app.popup.is_some() {
                handle_popup_click(app, col, row);
                return;
            }

            for (i, area) in app.sidebar_areas.iter().enumerate() {
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

            // Check for status bar hint clicks
            for (hint_rect, action_id) in &app.hint_areas.clone() {
                if col >= hint_rect.x && col < hint_rect.x + hint_rect.width && row == hint_rect.y {
                    handle_hint_click(app, action_id);
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
                                app.activate_selected_coin();
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
                                app.activate_selected_object();
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
                                app.activate_selected_transaction();
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
                                app.activate_selected_package();
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
                                app.activate_selected_address();
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
                            if is_icon_click(app, col)
                                && let Some(key) = app.keys.get(idx)
                            {
                                let addr = key.address.clone();
                                app.explore_item(addr);
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
                    } else if app.explorer.pagination_row_y > 0
                        && row == app.explorer.pagination_row_y
                    {
                        // Click on pagination row
                        handle_pagination_click(app, col);
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
        Some(Popup::SwitchNetwork) => centered_rect_min(50, 40, 36, 12, area),
        Some(Popup::ConfirmDeleteAddress) => centered_rect_min(55, 40, 44, 10, area),
        Some(Popup::ConfirmDeleteKey) => centered_rect_min(55, 40, 44, 10, area),
        Some(Popup::ConfirmClearTx) => centered_rect_min(55, 40, 44, 10, area),
        Some(Popup::LookupIotaName) => centered_rect_min(60, 30, 48, 10, area),
        Some(Popup::ErrorLog) => centered_rect_min(80, 80, 60, 20, area),
        Some(Popup::ConfirmQuit) => centered_rect_min(50, 30, 40, 7, area),
        Some(Popup::SplitCoin) => centered_rect_min(50, 30, 40, 9, area),
        Some(Popup::QuickTransfer) => centered_rect_min(60, 50, 48, 13, area),
        Some(Popup::ActionsMenu) => actions_menu_area(app, area),
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
            | Popup::QuickTransfer,
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
        _ => {}
    }
}

/// Handle clicks on the pagination row in Explorer views.
fn handle_pagination_click(app: &mut App, col: u16) {
    // Button layout: "  [ ◀ Prev ]  [ Next ▶ ]"
    // "  " (2) + "[ ◀ Prev ]" (10) + "  " (2) + "[ Next ▶ ]" (10)
    let x = app.content_area.x;
    let rel = col.saturating_sub(x) as usize;

    // Prev button: cols 2..12, Next button: cols 14..24
    // But if only next (no prev), next starts at col 2
    let is_prev_click;
    let is_next_click;

    match app.explorer.view {
        ExplorerView::Checkpoints => {
            let has_prev = !app.explorer.checkpoints_cursors.is_empty();
            let has_next = app.explorer.checkpoints_has_next;
            (is_prev_click, is_next_click) = pagination_hit(rel, has_prev, has_next);

            if is_prev_click {
                let prev = app.explorer.checkpoints_cursors.pop().flatten();
                app.explorer.checkpoints_page = app.explorer.checkpoints_page.saturating_sub(1);
                app.send_cmd(WalletCmd::RefreshCheckpoints { cursor: prev });
            } else if is_next_click {
                app.explorer
                    .checkpoints_cursors
                    .push(app.explorer.checkpoints_cursor.clone());
                app.explorer.checkpoints_page += 1;
                let cursor = app.explorer.checkpoints_cursor.clone();
                app.send_cmd(WalletCmd::RefreshCheckpoints { cursor });
            }
        }
        ExplorerView::Lookup if !app.explorer.search_results.is_empty() => {
            let has_prev = !app.explorer.search_cursors.is_empty();
            let has_next = app.explorer.search_has_next;
            (is_prev_click, is_next_click) = pagination_hit(rel, has_prev, has_next);

            if is_prev_click {
                let prev_cursor = app.explorer.search_cursors.pop().flatten();
                let type_filter = app.explorer.search_type.clone();
                app.send_cmd(WalletCmd::SearchObjectsByType {
                    type_filter,
                    cursor: prev_cursor,
                });
            } else if is_next_click {
                app.explorer
                    .search_cursors
                    .push(app.explorer.search_cursor.clone());
                let cursor = app.explorer.search_cursor.clone();
                let type_filter = app.explorer.search_type.clone();
                app.send_cmd(WalletCmd::SearchObjectsByType {
                    type_filter,
                    cursor,
                });
            }
        }
        ExplorerView::Lookup if app.explorer.lookup_address.is_some() => {
            let has_prev = !app.explorer.lookup_obj_cursors.is_empty();
            let has_next = app.explorer.lookup_obj_has_next || app.explorer.lookup_tx_has_next;
            (is_prev_click, is_next_click) = pagination_hit(rel, has_prev, has_next);

            if is_prev_click {
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
            } else if is_next_click {
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
        }
        _ => {}
    }
}

/// Determine if a click at `rel` column hits Prev or Next button.
fn pagination_hit(rel: usize, has_prev: bool, has_next: bool) -> (bool, bool) {
    // "  [ ◀ Prev ]  [ Next ▶ ]"
    // Prev button occupies cols 2..12, Next starts at 14..24
    // If no prev, Next starts at 2..12
    if has_prev && has_next {
        ((2..12).contains(&rel), (14..24).contains(&rel))
    } else if has_prev {
        ((2..12).contains(&rel), false)
    } else if has_next {
        (false, (2..12).contains(&rel))
    } else {
        (false, false)
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
