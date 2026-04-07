//! Address book persistence and combined address book helpers.

use crate::app::{AddressEntry, App};

fn address_book_path() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("iota-wallet-tui")
        .join("address_book.json")
}

pub fn load_address_book() -> Vec<AddressEntry> {
    let path = address_book_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

pub fn save_address_book(entries: &[AddressEntry]) {
    let path = address_book_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(entries) {
        let _ = std::fs::write(&path, data);
    }
}

impl App {
    /// Returns combined address book: keys first (read-only), then user entries.
    pub fn combined_address_book(&self) -> Vec<AddressEntry> {
        let mut combined: Vec<AddressEntry> = self
            .keys
            .iter()
            .map(|k| AddressEntry {
                label: format!("{} (key)", k.alias),
                address: k.address.clone(),
                notes: k.scheme.clone(),
            })
            .collect();
        combined.extend(self.address_book.iter().cloned());
        combined
    }

    /// Convert a combined address book index to a user address book index.
    /// Returns None if the index points to a key entry (read-only).
    pub fn user_address_index(&self, combined_idx: usize) -> Option<usize> {
        let key_count = self.key_entry_count();
        if combined_idx >= key_count {
            Some(combined_idx - key_count)
        } else {
            None
        }
    }
}
