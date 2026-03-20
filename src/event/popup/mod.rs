//! Keyboard event handling for popup overlays.

mod command_form;
pub(crate) use command_form::build_command_from_form as command_form_build_command;

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AddCommandType, App, InputMode, Popup, PopupFocus, save_address_book};
use crate::ui::common::screen_hints;
use crate::wallet::WalletCmd;

use super::input::handle_input_key;
use super::mouse::handle_hint_click;

/// Handle key events when focus is on Submit or Cancel buttons.
/// Returns `true` if the key was consumed (caller should not process further).
/// Handle Left/Right arrow keys to move between Submit and Cancel buttons.
/// Returns `true` if the key was consumed. Does NOT handle Tab (popup-specific
/// Tab handlers need to manage field-to-button transitions themselves).
pub(super) fn handle_button_focus_key(app: &mut App, key: KeyEvent) -> bool {
    if app.popup_focus == PopupFocus::Fields {
        return false;
    }
    match key.code {
        KeyCode::Right => {
            if app.popup_focus == PopupFocus::Submit {
                app.popup_focus = PopupFocus::Cancel;
            }
            true
        }
        KeyCode::Left => {
            if app.popup_focus == PopupFocus::Cancel {
                app.popup_focus = PopupFocus::Submit;
            }
            true
        }
        _ => false,
    }
}

