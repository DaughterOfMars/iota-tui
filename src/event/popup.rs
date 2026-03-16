//! Keyboard event handling for popup overlays.

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AddCommandType, App, InputMode, Popup, PtbCommand, save_address_book};
use crate::wallet::WalletCmd;

use super::input::handle_input_key;

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
        Some(Popup::AddAddress | Popup::EditAddress) => match key.code {
            KeyCode::Esc => {
                app.popup = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
            }
            KeyCode::Tab => {
                let val = app.input_buffer.clone();
                app.address_edit_buffers[app.address_edit_field] = val;
                app.address_edit_field = (app.address_edit_field + 1) % 3;
                let next_val = app.address_edit_buffers[app.address_edit_field].clone();
                app.start_input(&next_val);
            }
            KeyCode::Enter => {
                app.address_edit_buffers[app.address_edit_field] = app.input_buffer.clone();
                let [label, address, notes] = app.address_edit_buffers.clone();
                if !label.is_empty() && !address.is_empty() {
                    if app.popup == Some(Popup::AddAddress) {
                        // Auto-detect IOTA name: if address doesn't start with 0x, resolve it
                        if !address.starts_with("0x") {
                            app.send_cmd(WalletCmd::LookupIotaName {
                                name: address,
                                label,
                                notes,
                            });
                            app.set_status("Looking up IOTA name...");
                        } else {
                            app.address_book.push(crate::app::AddressEntry {
                                label,
                                address,
                                notes,
                            });
                            save_address_book(&app.address_book);
                            app.set_status("Address added");
                        }
                    } else if let Some(user_idx) = app.user_address_index(app.address_selected) {
                        if let Some(entry) = app.address_book.get_mut(user_idx) {
                            entry.label = label;
                            entry.address = address;
                            entry.notes = notes;
                            app.set_status("Address updated");
                        }
                        save_address_book(&app.address_book);
                    }
                }
                app.popup = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
            }
            _ => handle_input_key(app, key),
        },
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
                app.tx_adding_cmd = Some(ct);
                app.tx_edit_field = 0;
                app.tx_edit_buffers = vec![String::new(); field_count];
                app.tx_multi_values.clear();
                app.open_popup(Popup::AddCommandForm);
                app.start_input("");
            }
        }
        Some(Popup::AddCommandForm) => match key.code {
            KeyCode::Esc => {
                if app.autocomplete_idx.is_some() {
                    app.autocomplete_idx = None;
                } else {
                    app.popup = None;
                    app.tx_adding_cmd = None;
                    app.input_mode = InputMode::Normal;
                    app.input_clear();
                    app.autocomplete.clear();
                    app.tx_multi_values.clear();
                }
            }
            KeyCode::Down if !app.autocomplete.is_empty() => {
                let len = app.autocomplete.len();
                app.autocomplete_idx = Some(match app.autocomplete_idx {
                    None => 0,
                    Some(i) => (i + 1).min(len - 1),
                });
            }
            KeyCode::Up if app.autocomplete_idx.is_some() => {
                app.autocomplete_idx = match app.autocomplete_idx {
                    Some(0) => None,
                    Some(i) => Some(i - 1),
                    None => None,
                };
            }
            KeyCode::Backspace
                if app.input_buffer.is_empty()
                    && app.is_multi_value_field()
                    && !app.tx_multi_values.is_empty() =>
            {
                app.remove_last_multi_value();
            }
            KeyCode::Tab => {
                // Accept autocomplete or manual text for multi-value fields
                if app.accept_autocomplete() {
                    // accepted highlighted suggestion
                } else if !app.autocomplete.is_empty() {
                    app.autocomplete_idx = Some(0);
                    app.accept_autocomplete();
                } else if app.is_multi_value_field() && !app.input_buffer.is_empty() {
                    // Manual entry: add typed text as a value
                    let val = app.input_buffer.clone();
                    app.tx_multi_values.push(val);
                    app.input_buffer.clear();
                    app.input_cursor = 0;
                }

                if !app.is_multi_value_field() {
                    // Advance to next field for single-value fields
                    let val = app.input_buffer.clone();
                    app.tx_edit_buffers[app.tx_edit_field] = val;
                    let count = app.tx_edit_buffers.len();
                    app.tx_edit_field = (app.tx_edit_field + 1) % count;
                    let next_val = app.tx_edit_buffers[app.tx_edit_field].clone();
                    app.start_input(&next_val);
                }
                app.update_autocomplete();
            }
            KeyCode::Enter => {
                if app.autocomplete_idx.is_some() {
                    app.accept_autocomplete();
                } else if app.is_multi_value_field() && !app.input_buffer.is_empty() {
                    // Manual entry: add typed text as a value
                    let val = app.input_buffer.clone();
                    app.tx_multi_values.push(val);
                    app.input_buffer.clear();
                    app.input_cursor = 0;
                    app.update_autocomplete();
                } else {
                    app.tx_edit_buffers[app.tx_edit_field] = app.input_buffer.clone();
                    if let Some(cmd) = build_command_from_form(app) {
                        app.tx_commands.push(cmd);
                        app.tx_dry_run_dirty = true;
                        app.set_status("Command added");
                        app.popup = None;
                        app.tx_adding_cmd = None;
                        app.input_mode = InputMode::Normal;
                        app.input_clear();
                        app.autocomplete.clear();
                        app.autocomplete_idx = None;
                        app.tx_multi_values.clear();
                    } else {
                        app.set_status("Fill in all required fields");
                    }
                }
            }
            _ => {
                handle_input_key(app, key);
                app.update_autocomplete();
            }
        },
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
        Some(Popup::GenerateKeyAlias) => match key.code {
            KeyCode::Esc => {
                app.popup = None;
                app.keys_gen_scheme = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
            }
            KeyCode::Enter => {
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
                    app.set_status(format!("Generating {} keypair...", scheme));
                }
                app.popup = None;
            }
            _ => handle_input_key(app, key),
        },
        Some(Popup::ImportKey) => match key.code {
            KeyCode::Esc => {
                app.popup = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
            }
            KeyCode::Enter => {
                let val = app.stop_input();
                if !val.is_empty() {
                    let alias = format!("imported-{}", app.keys.len());
                    app.send_cmd(WalletCmd::ImportKey {
                        scheme: "ed25519".to_string(),
                        private_key_hex: val,
                        alias,
                    });
                    app.set_status("Importing key...");
                }
                app.popup = None;
            }
            _ => handle_input_key(app, key),
        },
        Some(Popup::RenameKey) => match key.code {
            KeyCode::Esc => {
                app.popup = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
            }
            KeyCode::Enter => {
                let new_alias = app.stop_input();
                if !new_alias.is_empty() {
                    let idx = app.keys_selected;
                    if let Some(k) = app.keys.get_mut(idx) {
                        k.alias = new_alias.clone();
                    }
                    app.send_cmd(WalletCmd::RenameKey { idx, new_alias });
                    app.set_status("Key renamed");
                }
                app.popup = None;
            }
            _ => handle_input_key(app, key),
        },
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
                app.set_status("Switching network...");
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
                    app.set_status("Address removed");
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
                    app.set_status("Key removed");
                }
                app.popup = None;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                app.popup = None;
            }
            _ => {}
        },
        Some(Popup::LookupIotaName) => match key.code {
            KeyCode::Esc => {
                app.popup = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
            }
            KeyCode::Enter => {
                let name = app.stop_input();
                if !name.is_empty() {
                    app.send_cmd(WalletCmd::LookupIotaName {
                        name,
                        label: String::new(),
                        notes: String::new(),
                    });
                    app.set_status("Looking up IOTA name...");
                }
                app.popup = None;
            }
            _ => handle_input_key(app, key),
        },
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
        Some(Popup::ConfirmQuit) => match key.code {
            KeyCode::Enter | KeyCode::Char('y') => {
                app.running = false;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
                app.popup = None;
            }
            _ => {}
        },
        None => {}
    }
}

