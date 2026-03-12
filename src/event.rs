use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, InputMode, Popup, Screen, TxBuilderStep, TxRecipient, save_address_book};
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
            app.running = false;
            return;
        }
        KeyCode::Char('?') => {
            app.popup = Some(Popup::Help);
            return;
        }
        KeyCode::Char('r') => {
            // Global refresh
            app.request_refresh();
            app.set_status("Refreshing...");
            return;
        }
        KeyCode::Char('1') => { app.navigate(Screen::Coins); return; }
        KeyCode::Char('2') => { app.navigate(Screen::Objects); return; }
        KeyCode::Char('3') => { app.navigate(Screen::Packages); return; }
        KeyCode::Char('4') => { app.navigate(Screen::AddressBook); return; }
        KeyCode::Char('5') => { app.navigate(Screen::Keys); return; }
        KeyCode::Char('6') => { app.navigate(Screen::TxBuilder); return; }
        KeyCode::Tab => {
            let idx = app.screen.index();
            let next = (idx + 1) % Screen::ALL.len();
            app.navigate(Screen::ALL[next]);
            return;
        }
        KeyCode::BackTab => {
            let idx = app.screen.index();
            let next = if idx == 0 { Screen::ALL.len() - 1 } else { idx - 1 };
            app.navigate(Screen::ALL[next]);
            return;
        }
        _ => {}
    }

    match app.screen {
        Screen::Coins => handle_coins_key(app, key),
        Screen::Objects => handle_objects_key(app, key),
        Screen::Packages => {} // No packages screen actions yet (read-only from objects)
        Screen::AddressBook => handle_address_key(app, key),
        Screen::Keys => handle_keys_key(app, key),
        Screen::TxBuilder => handle_tx_key(app, key),
    }
}

fn handle_popup_key(app: &mut App, key: KeyEvent) {
    match app.popup {
        Some(Popup::Help | Popup::Confirm) => {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => app.popup = None,
                _ => {}
            }
        }
        Some(Popup::AddAddress | Popup::EditAddress) => {
            match key.code {
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
                                label, address, notes,
                            });
                            app.set_status("Address added");
                        } else if let Some(entry) = app.address_book.get_mut(app.address_selected) {
                            entry.label = label;
                            entry.address = address;
                            entry.notes = notes;
                            app.set_status("Address updated");
                        }
                        save_address_book(&app.address_book);
                    }
                    app.popup = None;
                    app.input_mode = InputMode::Normal;
                    app.input_clear();
                }
                _ => handle_input_key(app, key),
            }
        }
        Some(Popup::AddRecipient) => {
            match key.code {
                KeyCode::Esc => {
                    app.popup = None;
                    app.input_mode = InputMode::Normal;
                    app.input_clear();
                }
                KeyCode::Tab => {
                    let val = app.input_buffer.clone();
                    app.tx_edit_buffers[app.tx_edit_field] = val;
                    app.tx_edit_field = (app.tx_edit_field + 1) % 2;
                    let next_val = app.tx_edit_buffers[app.tx_edit_field].clone();
                    app.start_input(&next_val);
                }
                KeyCode::Enter => {
                    app.tx_edit_buffers[app.tx_edit_field] = app.input_buffer.clone();
                    let [address, amount] = app.tx_edit_buffers.clone();
                    if !address.is_empty() && !amount.is_empty() {
                        app.tx_recipients.push(TxRecipient { address, amount });
                        app.set_status("Recipient added");
                    }
                    app.popup = None;
                    app.input_mode = InputMode::Normal;
                    app.input_clear();
                }
                _ => handle_input_key(app, key),
            }
        }
        Some(Popup::GenerateKey) => {
            let scheme = match key.code {
                KeyCode::Char('1') | KeyCode::Char('e') => Some("ed25519"),
                KeyCode::Char('2') | KeyCode::Char('s') => Some("secp256k1"),
                KeyCode::Char('3') | KeyCode::Char('r') => Some("secp256r1"),
                KeyCode::Esc => { app.popup = None; None }
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
        Some(Popup::ImportKey) => {
            match key.code {
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
            }
        }
        None => {}
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
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.coins_selected > 0 {
                app.coins_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.coins_selected + 1 < app.coins.len() {
                app.coins_selected += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(coin) = app.coins.get(app.coins_selected) {
                let id = &coin.object_id;
                let display = if id.len() > 20 { &id[..20] } else { id };
                app.set_status(format!("Object: {}", display));
            }
        }
        KeyCode::Char('f') => {
            // Request faucet
            if let Some(key) = app.active_key() {
                if let Ok(addr) = iota_sdk::types::Address::from_hex(&key.address) {
                    app.send_cmd(WalletCmd::RequestFaucet(addr));
                    app.set_status("Requesting faucet...");
                }
            }
        }
        _ => {}
    }
}

fn handle_objects_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.objects_selected > 0 {
                app.objects_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.objects_selected + 1 < app.objects.len() {
                app.objects_selected += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(obj) = app.objects.get(app.objects_selected) {
                let id = &obj.object_id;
                let display = if id.len() > 20 { &id[..20] } else { id };
                app.set_status(format!("Object: {}", display));
            }
        }
        _ => {}
    }
}

