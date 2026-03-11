use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::Duration;

use crate::app::{App, InputMode, Popup, Screen, TxBuilderStep, TxRecipient};

pub fn poll_event(timeout: Duration) -> color_eyre::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_event(app: &mut App, ev: Event) {
    match ev {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        Event::Resize(_, _) => {} // ratatui handles this
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    // Global quit
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.running = false;
        return;
    }

    // Popup handling takes priority
    if app.popup.is_some() {
        handle_popup_key(app, key);
        return;
    }

    // If editing, route to input handler
    if app.input_mode == InputMode::Editing {
        handle_input_key(app, key);
        return;
    }

    // Global keys
    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
            return;
        }
        KeyCode::Char('?') => {
            app.popup = Some(Popup::Help);
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

    // Per-screen keys
    match app.screen {
        Screen::Coins => handle_coins_key(app, key),
        Screen::Objects => handle_objects_key(app, key),
        Screen::Packages => handle_packages_key(app, key),
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
                    // Save current field, move to next
                    let val = app.input_buffer.clone();
                    app.address_edit_buffers[app.address_edit_field] = val;
                    app.address_edit_field = (app.address_edit_field + 1) % 3;
                    let next_val = app.address_edit_buffers[app.address_edit_field].clone();
                    app.start_input(&next_val);
                }
                KeyCode::Enter => {
                    // Save current field
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
            match key.code {
                KeyCode::Esc => app.popup = None,
                KeyCode::Char('1') | KeyCode::Char('e') => {
                    let addr = format!("0x{}", crate::app::random_hex_pub(64));
                    app.keys.push(crate::app::KeyEntry {
                        alias: format!("key-{}", app.keys.len()),
                        address: addr,
                        scheme: "ed25519".into(),
                        is_active: false,
                    });
                    app.set_status("Generated new ed25519 keypair");
                    app.popup = None;
                }
                KeyCode::Char('2') | KeyCode::Char('s') => {
                    let addr = format!("0x{}", crate::app::random_hex_pub(64));
                    app.keys.push(crate::app::KeyEntry {
                        alias: format!("key-{}", app.keys.len()),
                        address: addr,
                        scheme: "secp256k1".into(),
                        is_active: false,
                    });
                    app.set_status("Generated new secp256k1 keypair");
                    app.popup = None;
                }
                KeyCode::Char('3') | KeyCode::Char('r') => {
                    let addr = format!("0x{}", crate::app::random_hex_pub(64));
                    app.keys.push(crate::app::KeyEntry {
                        alias: format!("key-{}", app.keys.len()),
                        address: addr,
                        scheme: "secp256r1".into(),
                        is_active: false,
                    });
                    app.set_status("Generated new secp256r1 keypair");
                    app.popup = None;
                }
                _ => {}
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
                        let addr = format!("0x{}", crate::app::random_hex_pub(64));
                        app.keys.push(crate::app::KeyEntry {
                            alias: format!("imported-{}", app.keys.len()),
                            address: addr,
                            scheme: "ed25519".into(),
                            is_active: false,
                        });
                        app.set_status("Key imported");
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
                app.set_status(format!("Object: {}", &coin.object_id[..20]));
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
                app.set_status(format!("Object: {}", &obj.object_id[..20]));
            }
        }
        _ => {}
    }
}

fn handle_packages_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.packages_selected > 0 {
                app.packages_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.packages_selected + 1 < app.packages.len() {
                app.packages_selected += 1;
            }
        }
        KeyCode::Enter => {
            if app.packages_expanded == Some(app.packages_selected) {
                app.packages_expanded = None;
            } else {
                app.packages_expanded = Some(app.packages_selected);
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
            // Set active key
            for (i, k) in app.keys.iter_mut().enumerate() {
                k.is_active = i == app.keys_selected;
            }
            app.set_status("Active key changed");
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
                    if app.tx_sender > 0 {
                        app.tx_sender -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.tx_sender + 1 < app.keys.len() {
                        app.tx_sender += 1;
                    }
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
                    if app.tx_recipient_selected > 0 {
                        app.tx_recipient_selected -= 1;
                    }
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
                    KeyCode::Enter => {
                        app.tx_gas_budget = app.stop_input();
                    }
                    KeyCode::Esc => {
                        app.stop_input();
                    }
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
                    app.set_status("Transaction signed and submitted (mock)");
                    app.popup = Some(Popup::Confirm);
                }
                _ => {}
            }
        }
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let col = mouse.column;
            let row = mouse.row;

            // Check tab clicks
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

            // Screen-specific mouse handling
            // Row clicks on list items (simplified: assume content starts at row 4)
            let content_start = 4u16;
            if row >= content_start {
                let list_index = (row - content_start) as usize;
                match app.screen {
                    Screen::Coins => {
                        // Each coin takes 2 rows in the table (header + data), but we use a table
                        // so row offset maps directly if within bounds
                        if list_index < app.coins.len() {
                            app.coins_selected = list_index;
                        }
                    }
                    Screen::Objects => {
                        if list_index < app.objects.len() {
                            app.objects_selected = list_index;
                        }
                    }
                    Screen::Packages => {
                        if list_index < app.packages.len() {
                            app.packages_selected = list_index;
                        }
                    }
                    Screen::AddressBook => {
                        if list_index < app.address_book.len() {
                            app.address_selected = list_index;
                        }
                    }
                    Screen::Keys => {
                        if list_index < app.keys.len() {
                            app.keys_selected = list_index;
                        }
                    }
                    Screen::TxBuilder => {
                        // Check if clicking on step indicators (row 3)
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
                Screen::Coins => {
                    if app.coins_selected > 0 { app.coins_selected -= 1; }
                }
                Screen::Objects => {
                    if app.objects_selected > 0 { app.objects_selected -= 1; }
                }
                Screen::Packages => {
                    if app.packages_selected > 0 { app.packages_selected -= 1; }
                }
                Screen::AddressBook => {
                    if app.address_selected > 0 { app.address_selected -= 1; }
                }
                Screen::Keys => {
                    if app.keys_selected > 0 { app.keys_selected -= 1; }
                }
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            match app.screen {
                Screen::Coins => {
                    if app.coins_selected + 1 < app.coins.len() { app.coins_selected += 1; }
                }
                Screen::Objects => {
                    if app.objects_selected + 1 < app.objects.len() { app.objects_selected += 1; }
                }
                Screen::Packages => {
                    if app.packages_selected + 1 < app.packages.len() { app.packages_selected += 1; }
                }
                Screen::AddressBook => {
                    if app.address_selected + 1 < app.address_book.len() { app.address_selected += 1; }
                }
                Screen::Keys => {
                    if app.keys_selected + 1 < app.keys.len() { app.keys_selected += 1; }
                }
                _ => {}
            }
        }
        _ => {}
    }
}
