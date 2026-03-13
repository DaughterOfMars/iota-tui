use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::app::{
    AddCommandType, App, InputMode, Popup, PtbCommand, Screen, TxBuilderStep, save_address_book,
};
use crate::wallet::WalletCmd;

pub fn handle_event(app: &mut App, ev: Event) {
    match ev {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        Event::Resize(_, _) => {}
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.running = false;
        return;
    }

    if app.popup.is_some() {
        handle_popup_key(app, key);
        return;
    }

    if app.input_mode == InputMode::Editing {
        handle_input_key(app, key);
        return;
    }

    match key.code {
        KeyCode::Char('q') => {
            app.open_popup(Popup::ConfirmQuit);
            return;
        }
        KeyCode::Char('?') => {
            app.open_popup(Popup::Help);
            return;
        }
        KeyCode::Char('r') => {
            app.request_refresh();
            app.set_status("Refreshing...");
            return;
        }
        KeyCode::Char('n') => {
            app.open_popup(Popup::SwitchNetwork);
            return;
        }
        KeyCode::Char('s') => {
            app.show_all_addresses = !app.show_all_addresses;
            app.set_status(if app.show_all_addresses {
                "Showing all addresses"
            } else {
                "Showing active address only"
            });
            app.request_refresh();
            return;
        }
        KeyCode::Char('1') => {
            app.navigate(Screen::Coins);
            return;
        }
        KeyCode::Char('2') => {
            app.navigate(Screen::Objects);
            return;
        }
        KeyCode::Char('3') => {
            app.navigate(Screen::Transactions);
            return;
        }
        KeyCode::Char('4') => {
            app.navigate(Screen::Packages);
            return;
        }
        KeyCode::Char('5') => {
            app.navigate(Screen::AddressBook);
            return;
        }
        KeyCode::Char('6') => {
            app.navigate(Screen::Keys);
            return;
        }
        KeyCode::Char('7') => {
            app.navigate(Screen::TxBuilder);
            return;
        }
        KeyCode::Tab => {
            let idx = app.screen.index();
            let next = (idx + 1) % Screen::ALL.len();
            app.navigate(Screen::ALL[next]);
            return;
        }
        KeyCode::BackTab => {
            let idx = app.screen.index();
            let next = if idx == 0 {
                Screen::ALL.len() - 1
            } else {
                idx - 1
            };
            app.navigate(Screen::ALL[next]);
            return;
        }
        _ => {}
    }

    match app.screen {
        Screen::Coins => handle_coins_key(app, key),
        Screen::Objects => handle_objects_key(app, key),
        Screen::Transactions => handle_transactions_key(app, key),
        Screen::Packages => {}
        Screen::AddressBook => handle_address_key(app, key),
        Screen::Keys => handle_keys_key(app, key),
        Screen::TxBuilder => handle_tx_key(app, key),
    }
}