fn handle_address_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.address_selected > 0 {
                app.address_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.address_selected + 1 < app.address_book.len() {
                app.address_selected += 1;
            }
        }
        KeyCode::Char('a') => {
            app.address_edit_field = 0;
            app.address_edit_buffers = [String::new(), String::new(), String::new()];
            app.popup = Some(Popup::AddAddress);
            app.start_input("");
        }
        KeyCode::Char('e') => {
            if let Some(entry) = app.address_book.get(app.address_selected) {
                let label = entry.label.clone();
                let address = entry.address.clone();
                let notes = entry.notes.clone();
                app.address_edit_field = 0;
                app.address_edit_buffers = [label.clone(), address, notes];
                app.popup = Some(Popup::EditAddress);
                app.start_input(&label);
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if !app.address_book.is_empty() {
                app.address_book.remove(app.address_selected);
                if app.address_selected >= app.address_book.len() && app.address_selected > 0 {
                    app.address_selected -= 1;
                }
                save_address_book(&app.address_book);
                app.set_status("Address removed");
            }
        }
        _ => {}
    }
}

fn handle_keys_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.keys_selected > 0 {
                app.keys_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.keys_selected + 1 < app.keys.len() {
                app.keys_selected += 1;
            }
        }
        KeyCode::Enter => {
            // Set active key and refresh
            for (i, k) in app.keys.iter_mut().enumerate() {
                k.is_active = i == app.keys_selected;
            }
            app.set_status("Active key changed");
            app.request_refresh();
        }
        KeyCode::Char('g') => {
            app.popup = Some(Popup::GenerateKey);
        }
        KeyCode::Char('i') => {
            app.popup = Some(Popup::ImportKey);
            app.start_input("");
        }
        KeyCode::Char('p') => {
            app.keys_show_private = !app.keys_show_private;
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if app.keys.len() > 1 {
                let removed = app.keys.remove(app.keys_selected);
                if removed.is_active && !app.keys.is_empty() {
                    app.keys[0].is_active = true;
                }
                if app.keys_selected >= app.keys.len() && app.keys_selected > 0 {
                    app.keys_selected -= 1;
                }
                app.set_status("Key removed");
            } else {
                app.set_status("Cannot remove last key");
            }
        }
        _ => {}
    }
}