/// Parse the form buffers into a PtbCommand based on the selected command type.
/// Address fields are resolved through aliases (key aliases + address book labels).
fn build_command_from_form(app: &App) -> Option<PtbCommand> {
    let ct = app.tx_adding_cmd?;
    let bufs = &app.tx_edit_buffers;
    match ct {
        AddCommandType::TransferIota => {
            let recipient = app.resolve_address(bufs.first()?);
            let amount = bufs.get(1)?.clone();
            if recipient.is_empty() || amount.is_empty() {
                return None;
            }
            Some(PtbCommand::TransferIota { recipient, amount })
        }
        AddCommandType::TransferObjects => {
            let recipient = app.resolve_address(bufs.first()?);
            if recipient.is_empty() || app.tx_multi_values.is_empty() {
                return None;
            }
            Some(PtbCommand::TransferObjects {
                recipient,
                object_ids: app.tx_multi_values.clone(),
            })
        }
        AddCommandType::MoveCall => {
            let package = bufs.first()?.clone();
            let module = bufs.get(1)?.clone();
            let function = bufs.get(2)?.clone();
            let type_args_str = bufs.get(3).cloned().unwrap_or_default();
            let args_str = bufs.get(4).cloned().unwrap_or_default();
            if package.is_empty() || module.is_empty() || function.is_empty() {
                return None;
            }
            let type_args: Vec<String> = if type_args_str.is_empty() {
                vec![]
            } else {
                type_args_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            };
            let args: Vec<String> = if args_str.is_empty() {
                vec![]
            } else {
                args_str.split(',').map(|s| s.trim().to_string()).collect()
            };
            Some(PtbCommand::MoveCall {
                package,
                module,
                function,
                type_args,
                args,
            })
        }
        AddCommandType::SplitCoins => {
            let coin = bufs.first()?.clone();
            let amounts_str = bufs.get(1)?.clone();
            if coin.is_empty() || amounts_str.is_empty() {
                return None;
            }
            let amounts: Vec<String> = amounts_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            Some(PtbCommand::SplitCoins { coin, amounts })
        }
        AddCommandType::MergeCoins => {
            let primary = bufs.first()?.clone();
            if primary.is_empty() || app.tx_multi_values.is_empty() {
                return None;
            }
            Some(PtbCommand::MergeCoins {
                primary,
                sources: app.tx_multi_values.clone(),
            })
        }
        AddCommandType::Stake => {
            let amount = bufs.first()?.clone();
            let validator = app.resolve_address(bufs.get(1)?);
            if amount.is_empty() || validator.is_empty() {
                return None;
            }
            Some(PtbCommand::Stake { amount, validator })
        }
        AddCommandType::Unstake => {
            let staked_iota_id = bufs.first()?.clone();
            if staked_iota_id.is_empty() {
                return None;
            }
            Some(PtbCommand::Unstake { staked_iota_id })
        }
    }
}
