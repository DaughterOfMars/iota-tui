//! Keyboard event handling for the AddCommandForm popup.

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AddCommandType, App, InputMode, PtbCommand};

use super::super::input::handle_input_key;

pub fn handle_command_form_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            if app.autocomplete_idx.is_some() {
                app.autocomplete_idx = None;
            } else {
                app.popup = None;
                app.tx.adding_cmd = None;
                app.input_mode = InputMode::Normal;
                app.input_clear();
                app.autocomplete.clear();
                app.tx.multi_values.clear();
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
                && app.tx.is_multi_value_field()
                && !app.tx.multi_values.is_empty() =>
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
            } else if app.tx.is_multi_value_field() && !app.input_buffer.is_empty() {
                // Manual entry: add typed text as a value
                let val = app.input_buffer.clone();
                app.tx.multi_values.push(val);
                app.input_buffer.clear();
                app.input_cursor = 0;
            }

            if !app.tx.is_multi_value_field() {
                // Advance to next field for single-value fields
                let val = app.input_buffer.clone();
                app.tx.edit_buffers[app.tx.edit_field] = val;
                let count = app.tx.edit_buffers.len();
                app.tx.edit_field = (app.tx.edit_field + 1) % count;
                let next_val = app.tx.edit_buffers[app.tx.edit_field].clone();
                app.start_input(&next_val);
            }
            app.update_autocomplete();
        }
        KeyCode::Enter => {
            if app.autocomplete_idx.is_some() {
                app.accept_autocomplete();
            } else if app.tx.is_multi_value_field() && !app.input_buffer.is_empty() {
                // Manual entry: add typed text as a value
                let val = app.input_buffer.clone();
                app.tx.multi_values.push(val);
                app.input_buffer.clear();
                app.input_cursor = 0;
                app.update_autocomplete();
            } else {
                app.tx.edit_buffers[app.tx.edit_field] = app.input_buffer.clone();
                if let Some(cmd) = build_command_from_form(app) {
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
        _ => {
            handle_input_key(app, key);
            app.update_autocomplete();
        }
    }
}

/// Parse the form buffers into a PtbCommand based on the selected command type.
/// Address fields are resolved through aliases (key aliases + address book labels).
pub(crate) fn build_command_from_form(app: &App) -> Option<PtbCommand> {
    let ct = app.tx.adding_cmd?;
    let bufs = &app.tx.edit_buffers;
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
            if recipient.is_empty() || app.tx.multi_values.is_empty() {
                return None;
            }
            Some(PtbCommand::TransferObjects {
                recipient,
                object_ids: app.tx.multi_values.clone(),
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
            if primary.is_empty() || app.tx.multi_values.is_empty() {
                return None;
            }
            Some(PtbCommand::MergeCoins {
                primary,
                sources: app.tx.multi_values.clone(),
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