/// Dispatch keyboard events when a popup is open.
pub fn handle_popup_key(app: &mut App, key: KeyEvent) {
    match app.popup {
        Some(Popup::Help) => match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => app.popup = None,
            KeyCode::Down => {
                app.popup_scroll = app.popup_scroll.saturating_add(1);
            }
            KeyCode::Up => {
                app.popup_scroll = app.popup_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                app.popup_scroll = app.popup_scroll.saturating_add(5);
            }
            KeyCode::PageUp => {
                app.popup_scroll = app.popup_scroll.saturating_sub(5);
            }
            _ => {}
        },
        Some(Popup::Detail) => match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => app.popup = None,
            KeyCode::Down => {
                app.popup_scroll = app.popup_scroll.saturating_add(1);
            }
            KeyCode::Up => {
                app.popup_scroll = app.popup_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                app.popup_scroll = app.popup_scroll.saturating_add(5);
            }
            KeyCode::PageUp => {
                app.popup_scroll = app.popup_scroll.saturating_sub(5);
            }
            _ => {}
        },
        Some(Popup::AddAddress | Popup::EditAddress) => {
            if handle_button_focus_key(app, key) {
                return;
            }
            match key.code {
                KeyCode::Esc => {
                    if app.popup_focus != PopupFocus::Fields {
                        app.popup_focus = PopupFocus::Fields;
                    } else {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                }
                KeyCode::Tab => match app.popup_focus {
                    PopupFocus::Fields => {
                        let val = app.input_buffer.clone();
                        app.address_edit_buffers[app.address_edit_field] = val;
                        if app.address_edit_field < 2 {
                            app.address_edit_field += 1;
                            let next_val = app.address_edit_buffers[app.address_edit_field].clone();
                            app.start_input(&next_val);
                        } else {
                            app.popup_focus = PopupFocus::Submit;
                        }
                    }
                    PopupFocus::Submit => app.popup_focus = PopupFocus::Cancel,
                    PopupFocus::Cancel => {
                        app.address_edit_field = 0;
                        let val = app.address_edit_buffers[0].clone();
                        app.start_input(&val);
                        app.popup_focus = PopupFocus::Fields;
                    }
                },
                KeyCode::BackTab => match app.popup_focus {
                    PopupFocus::Fields => {
                        if app.address_edit_field > 0 {
                            let val = app.input_buffer.clone();
                            app.address_edit_buffers[app.address_edit_field] = val;
                            app.address_edit_field -= 1;
                            let next_val = app.address_edit_buffers[app.address_edit_field].clone();
                            app.start_input(&next_val);
                        } else {
                            app.popup_focus = PopupFocus::Cancel;
                        }
                    }
                    PopupFocus::Submit => {
                        let val = app.address_edit_buffers[2].clone();
                        app.address_edit_field = 2;
                        app.start_input(&val);
                        app.popup_focus = PopupFocus::Fields;
                    }
                    PopupFocus::Cancel => app.popup_focus = PopupFocus::Submit,
                },
                KeyCode::Enter => match app.popup_focus {
                    PopupFocus::Fields => {
                        // Advance to next field, or to Submit on last field
                        let val = app.input_buffer.clone();
                        app.address_edit_buffers[app.address_edit_field] = val;
                        if app.address_edit_field < 2 {
                            app.address_edit_field += 1;
                            let next_val = app.address_edit_buffers[app.address_edit_field].clone();
                            app.start_input(&next_val);
                        } else {
                            app.popup_focus = PopupFocus::Submit;
                        }
                    }
                    PopupFocus::Submit => {
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
                                    save_address_book(&app.address_book);
                                }
                            } else if let Some(user_idx) =
                                app.user_address_index(app.address_selected)
                            {
                                if let Some(entry) = app.address_book.get_mut(user_idx) {
                                    entry.label = label;
                                    entry.address = address;
                                    entry.notes = notes;
                                }
                                save_address_book(&app.address_book);
                            }
                        }
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                    PopupFocus::Cancel => {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                },
                _ => {
                    if app.popup_focus == PopupFocus::Fields {
                        handle_input_key(app, key);
                    }
                }
            }
        }
        Some(Popup::AddCommand) => {
            let cmd_type = match key.code {
                KeyCode::Char('1') | KeyCode::Char('t') => Some(AddCommandType::TransferIota),
                KeyCode::Char('2') | KeyCode::Char('o') => Some(AddCommandType::TransferObjects),
                KeyCode::Char('3') | KeyCode::Char('m') => Some(AddCommandType::MoveCall),
                KeyCode::Char('4') | KeyCode::Char('s') => Some(AddCommandType::SplitCoins),
                KeyCode::Char('5') | KeyCode::Char('r') => Some(AddCommandType::MergeCoins),
                KeyCode::Char('6') | KeyCode::Char('k') => Some(AddCommandType::Stake),
                KeyCode::Char('7') | KeyCode::Char('u') => Some(AddCommandType::Unstake),
                KeyCode::Esc => {
                    app.popup = None;
                    None
                }
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
        Some(Popup::AddCommandForm) => command_form::handle_command_form_key(app, key),
        Some(Popup::GenerateKey) => {
            let scheme = match key.code {
                KeyCode::Char('1') | KeyCode::Char('e') => Some("ed25519"),
                KeyCode::Char('2') | KeyCode::Char('s') => Some("secp256k1"),
                KeyCode::Char('3') | KeyCode::Char('r') => Some("secp256r1"),
                KeyCode::Esc => {
                    app.popup = None;
                    None
                }
                _ => None,
            };
            if let Some(scheme) = scheme {
                app.keys_gen_scheme = Some(scheme.to_string());
                let default_alias = format!("key-{}", app.keys.len());
                app.open_popup(Popup::GenerateKeyAlias);
                app.start_input(&default_alias);
            }
        }
        Some(Popup::GenerateKeyAlias) => {
            if handle_button_focus_key(app, key) {
                return;
            }
            match key.code {
                KeyCode::Esc => {
                    if app.popup_focus != PopupFocus::Fields {
                        app.popup_focus = PopupFocus::Fields;
                    } else {
                        app.popup = None;
                        app.keys_gen_scheme = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                }
                KeyCode::Tab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Submit,
                        PopupFocus::Submit => PopupFocus::Cancel,
                        PopupFocus::Cancel => PopupFocus::Fields,
                    };
                }
                KeyCode::BackTab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Cancel,
                        PopupFocus::Submit => PopupFocus::Fields,
                        PopupFocus::Cancel => PopupFocus::Submit,
                    };
                }
                KeyCode::Enter => match app.popup_focus {
                    PopupFocus::Fields => {
                        app.popup_focus = PopupFocus::Submit;
                    }
                    PopupFocus::Submit => {
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
                    PopupFocus::Cancel => {
                        app.popup = None;
                        app.keys_gen_scheme = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                },
                _ => {
                    if app.popup_focus == PopupFocus::Fields {
                        handle_input_key(app, key);
                    }
                }
            }
        }
        Some(Popup::ImportKey) => {
            if handle_button_focus_key(app, key) {
                return;
            }
            match key.code {
                KeyCode::Esc => {
                    if app.popup_focus != PopupFocus::Fields {
                        app.popup_focus = PopupFocus::Fields;
                    } else {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                }
                KeyCode::Tab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Submit,
                        PopupFocus::Submit => PopupFocus::Cancel,
                        PopupFocus::Cancel => PopupFocus::Fields,
                    };
                }
                KeyCode::BackTab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Cancel,
                        PopupFocus::Submit => PopupFocus::Fields,
                        PopupFocus::Cancel => PopupFocus::Submit,
                    };
                }
                KeyCode::Enter => match app.popup_focus {
                    PopupFocus::Fields => {
                        app.popup_focus = PopupFocus::Submit;
                    }
                    PopupFocus::Submit => {
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
                    PopupFocus::Cancel => {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                },
                _ => {
                    if app.popup_focus == PopupFocus::Fields {
                        handle_input_key(app, key);
                    }
                }
            }
        }
        Some(Popup::RenameKey) => {
            if handle_button_focus_key(app, key) {
                return;
            }
            match key.code {
                KeyCode::Esc => {
                    if app.popup_focus != PopupFocus::Fields {
                        app.popup_focus = PopupFocus::Fields;
                    } else {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                }
                KeyCode::Tab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Submit,
                        PopupFocus::Submit => PopupFocus::Cancel,
                        PopupFocus::Cancel => PopupFocus::Fields,
                    };
                }
                KeyCode::BackTab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Cancel,
                        PopupFocus::Submit => PopupFocus::Fields,
                        PopupFocus::Cancel => PopupFocus::Submit,
                    };
                }
                KeyCode::Enter => match app.popup_focus {
                    PopupFocus::Fields => {
                        app.popup_focus = PopupFocus::Submit;
                    }
                    PopupFocus::Submit => {
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
                    PopupFocus::Cancel => {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                },
                _ => {
                    if app.popup_focus == PopupFocus::Fields {
                        handle_input_key(app, key);
                    }
                }
            }
        }
        Some(Popup::SwitchNetwork) => {
            use crate::wallet::Network;
            let network = match key.code {
                KeyCode::Char('1') | KeyCode::Char('m') => Some(Network::Mainnet),
                KeyCode::Char('2') | KeyCode::Char('t') => Some(Network::Testnet),
                KeyCode::Char('3') | KeyCode::Char('d') => Some(Network::Devnet),
                KeyCode::Esc => {
                    app.popup = None;
                    None
                }
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
        Some(Popup::ConfirmDeleteAddress) => match key.code {
            KeyCode::Enter | KeyCode::Char('y') => {
                if let Some(user_idx) = app.user_address_index(app.address_selected)
                    && user_idx < app.address_book.len()
                {
                    app.address_book.remove(user_idx);
                    let combined_len = app.key_entry_count() + app.address_book.len();
                    if app.address_selected >= combined_len && app.address_selected > 0 {
                        app.address_selected -= 1;
                    }
                    save_address_book(&app.address_book);
                }
                app.popup = None;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                app.popup = None;
            }
            _ => {}
        },
        Some(Popup::ConfirmDeleteKey) => match key.code {
            KeyCode::Enter | KeyCode::Char('y') => {
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
                app.popup = None;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                app.popup = None;
            }
            _ => {}
        },
        Some(Popup::LookupIotaName) => {
            if handle_button_focus_key(app, key) {
                return;
            }
            match key.code {
                KeyCode::Esc => {
                    if app.popup_focus != PopupFocus::Fields {
                        app.popup_focus = PopupFocus::Fields;
                    } else {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                }
                KeyCode::Tab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Submit,
                        PopupFocus::Submit => PopupFocus::Cancel,
                        PopupFocus::Cancel => PopupFocus::Fields,
                    };
                }
                KeyCode::BackTab => {
                    app.popup_focus = match app.popup_focus {
                        PopupFocus::Fields => PopupFocus::Cancel,
                        PopupFocus::Submit => PopupFocus::Fields,
                        PopupFocus::Cancel => PopupFocus::Submit,
                    };
                }
                KeyCode::Enter => match app.popup_focus {
                    PopupFocus::Fields => {
                        app.popup_focus = PopupFocus::Submit;
                    }
                    PopupFocus::Submit => {
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
                    PopupFocus::Cancel => {
                        app.popup = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                    }
                },
                _ => {
                    if app.popup_focus == PopupFocus::Fields {
                        handle_input_key(app, key);
                    }
                }
            }
        }
        Some(Popup::ErrorLog) => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => app.popup = None,
            KeyCode::Down => {
                app.popup_scroll = app.popup_scroll.saturating_add(1);
            }
            KeyCode::Up => {
                app.popup_scroll = app.popup_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                app.popup_scroll = app.popup_scroll.saturating_add(5);
            }
            KeyCode::PageUp => {
                app.popup_scroll = app.popup_scroll.saturating_sub(5);
            }
            _ => {}
        },
        Some(Popup::ConfirmClearTx) => match key.code {
            KeyCode::Enter | KeyCode::Char('y') => {
                app.tx.reset();
                app.popup = None;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                app.popup = None;
            }
            _ => {}
        },
        Some(Popup::ConfirmQuit) => match key.code {
            KeyCode::Enter | KeyCode::Char('y') => {
                app.running = false;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
                app.popup = None;
            }
            _ => {}
        },
        Some(Popup::ActionsMenu) => {
            let hints = screen_hints(app.screen);
            let clickable: Vec<_> = hints.iter().filter(|(_, _, id)| !id.is_empty()).collect();
            match key.code {
                KeyCode::Esc | KeyCode::Char('.') => {
                    app.popup = None;
                }
                KeyCode::Up => {
                    if app.action_menu_selected > 0 {
                        app.action_menu_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.action_menu_selected + 1 < clickable.len() {
                        app.action_menu_selected += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some((_, _, action_id)) = clickable.get(app.action_menu_selected) {
                        let id = *action_id;
                        app.popup = None;
                        handle_hint_click(app, id);
                    }
                }
                KeyCode::Char(c) => {
                    // Direct shortcut: match single-char key labels
                    let key_str = c.to_string();
                    if let Some((_, _, action_id)) =
                        clickable.iter().find(|(label, _, _)| *label == key_str)
                    {
                        let id = *action_id;
                        app.popup = None;
                        handle_hint_click(app, id);
                    }
                }
                _ => {}
            }
        }
        None => {}
    }
}
