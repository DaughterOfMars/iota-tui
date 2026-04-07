//! Autocomplete and address resolution logic.

use crate::app::{App, Popup, PopupFocus};

impl App {
    /// Compute autocomplete suggestions based on current input.
    pub fn update_autocomplete(&mut self) {
        // Quick action popups also support address autocomplete on their recipient field
        let quick_addr = matches!(
            self.popup,
            Some(Popup::QuickTransfer) | Some(Popup::ObjectTransfer)
        ) && self.popup_focus == PopupFocus::Fields
            && (self.popup == Some(Popup::ObjectTransfer) || self.quick_transfer_field == 0);

        let is_addr = self.tx.is_address_field() || quick_addr;
        let is_obj = self.tx.is_object_field();

        if (!is_addr && !is_obj) || self.input_buffer.is_empty() {
            self.autocomplete.clear();
            self.autocomplete_idx = None;
            return;
        }

        if is_addr && self.input_buffer.starts_with("0x") {
            self.autocomplete.clear();
            self.autocomplete_idx = None;
            return;
        }

        let query = self.input_buffer.to_lowercase();
        let mut matches: Vec<(String, String)> = Vec::new();

        if is_addr {
            for key in &self.keys {
                if key.alias.to_lowercase().contains(&query) {
                    matches.push((key.alias.clone(), key.address.clone()));
                }
            }
            for entry in &self.address_book {
                if entry.label.to_lowercase().contains(&query) {
                    matches.push((entry.label.clone(), entry.address.clone()));
                }
            }
        } else if is_obj {
            let already = &self.tx.multi_values;
            if self.tx.is_coin_field() {
                for coin in &self.coins {
                    if already.contains(&coin.object_id) {
                        continue;
                    }
                    let label = format!("{} ({})", coin.symbol, coin.balance_display);
                    if label.to_lowercase().contains(&query)
                        || coin.object_id.to_lowercase().contains(&query)
                    {
                        matches.push((label, coin.object_id.clone()));
                    }
                }
            } else {
                for obj in &self.objects {
                    if already.contains(&obj.object_id) {
                        continue;
                    }
                    let short_type = obj.type_name.rsplit("::").next().unwrap_or(&obj.type_name);
                    let label = format!(
                        "{} {}",
                        short_type,
                        &obj.object_id[..12.min(obj.object_id.len())]
                    );
                    if label.to_lowercase().contains(&query)
                        || obj.object_id.to_lowercase().contains(&query)
                        || obj.type_name.to_lowercase().contains(&query)
                    {
                        matches.push((label, obj.object_id.clone()));
                    }
                }
            }
        }

        matches.truncate(5);
        if let Some(idx) = self.autocomplete_idx
            && idx >= matches.len()
        {
            self.autocomplete_idx = if matches.is_empty() {
                None
            } else {
                Some(matches.len() - 1)
            };
        }
        self.autocomplete = matches;
    }

    /// Accept the currently highlighted autocomplete suggestion.
    /// Returns true if a suggestion was accepted.
    /// For address fields, inserts the alias (resolved later). For object fields, inserts the ID.
    /// For multi-value fields, adds to `tx.multi_values` and clears the input for the next pick.
    pub fn accept_autocomplete(&mut self) -> bool {
        if self.autocomplete.is_empty() {
            return false;
        }
        let idx = self.autocomplete_idx.unwrap_or(0);
        let is_obj = self.tx.is_object_field();
        if let Some((label, value)) = self.autocomplete.get(idx) {
            let insertion = if is_obj { value.clone() } else { label.clone() };

            if self.tx.is_multi_value_field() {
                // Add to the accumulated list and clear input for the next selection
                self.tx.multi_values.push(insertion);
                self.input_buffer.clear();
                self.input_cursor = 0;
            } else {
                self.input_buffer = insertion;
                self.input_cursor = self.input_buffer.len();
            }

            self.autocomplete.clear();
            self.autocomplete_idx = None;
            return true;
        }
        false
    }

    /// Remove the last item from multi-value accumulator (undo last pick).
    pub fn remove_last_multi_value(&mut self) {
        self.tx.multi_values.pop();
    }

    /// Resolve an alias or label to an address.
    /// Checks key aliases first, then address book labels. Case-insensitive.
    /// Returns the original string if no match is found.
    pub fn resolve_address(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();
        for key in &self.keys {
            if key.alias.to_lowercase() == input_lower {
                return key.address.clone();
            }
        }
        for entry in &self.address_book {
            if entry.label.to_lowercase() == input_lower {
                return entry.address.clone();
            }
        }
        input.to_string()
    }
}