fn handle_tx_key(app: &mut App, key: KeyEvent) {
    match app.tx_step {
        TxBuilderStep::SelectSender => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.tx_sender > 0 { app.tx_sender -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.tx_sender + 1 < app.keys.len() { app.tx_sender += 1; }
                }
                KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                    app.tx_step = TxBuilderStep::AddRecipients;
                }
                _ => {}
            }
        }
        TxBuilderStep::AddRecipients => {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    app.tx_step = TxBuilderStep::SelectSender;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    app.tx_step = TxBuilderStep::SetGas;
                }
                KeyCode::Char('a') => {
                    app.tx_edit_field = 0;
                    app.tx_edit_buffers = [String::new(), String::new()];
                    app.popup = Some(Popup::AddRecipient);
                    app.start_input("");
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.tx_recipient_selected > 0 { app.tx_recipient_selected -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.tx_recipient_selected + 1 < app.tx_recipients.len() {
                        app.tx_recipient_selected += 1;
                    }
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    if !app.tx_recipients.is_empty() {
                        app.tx_recipients.remove(app.tx_recipient_selected);
                        if app.tx_recipient_selected >= app.tx_recipients.len()
                            && app.tx_recipient_selected > 0
                        {
                            app.tx_recipient_selected -= 1;
                        }
                    }
                }
                _ => {}
            }
        }
        TxBuilderStep::SetGas => {
            if app.input_mode == InputMode::Editing {
                match key.code {
                    KeyCode::Enter => { app.tx_gas_budget = app.stop_input(); }
                    KeyCode::Esc => { app.stop_input(); }
                    _ => handle_input_key(app, key),
                }
            } else {
                match key.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.tx_step = TxBuilderStep::AddRecipients;
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.tx_step = TxBuilderStep::Review;
                    }
                    KeyCode::Enter | KeyCode::Char('e') => {
                        app.start_input(&app.tx_gas_budget.clone());
                    }
                    _ => {}
                }
            }
        }
        TxBuilderStep::Review => {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    app.tx_step = TxBuilderStep::SetGas;
                }
                KeyCode::Enter => {
                    submit_transaction(app);
                }
                _ => {}
            }
        }
    }
}

fn submit_transaction(app: &mut App) {
    if app.keys.is_empty() {
        app.set_status("No keys available");
        return;
    }
    if app.tx_recipients.is_empty() {
        app.set_status("No recipients added");
        return;
    }

    let gas_budget: u64 = app.tx_gas_budget.parse().unwrap_or(10_000_000);

    // For now, send to first recipient only (multi-recipient needs PTB)
    let recipient = &app.tx_recipients[0];
    let amount_str = &recipient.amount;

    // Parse amount as IOTA (with 9 decimals = NANOS)
    let amount_nanos: u64 = if let Ok(f) = amount_str.parse::<f64>() {
        (f * 1_000_000_000.0) as u64
    } else if let Ok(n) = amount_str.parse::<u64>() {
        n * 1_000_000_000
    } else {
        app.set_status("Invalid amount");
        return;
    };

    let Ok(recipient_addr) = iota_sdk::types::Address::from_hex(&recipient.address) else {
        app.set_status("Invalid recipient address");
        return;
    };

    app.send_cmd(WalletCmd::SendIota {
        sender_idx: app.tx_sender,
        recipient: recipient_addr,
        amount: amount_nanos,
        gas_budget,
    });
    app.set_status("Submitting transaction...");
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let col = mouse.column;
            let row = mouse.row;

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

            let content_start = 4u16;
            if row >= content_start {
                let list_index = (row - content_start) as usize;
                match app.screen {
                    Screen::Coins => {
                        if list_index < app.coins.len() { app.coins_selected = list_index; }
                    }
                    Screen::Objects => {
                        if list_index < app.objects.len() { app.objects_selected = list_index; }
                    }
                    Screen::Packages => {}
                    Screen::AddressBook => {
                        if list_index < app.address_book.len() { app.address_selected = list_index; }
                    }
                    Screen::Keys => {
                        if list_index < app.keys.len() { app.keys_selected = list_index; }
                    }
                    Screen::TxBuilder => {
                        if row == 3 {
                            let step_width = col as usize / 20;
                            if let Some(&step) = TxBuilderStep::ALL.get(step_width) {
                                app.tx_step = step;
                            }
                        }
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            match app.screen {
                Screen::Coins => { if app.coins_selected > 0 { app.coins_selected -= 1; } }
                Screen::Objects => { if app.objects_selected > 0 { app.objects_selected -= 1; } }
                Screen::AddressBook => { if app.address_selected > 0 { app.address_selected -= 1; } }
                Screen::Keys => { if app.keys_selected > 0 { app.keys_selected -= 1; } }
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            match app.screen {
                Screen::Coins => { if app.coins_selected + 1 < app.coins.len() { app.coins_selected += 1; } }
                Screen::Objects => { if app.objects_selected + 1 < app.objects.len() { app.objects_selected += 1; } }
                Screen::AddressBook => { if app.address_selected + 1 < app.address_book.len() { app.address_selected += 1; } }
                Screen::Keys => { if app.keys_selected + 1 < app.keys.len() { app.keys_selected += 1; } }
                _ => {}
            }
        }
        _ => {}
    }
}
