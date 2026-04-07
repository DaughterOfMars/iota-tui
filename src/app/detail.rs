//! Detail popup info and clipboard copy logic.

use crate::app::{App, Screen};

impl App {
    /// Returns (title, key-value lines) for the detail popup of the selected item.
    pub fn detail_info(&self) -> (&'static str, Vec<(&'static str, String)>) {
        match self.screen {
            Screen::Coins => {
                if let Some(c) = self.coins.get(self.coins_selected) {
                    let mut fields = vec![
                        ("Symbol", c.symbol.clone()),
                        ("Coin Type", c.coin_type.clone()),
                        ("Balance", c.balance_display.clone()),
                        ("Raw Balance", c.balance.to_string()),
                        ("Object ID", c.object_id.clone()),
                    ];
                    if self.show_multiple_owners() {
                        fields.push(("Owner", c.owner_alias.clone()));
                    }
                    ("Coin Details", fields)
                } else {
                    ("Coin Details", vec![])
                }
            }
            Screen::Objects => {
                if let Some(o) = self.objects.get(self.objects_selected) {
                    let mut fields = vec![
                        ("Object ID", o.object_id.clone()),
                        ("Type", o.type_name.clone()),
                        ("Version", o.version.clone()),
                        ("Digest", o.digest.clone()),
                        ("Owner", o.owner.clone()),
                    ];
                    if self.show_multiple_owners() {
                        fields.push(("Key", o.owner_alias.clone()));
                    }
                    ("Object Details", fields)
                } else {
                    ("Object Details", vec![])
                }
            }
            Screen::Transactions => {
                if let Some(tx) = self.transactions.get(self.transactions_selected) {
                    (
                        "Transaction Details",
                        vec![
                            ("Digest", tx.digest.clone()),
                            ("Status", tx.status.clone()),
                            ("Gas Used", tx.gas_used.clone()),
                            ("Epoch", tx.epoch.clone()),
                        ],
                    )
                } else {
                    ("Transaction Details", vec![])
                }
            }
            Screen::Staking => {
                if let Some(s) = self.stakes.get(self.stakes_selected) {
                    (
                        "Stake Details",
                        vec![
                            ("Object ID", s.object_id.clone()),
                            ("Principal", s.principal_display.clone()),
                            ("Validator", s.validator_address.clone()),
                            ("Activation Epoch", s.activation_epoch.clone()),
                            ("Status", s.status.clone()),
                        ],
                    )
                } else {
                    ("Stake Details", vec![])
                }
            }
            Screen::Keys => {
                if let Some(k) = self.keys.get(self.keys_selected) {
                    let mut fields = vec![
                        ("Alias", k.alias.clone()),
                        ("Address", k.address.clone()),
                        ("Scheme", k.scheme.clone()),
                        ("Active", if k.is_active { "Yes" } else { "No" }.to_string()),
                    ];
                    if self.keys_show_private {
                        fields.push(("Private Key", k.private_key_hex.clone()));
                    }
                    ("Key Details", fields)
                } else {
                    ("Key Details", vec![])
                }
            }
            Screen::AddressBook => {
                let combined = self.combined_address_book();
                if let Some(entry) = combined.get(self.address_selected) {
                    (
                        "Address Details",
                        vec![
                            ("Label", entry.label.clone()),
                            ("Address", entry.address.clone()),
                            ("Notes", entry.notes.clone()),
                        ],
                    )
                } else {
                    ("Address Details", vec![])
                }
            }
            Screen::Explorer => ("Details", vec![]),
            _ => ("Details", vec![]),
        }
    }

    /// Copy the primary field of the selected item to the system clipboard.
    pub fn copy_selected(&mut self) {
        let text = match self.screen {
            Screen::Coins => self
                .coins
                .get(self.coins_selected)
                .map(|c| c.object_id.clone()),
            Screen::Objects => self
                .objects
                .get(self.objects_selected)
                .map(|o| o.object_id.clone()),
            Screen::Transactions => self
                .transactions
                .get(self.transactions_selected)
                .map(|t| t.digest.clone()),
            Screen::Staking => self
                .stakes
                .get(self.stakes_selected)
                .map(|s| s.object_id.clone()),
            Screen::Packages => {
                let indices = self.package_indices();
                indices
                    .get(self.packages_selected)
                    .and_then(|&i| self.objects.get(i))
                    .map(|o| o.object_id.clone())
            }
            Screen::AddressBook => {
                let combined = self.combined_address_book();
                combined
                    .get(self.address_selected)
                    .map(|e| e.address.clone())
            }
            Screen::Keys => self.keys.get(self.keys_selected).map(|k| k.address.clone()),
            Screen::Explorer => {
                if let Some(ref result) = self.explorer.lookup_result {
                    if self.explorer.lookup_depth == 1 {
                        result
                            .sections()
                            .get(self.explorer.lookup_section)
                            .and_then(|s| s.fields.get(self.explorer.lookup_field_idx))
                            .map(|f| f.value.clone())
                    } else {
                        result
                            .sections()
                            .get(self.explorer.lookup_section)
                            .map(|s| s.title.clone())
                    }
                } else {
                    None
                }
            }
            Screen::TxBuilder => None,
        };

        if let Some(text) = text {
            match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(&text)) {
                Ok(()) => {
                    let display = if text.len() > 24 {
                        format!("{}..{}", &text[..10], &text[text.len() - 10..])
                    } else {
                        text
                    };
                    self.clipboard_toast =
                        Some((format!("Copied: {}", display), std::time::Instant::now()));
                }
                Err(e) => {
                    self.clipboard_toast =
                        Some((format!("Copy failed: {}", e), std::time::Instant::now()));
                }
            }
        }
    }
}