fn handle_popup_key(app: &mut App, key: KeyEvent) {
    match app.popup {
        Some(Popup::Help) => match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => app.popup = None,
            KeyCode::Down | KeyCode::Char('j') => {
                app.popup_scroll = app.popup_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
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
            KeyCode::Down | KeyCode::Char('j') => {
                app.popup_scroll = app.popup_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
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
                        app.address_book.push(crate::app::AddressEntry {
                            label,
                            address,
                            notes,
                        });
                        app.set_status("Address added");
                    } else if let Some(user_idx) = app.user_address_index(app.address_selected) {
                        if let Some(entry) = app.address_book.get_mut(user_idx) {
                            entry.label = label;
                            entry.address = address;
                            entry.notes = notes;
                            app.set_status("Address updated");
                        }
                    }
                    save_address_book(&app.address_book);
                }
                app.popup = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
            }
            _ => handle_input_key(app, key),
        },
        Some(Popup::AddCommand) => {
            // Pick which command type to add
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
                    AddCommandType::TransferIota => 2,    // recipient, amount
                    AddCommandType::TransferObjects => 2, // recipient, object_ids (comma-sep)
                    AddCommandType::MoveCall => 5, // package, module, function, type_args, args
                    AddCommandType::SplitCoins => 2, // coin, amounts (comma-sep)
                    AddCommandType::MergeCoins => 2, // primary, sources (comma-sep)
                    AddCommandType::Stake => 2,    // amount, validator address
                    AddCommandType::Unstake => 1,  // staked iota object id
                };
                app.tx_adding_cmd = Some(ct);
                app.tx_edit_field = 0;
                app.tx_edit_buffers = vec![String::new(); field_count];
                app.open_popup(Popup::AddCommandForm);
                app.start_input("");
            }
        }
        Some(Popup::AddCommandForm) => match key.code {
            KeyCode::Esc => {
                if app.autocomplete_idx.is_some() {
                    // First Esc dismisses autocomplete selection
                    app.autocomplete_idx = None;
                } else {
                    app.popup = None;
                    app.tx_adding_cmd = None;
                    app.input_mode = InputMode::Normal;
                    app.input_clear();
                    app.autocomplete.clear();
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
            KeyCode::Tab => {
                // Accept autocomplete if active, then advance to next field
                if app.accept_autocomplete() {
                    // accepted highlighted suggestion
                } else if !app.autocomplete.is_empty() {
                    app.autocomplete_idx = Some(0);
                    app.accept_autocomplete();
                }
                // Move to next field
                let val = app.input_buffer.clone();
                app.tx_edit_buffers[app.tx_edit_field] = val;
                let count = app.tx_edit_buffers.len();
                app.tx_edit_field = (app.tx_edit_field + 1) % count;
                let next_val = app.tx_edit_buffers[app.tx_edit_field].clone();
                app.start_input(&next_val);
                app.update_autocomplete();
            }
            KeyCode::Enter => {
                // If autocomplete is active, accept the suggestion instead of submitting
                if app.autocomplete_idx.is_some() {
                    app.accept_autocomplete();
                } else {
                    app.tx_edit_buffers[app.tx_edit_field] = app.input_buffer.clone();
                    if let Some(cmd) = build_command_from_form(app) {
                        app.tx_commands.push(cmd);
                        app.tx_dry_run_dirty = true;
                        app.set_status("Command added");
                    }
                    app.popup = None;
                    app.tx_adding_cmd = None;
                    app.input_mode = InputMode::Normal;
                    app.input_clear();
                    app.autocomplete.clear();
                    app.autocomplete_idx = None;
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
                let alias = format!("key-{}", app.keys.len());
                app.send_cmd(WalletCmd::GenerateKey {
                    scheme: scheme.to_string(),
                    alias,
                });
                app.set_status(format!("Generating {} keypair...", scheme));
                app.popup = None;
            }
        }
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
                if let Some(user_idx) = app.user_address_index(app.address_selected) {
                    if user_idx < app.address_book.len() {
                        app.address_book.remove(user_idx);
                        let combined_len = app.key_entry_count() + app.address_book.len();
                        if app.address_selected >= combined_len && app.address_selected > 0 {
                            app.address_selected -= 1;
                        }
                        save_address_book(&app.address_book);
                        app.set_status("Address removed");
                    }
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
            let ids_str = bufs.get(1)?.clone();
            if recipient.is_empty() || ids_str.is_empty() {
                return None;
            }
            let object_ids: Vec<String> =
                ids_str.split(',').map(|s| s.trim().to_string()).collect();
            Some(PtbCommand::TransferObjects {
                recipient,
                object_ids,
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
            let sources_str = bufs.get(1)?.clone();
            if primary.is_empty() || sources_str.is_empty() {
                return None;
            }
            let sources: Vec<String> = sources_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            Some(PtbCommand::MergeCoins { primary, sources })
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

fn handle_input_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => app.input_insert(c),
        KeyCode::Backspace => app.input_backspace(),
        KeyCode::Delete => app.input_delete(),
        KeyCode::Left => app.input_left(),
        KeyCode::Right => app.input_right(),
        KeyCode::Home => app.input_cursor = 0,
        KeyCode::End => app.input_cursor = app.input_buffer.len(),
        _ => {}
    }
}

fn handle_coins_key(app: &mut App, key: KeyEvent) {
    let len = app.coins.len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.coins_selected > 0 {
                app.coins_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.coins_selected + 1 < len {
                app.coins_selected += 1;
            }
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.coins_selected = 0;
        }
        KeyCode::End | KeyCode::Char('G') => {
            if len > 0 {
                app.coins_selected = len - 1;
            }
        }
        KeyCode::PageUp => {
            app.coins_selected = app.coins_selected.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.coins_selected = (app.coins_selected + 10).min(len.saturating_sub(1));
        }
        KeyCode::Enter => {
            if !app.coins.is_empty() {
                app.open_popup(Popup::Detail);
            }
        }
        KeyCode::Char('f') => {
            if let Some(key) = app.active_key() {
                if let Ok(addr) = iota_sdk::types::Address::from_hex(&key.address) {
                    app.send_cmd(WalletCmd::RequestFaucet(addr));
                    app.set_status("Requesting faucet...");
                }
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.coins_selected, &mut app.coins_offset, 20);
}

fn handle_objects_key(app: &mut App, key: KeyEvent) {
    let len = app.objects.len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.objects_selected > 0 {
                app.objects_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.objects_selected + 1 < len {
                app.objects_selected += 1;
            }
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.objects_selected = 0;
        }
        KeyCode::End | KeyCode::Char('G') => {
            if len > 0 {
                app.objects_selected = len - 1;
            }
        }
        KeyCode::PageUp => {
            app.objects_selected = app.objects_selected.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.objects_selected = (app.objects_selected + 10).min(len.saturating_sub(1));
        }
        KeyCode::Enter => {
            if !app.objects.is_empty() {
                app.open_popup(Popup::Detail);
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.objects_selected, &mut app.objects_offset, 20);
}

fn handle_transactions_key(app: &mut App, key: KeyEvent) {
    let len = app.transactions.len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.transactions_selected > 0 {
                app.transactions_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.transactions_selected + 1 < len {
                app.transactions_selected += 1;
            }
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.transactions_selected = 0;
        }
        KeyCode::End | KeyCode::Char('G') => {
            if len > 0 {
                app.transactions_selected = len - 1;
            }
        }
        KeyCode::PageUp => {
            app.transactions_selected = app.transactions_selected.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.transactions_selected = (app.transactions_selected + 10).min(len.saturating_sub(1));
        }
        KeyCode::Enter => {
            if !app.transactions.is_empty() {
                app.open_popup(Popup::Detail);
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.transactions_selected, &mut app.transactions_offset, 20);
}

fn handle_address_key(app: &mut App, key: KeyEvent) {
    let combined_len = app.key_entry_count() + app.address_book.len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.address_selected > 0 {
                app.address_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.address_selected + 1 < combined_len {
                app.address_selected += 1;
            }
        }
        KeyCode::Home => {
            app.address_selected = 0;
        }
        KeyCode::End => {
            if combined_len > 0 {
                app.address_selected = combined_len - 1;
            }
        }
        KeyCode::Enter => {
            if combined_len > 0 {
                app.open_popup(Popup::Detail);
            }
        }
        KeyCode::Char('a') => {
            app.address_edit_field = 0;
            app.address_edit_buffers = [String::new(), String::new(), String::new()];
            app.open_popup(Popup::AddAddress);
            app.start_input("");
        }
        KeyCode::Char('e') => {
            // Only allow editing user entries, not key entries
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
        _ => {}
    }
    App::scroll_into_view(app.address_selected, &mut app.address_offset, 20);
}

fn handle_keys_key(app: &mut App, key: KeyEvent) {
    let len = app.keys.len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.keys_selected > 0 {
                app.keys_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.keys_selected + 1 < len {
                app.keys_selected += 1;
            }
        }
        KeyCode::Home => {
            app.keys_selected = 0;
        }
        KeyCode::End => {
            if len > 0 {
                app.keys_selected = len - 1;
            }
        }
        KeyCode::Enter => {
            let idx = app.keys_selected;
            for (i, k) in app.keys.iter_mut().enumerate() {
                k.is_active = i == idx;
            }
            app.send_cmd(WalletCmd::SetActiveKey(idx));
            app.set_status("Active key changed");
            app.request_refresh();
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
        KeyCode::Char('d') | KeyCode::Delete => {
            if !app.keys.is_empty() {
                app.open_popup(Popup::ConfirmDeleteKey);
            }
        }
        _ => {}
    }
    App::scroll_into_view(app.keys_selected, &mut app.keys_offset, 20);
}

fn handle_tx_key(app: &mut App, key: KeyEvent) {
    match app.tx_step {
        TxBuilderStep::SelectSender => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if app.tx_sender > 0 {
                    app.tx_sender -= 1;
                    app.tx_dry_run_dirty = true;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.tx_sender + 1 < app.keys.len() {
                    app.tx_sender += 1;
                    app.tx_dry_run_dirty = true;
                }
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                app.tx_step = TxBuilderStep::EditCommands;
            }
            _ => {}
        },
        TxBuilderStep::EditCommands => match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                app.tx_step = TxBuilderStep::SelectSender;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.tx_step = TxBuilderStep::SetGas;
            }
            KeyCode::Char('a') => {
                app.open_popup(Popup::AddCommand);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.tx_cmd_selected > 0 {
                    app.tx_cmd_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
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
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.tx_step = TxBuilderStep::EditCommands;
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.tx_step = TxBuilderStep::Review;
                        trigger_dry_run(app);
                    }
                    KeyCode::Enter | KeyCode::Char('e') => {
                        app.start_input(&app.tx_gas_budget.clone());
                    }
                    _ => {}
                }
            }
        }
        TxBuilderStep::Review => match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                app.tx_step = TxBuilderStep::SetGas;
            }
            KeyCode::Enter => {
                submit_transaction(app);
            }
            _ => {}
        },
    }
}

fn submit_transaction(app: &mut App) {
    if app.keys.is_empty() {
        app.set_status("No keys available");
        return;
    }
    if app.tx_commands.is_empty() {
        app.set_status("No commands added");
        return;
    }

    let gas_budget: u64 = app.tx_gas_budget.parse().unwrap_or(10_000_000);

    app.send_cmd(WalletCmd::ExecutePtb {
        sender_idx: app.tx_sender,
        commands: app.tx_commands.clone(),
        gas_budget,
    });
    app.set_status("Submitting transaction...");
}

fn trigger_dry_run(app: &mut App) {
    if !app.tx_dry_run_dirty || app.keys.is_empty() || app.tx_commands.is_empty() {
        return;
    }
    app.tx_dry_run = None;
    app.tx_dry_running = true;
    app.tx_dry_run_dirty = false;
    app.send_cmd(WalletCmd::DryRun {
        sender_idx: app.tx_sender,
        commands: app.tx_commands.clone(),
    });
}

fn scroll_selection(app: &mut App, delta: i32) {
    match app.screen {
        Screen::Coins => {
            app.coins_selected = apply_delta(app.coins_selected, delta, app.coins.len());
            App::scroll_into_view(app.coins_selected, &mut app.coins_offset, 20);
        }
        Screen::Objects => {
            app.objects_selected = apply_delta(app.objects_selected, delta, app.objects.len());
            App::scroll_into_view(app.objects_selected, &mut app.objects_offset, 20);
        }
        Screen::Transactions => {
            app.transactions_selected =
                apply_delta(app.transactions_selected, delta, app.transactions.len());
            App::scroll_into_view(app.transactions_selected, &mut app.transactions_offset, 20);
        }
        Screen::AddressBook => {
            let combined_len = app.key_entry_count() + app.address_book.len();
            app.address_selected = apply_delta(app.address_selected, delta, combined_len);
            App::scroll_into_view(app.address_selected, &mut app.address_offset, 20);
        }
        Screen::Keys => {
            app.keys_selected = apply_delta(app.keys_selected, delta, app.keys.len());
            App::scroll_into_view(app.keys_selected, &mut app.keys_offset, 20);
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
    }
}

fn apply_delta(current: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = current as i32 + delta;
    new.clamp(0, (len as i32) - 1) as usize
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
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
                {
                    if let Some(&screen) = Screen::ALL.get(i) {
                        app.navigate(screen);
                        return;
                    }
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
                        // Click step indicator (row 3 area)
                        if row <= 4 {
                            let step_width = col as usize / 20;
                            if let Some(&step) = TxBuilderStep::ALL.get(step_width) {
                                app.tx_step = step;
                            }
                        } else {
                            // Click items in the current step content
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
                                    // Account for table header + margin
                                    if step_row >= 2 && step_row - 2 < app.tx_commands.len() {
                                        app.tx_cmd_selected = step_row - 2;
                                    }
                                }
                                _ => {}
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
