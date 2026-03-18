//! Mouse event handling.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, ExplorerView, InputMode, Screen, TxBuilderStep};

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

            // Content area starts after tab bar (1) + status bar (1) + border (1) + header row (1)
            let content_start = 5u16;
            if row >= content_start {
                let visual_index = (row - content_start) as usize;
                match app.screen {
                    Screen::Coins => {
                        let idx = app.coins_offset + visual_index;
                        if idx < app.coins.len() {
                            app.coins_selected = idx;
                        }
                    }
                    Screen::Objects => {
                        let idx = app.objects_offset + visual_index;
                        if idx < app.objects.len() {
                            app.objects_selected = idx;
                        }
                    }
                    Screen::Transactions => {
                        let idx = app.transactions_offset + visual_index;
                        if idx < app.transactions.len() {
                            app.transactions_selected = idx;
                        }
                    }
                    Screen::Packages => {}
                    Screen::AddressBook => {
                        let idx = app.address_offset + visual_index;
                        let combined_len = app.key_entry_count() + app.address_book.len();
                        if idx < combined_len {
                            app.address_selected = idx;
                        }
                    }
                    Screen::Keys => {
                        let idx = app.keys_offset + visual_index;
                        if idx < app.keys.len() {
                            app.keys_selected = idx;
                        }
                    }
                    Screen::TxBuilder => {
                        if row <= 4 {
                            let step_width = col as usize / 20;
                            if let Some(&step) = TxBuilderStep::ALL.get(step_width) {
                                app.tx_step = step;
                            }
                        } else {
                            let step_row = (row.saturating_sub(5)) as usize;
                            match app.tx_step {
                                TxBuilderStep::SelectSender => {
                                    if step_row < app.keys.len() {
                                        if app.tx_sender != step_row {
                                            app.tx_dry_run_dirty = true;
                                        }
                                        app.tx_sender = step_row;
                                    }
                                }
                                TxBuilderStep::EditCommands => {
                                    if step_row >= 2 && step_row - 2 < app.tx_commands.len() {
                                        app.tx_cmd_selected = step_row - 2;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Screen::Explorer => {
                        // Explorer has a sub-tab bar (3 rows) before its content.
                        // Row content_start-2 is the text row inside the sub-tab block.
                        let sub_tab_row = content_start.saturating_sub(2);
                        if row == sub_tab_row {
                            // Determine which sub-tab was clicked from column position
                            let mut x = 2u16; // inside border
                            for &view in ExplorerView::ALL.iter() {
                                let w = view.title().len() as u16 + 3; // " title " + space
                                if col >= x && col < x + w {
                                    app.explorer_view = view;
                                    break;
                                }
                                x += w;
                            }
                        } else if row >= content_start + 3 {
                            // Content data rows (after sub-tab bar + border + header + margin)
                            let data_index =
                                (row - content_start - 3) as usize;
                            match app.explorer_view {
                                ExplorerView::Checkpoints => {
                                    let idx =
                                        app.explorer_checkpoints_offset + data_index;
                                    if idx < app.explorer_checkpoints.len() {
                                        app.explorer_checkpoints_selected = idx;
                                    }
                                }
                                ExplorerView::Validators => {
                                    let idx =
                                        app.explorer_validators_offset + data_index;
                                    if idx < app.explorer_validators.len() {
                                        app.explorer_validators_selected = idx;
                                    }
                                }
                                ExplorerView::Lookup => {
                                    if !app.explorer_search_results.is_empty() {
                                        let idx =
                                            app.explorer_search_offset + data_index;
                                        if idx < app.explorer_search_results.len() {
                                            app.explorer_search_selected = idx;
                                        }
                                    } else if let Some(ref result) =
                                        app.explorer_lookup_result
                                    {
                                        let total = result.total_fields();
                                        let idx =
                                            app.explorer_lookup_offset + data_index;
                                        if idx < total {
                                            app.explorer_lookup_selected = idx;
                                        }
                                    }
                                }
                                ExplorerView::Overview => {}
                            }
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
            App::scroll_into_view(app.coins_selected, &mut app.coins_offset, app.content_visible_rows);
        }
        Screen::Objects => {
            app.objects_selected = apply_delta(app.objects_selected, delta, app.objects.len());
            App::scroll_into_view(app.objects_selected, &mut app.objects_offset, app.content_visible_rows);
        }
        Screen::Transactions => {
            app.transactions_selected =
                apply_delta(app.transactions_selected, delta, app.transactions.len());
            App::scroll_into_view(app.transactions_selected, &mut app.transactions_offset, app.content_visible_rows);
        }
        Screen::AddressBook => {
            let combined_len = app.key_entry_count() + app.address_book.len();
            app.address_selected = apply_delta(app.address_selected, delta, combined_len);
            App::scroll_into_view(app.address_selected, &mut app.address_offset, app.content_visible_rows);
        }
        Screen::Keys => {
            app.keys_selected = apply_delta(app.keys_selected, delta, app.keys.len());
            App::scroll_into_view(app.keys_selected, &mut app.keys_offset, app.content_visible_rows);
        }
        Screen::TxBuilder => match app.tx_step {
            TxBuilderStep::SelectSender => {
                let old = app.tx_sender;
                app.tx_sender = apply_delta(app.tx_sender, delta, app.keys.len());
                if app.tx_sender != old {
                    app.tx_dry_run_dirty = true;
                }
            }
            TxBuilderStep::EditCommands => {
                app.tx_cmd_selected =
                    apply_delta(app.tx_cmd_selected, delta, app.tx_commands.len());
            }
            _ => {}
        },
        Screen::Packages => {}
        Screen::Explorer => {
            // Explorer sub-view scroll: checkpoints, validators, search results
            use crate::app::ExplorerView;
            match app.explorer_view {
                ExplorerView::Checkpoints => {
                    app.explorer_checkpoints_selected = apply_delta(
                        app.explorer_checkpoints_selected,
                        delta,
                        app.explorer_checkpoints.len(),
                    );
                    App::scroll_into_view(
                        app.explorer_checkpoints_selected,
                        &mut app.explorer_checkpoints_offset,
                        app.content_visible_rows,
                    );
                }
                ExplorerView::Validators => {
                    app.explorer_validators_selected = apply_delta(
                        app.explorer_validators_selected,
                        delta,
                        app.explorer_validators.len(),
                    );
                    App::scroll_into_view(
                        app.explorer_validators_selected,
                        &mut app.explorer_validators_offset,
                        app.content_visible_rows,
                    );
                }
                ExplorerView::Lookup if !app.explorer_search_results.is_empty() => {
                    app.explorer_search_selected = apply_delta(
                        app.explorer_search_selected,
                        delta,
                        app.explorer_search_results.len(),
                    );
                    App::scroll_into_view(
                        app.explorer_search_selected,
                        &mut app.explorer_search_offset,
                        app.content_visible_rows,
                    );
                }
                ExplorerView::Lookup if app.explorer_lookup_result.is_some() => {
                    let total = app
                        .explorer_lookup_result
                        .as_ref()
                        .map(|r| r.total_fields())
                        .unwrap_or(0);
                    app.explorer_lookup_selected =
                        apply_delta(app.explorer_lookup_selected, delta, total);
                    App::scroll_into_view(
                        app.explorer_lookup_selected,
                        &mut app.explorer_lookup_offset,
                        app.content_visible_rows,
                    );
                }
                _ => {}
            }
        }
    }
}

fn apply_delta(current: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = current as i32 + delta;
    new.clamp(0, (len as i32) - 1) as usize
}
