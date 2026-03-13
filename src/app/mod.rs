//! Application state and logic for the TUI.

mod types;

pub use types::*;

use tokio::sync::mpsc;

use crate::wallet::{StoredKey, WalletCmd, WalletEvent};

/// Central application state shared between the event handler and UI renderer.
pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub popup: Option<Popup>,
    pub status_message: Option<(String, std::time::Instant)>,

    // Network state
    pub connected: bool,
    pub network_name: String,
    pub loading: bool,

    // Wallet command channel
    pub cmd_tx: mpsc::Sender<WalletCmd>,

    // Per-screen data
    pub coins: Vec<CoinDisplay>,
    pub coins_selected: usize,
    pub coins_offset: usize,
    pub total_balance_iota: u128,

    pub objects: Vec<ObjectDisplay>,
    pub objects_selected: usize,
    pub objects_offset: usize,

    pub transactions: Vec<TransactionDisplay>,
    pub transactions_selected: usize,
    pub transactions_offset: usize,

    pub address_book: Vec<AddressEntry>,
    pub address_selected: usize,
    pub address_offset: usize,
    pub address_edit_field: usize,
    pub address_edit_buffers: [String; 3],

    pub keys: Vec<KeyDisplay>,
    pub keys_selected: usize,
    pub keys_offset: usize,
    pub keys_show_private: bool,
    pub keys_gen_scheme: Option<String>,

    pub tx_step: TxBuilderStep,
    pub tx_sender: usize,
    pub tx_commands: Vec<PtbCommand>,
    pub tx_cmd_selected: usize,
    pub tx_gas_budget: String,
    pub tx_edit_field: usize,
    pub tx_edit_buffers: Vec<String>,
    pub tx_adding_cmd: Option<AddCommandType>,
    pub tx_dry_run: Option<DryRunInfo>,
    pub tx_dry_running: bool,
    pub tx_dry_run_dirty: bool,
    pub tx_gas_edited: bool,

    // Autocomplete state for address fields
    pub autocomplete: Vec<(String, String)>, // (alias/label, address)
    pub autocomplete_idx: Option<usize>,

    // Error log content (loaded on demand)
    pub error_log_lines: Vec<String>,

    // Popup scroll state
    pub popup_scroll: usize,

    // Layout state for mouse hit-testing
    pub tab_areas: Vec<ratatui::layout::Rect>,
}

impl App {
    pub fn new(cmd_tx: mpsc::Sender<WalletCmd>, initial_keys: Vec<StoredKey>) -> Self {
        let keys: Vec<KeyDisplay> = initial_keys
            .iter()
            .map(|k| KeyDisplay {
                alias: k.alias.clone(),
                address: k.address.clone(),
                scheme: k.scheme.clone(),
                is_active: k.is_active,
                visible: true,
                private_key_hex: hex::encode(&k.private_key_bytes),
            })
            .collect();

        let address_book = load_address_book();

        App {
            running: true,
            screen: Screen::Coins,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_cursor: 0,
            popup: None,
            status_message: None,

            connected: false,
            network_name: "disconnected".into(),
            loading: false,

            cmd_tx,

            coins: vec![],
            coins_selected: 0,
            coins_offset: 0,
            total_balance_iota: 0,

            objects: vec![],
            objects_selected: 0,
            objects_offset: 0,

            transactions: vec![],
            transactions_selected: 0,
            transactions_offset: 0,

            address_book,
            address_selected: 0,
            address_offset: 0,
            address_edit_field: 0,
            address_edit_buffers: [String::new(), String::new(), String::new()],

            keys,
            keys_selected: 0,
            keys_offset: 0,
            keys_show_private: false,
            keys_gen_scheme: None,

            tx_step: TxBuilderStep::SelectSender,
            tx_sender: 0,
            tx_commands: vec![],
            tx_cmd_selected: 0,
            tx_gas_budget: "10000000".into(),
            tx_edit_field: 0,
            tx_edit_buffers: vec![],
            tx_adding_cmd: None,
            tx_dry_run: None,
            tx_dry_running: false,
            tx_dry_run_dirty: true,
            tx_gas_edited: false,

            autocomplete: vec![],
            autocomplete_idx: None,

            error_log_lines: vec![],

            popup_scroll: 0,

            tab_areas: vec![],
        }
    }

    /// Handle a response from the wallet backend.
    pub fn handle_wallet_event(&mut self, event: WalletEvent) {
        self.loading = false;
        match event {
            WalletEvent::Connected(network) => {
                self.connected = true;
                self.network_name = network;
                self.set_status("Connected");
                self.request_refresh();
            }
            WalletEvent::Balances(balances) => {
                for b in &balances {
                    if b.coin_type.contains("IOTA") {
                        if self.visible_key_count() > 1 {
                            self.total_balance_iota += b.total_balance;
                        } else {
                            self.total_balance_iota = b.total_balance;
                        }
                    }
                }
            }
            WalletEvent::Coins { coins, owner_alias } => {
                let new_coins: Vec<CoinDisplay> = coins
                    .into_iter()
                    .map(|c| CoinDisplay {
                        balance_display: format_balance(c.balance, 9),
                        coin_type: c.coin_type,
                        symbol: c.symbol,
                        balance: c.balance,
                        object_id: c.object_id,
                        owner_alias: owner_alias.clone(),
                    })
                    .collect();
                if self.visible_key_count() > 1 {
                    self.coins.extend(new_coins);
                } else {
                    self.coins = new_coins;
                }
                if self.coins_selected >= self.coins.len() {
                    self.coins_selected = self.coins.len().saturating_sub(1);
                }
            }
            WalletEvent::Transactions(txs) => {
                self.transactions = txs;
                if self.transactions_selected >= self.transactions.len() {
                    self.transactions_selected = self.transactions.len().saturating_sub(1);
                }
            }
            WalletEvent::Objects {
                objects,
                owner_alias,
            } => {
                let new_objects: Vec<ObjectDisplay> = objects
                    .into_iter()
                    .map(|o| ObjectDisplay {
                        object_id: o.object_id,
                        type_name: o.type_name,
                        version: o
                            .version
                            .map(|v| format!("v{}", v))
                            .unwrap_or_else(|| "?".into()),
                        digest: o.digest,
                        owner: o.owner,
                        owner_alias: owner_alias.clone(),
                    })
                    .collect();
                if self.visible_key_count() > 1 {
                    self.objects.extend(new_objects);
                } else {
                    self.objects = new_objects;
                }
                if self.objects_selected >= self.objects.len() {
                    self.objects_selected = self.objects.len().saturating_sub(1);
                }
            }
            WalletEvent::KeyGenerated {
                alias,
                address,
                scheme,
                private_key_hex,
            }
            | WalletEvent::KeyImported {
                alias,
                address,
                scheme,
                private_key_hex,
            } => {
                let is_first = self.keys.is_empty();
                self.keys.push(KeyDisplay {
                    alias: alias.clone(),
                    address,
                    scheme,
                    is_active: is_first,
                    visible: true,
                    private_key_hex,
                });
                self.set_status(format!("Key '{}' ready", alias));
                if is_first {
                    self.request_refresh();
                }
            }
            WalletEvent::DryRunResult(info) => {
                self.tx_dry_running = false;
                if !self.tx_gas_edited {
                    if let Some(gas) = info.estimated_gas {
                        self.tx_gas_budget = gas.to_string();
                    }
                }
                self.tx_dry_run = Some(info);
            }
            WalletEvent::TxSubmitted { digest } => {
                self.set_status(format!(
                    "Tx submitted: {}..{}",
                    &digest[..8],
                    &digest[digest.len().saturating_sub(6)..]
                ));
                self.reset_tx_builder();
                self.navigate(Screen::Transactions);
                self.request_refresh();
            }
            WalletEvent::IotaNameResolved {
                name,
                label,
                notes,
                address,
            } => {
                if let Some(addr) = address {
                    let display_label = if label.is_empty() {
                        name.clone()
                    } else {
                        label
                    };
                    let display_notes = if notes.is_empty() {
                        "IOTA-Name".into()
                    } else {
                        notes
                    };
                    self.address_book.push(AddressEntry {
                        label: display_label,
                        address: addr,
                        notes: display_notes,
                    });
                    save_address_book(&self.address_book);
                    self.set_status(format!("Resolved @{}", name));
                } else {
                    self.set_status(format!("Name '{}' not found", name));
                }
            }
            WalletEvent::FaucetRequested(msg) => {
                self.set_status(msg);
                self.request_refresh();
            }
            WalletEvent::Error(e) => {
                self.set_status(format!("Error: {}", e));
            }
        }
    }

    /// Send a command to the wallet backend (non-blocking).
    pub fn send_cmd(&self, cmd: WalletCmd) {
        let _ = self.cmd_tx.try_send(cmd);
    }

    /// Number of keys with visibility enabled.
    pub fn visible_key_count(&self) -> usize {
        self.keys.iter().filter(|k| k.visible).count()
    }

    /// Returns true when multiple keys are visible (show Owner column).
    pub fn show_multiple_owners(&self) -> bool {
        self.visible_key_count() > 1
    }

    /// Request a data refresh for visible keys' addresses.
    pub fn request_refresh(&mut self) {
        self.loading = true;
        let visible_keys: Vec<KeyDisplay> =
            self.keys.iter().filter(|k| k.visible).cloned().collect();

        if visible_keys.len() > 1 {
            self.coins.clear();
            self.objects.clear();
            self.total_balance_iota = 0;
            for key in &visible_keys {
                if let Some(addr) = parse_address(&key.address) {
                    self.send_cmd(WalletCmd::RefreshCoins {
                        addr,
                        alias: key.alias.clone(),
                    });
                    self.send_cmd(WalletCmd::RefreshObjects {
                        addr,
                        alias: key.alias.clone(),
                    });
                    self.send_cmd(WalletCmd::RefreshBalances(addr));
                }
            }
            if let Some(key) = self.active_key().cloned() {
                if let Some(addr) = parse_address(&key.address) {
                    self.send_cmd(WalletCmd::RefreshTransactions(addr));
                }
            }
        } else if let Some(key) = visible_keys.first().or(self.active_key()).cloned() {
            if let Some(addr) = parse_address(&key.address) {
                self.send_cmd(WalletCmd::RefreshCoins {
                    addr,
                    alias: key.alias.clone(),
                });
                self.send_cmd(WalletCmd::RefreshObjects {
                    addr,
                    alias: key.alias.clone(),
                });
                self.send_cmd(WalletCmd::RefreshBalances(addr));
                self.send_cmd(WalletCmd::RefreshTransactions(addr));
            }
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn clear_expired_status(&mut self) {
        if let Some((_, instant)) = &self.status_message {
            if instant.elapsed().as_secs() >= 5 {
                self.status_message = None;
            }
        }
    }

    pub fn navigate(&mut self, screen: Screen) {
        self.screen = screen;
        self.input_mode = InputMode::Normal;
        self.popup = None;
        self.popup_scroll = 0;
    }

    pub fn open_popup(&mut self, popup: Popup) {
        self.popup = Some(popup);
        self.popup_scroll = 0;
    }

    /// Calculate total IOTA nanos being transferred by all TransferIota commands.
    pub fn total_transfer_nanos(&self) -> u64 {
        self.tx_commands
            .iter()
            .filter_map(|cmd| {
                if let PtbCommand::TransferIota { amount, .. } = cmd {
                    parse_iota_amount(amount)
                } else {
                    None
                }
            })
            .sum()
    }

    /// Validate that available balance covers transfers + gas.
    pub fn validate_balance(&self) -> Result<(), String> {
        let gas_budget: u64 = self.tx_gas_budget.parse().unwrap_or(10_000_000);
        let transfer_total = self.total_transfer_nanos();
        let required = transfer_total as u128 + gas_budget as u128;
        if required > self.total_balance_iota {
            Err(format!(
                "Insufficient balance: need {} IOTA but have {}",
                format_iota(required),
                format_iota(self.total_balance_iota),
            ))
        } else {
            Ok(())
        }
    }

    pub fn reset_tx_builder(&mut self) {
        self.tx_step = TxBuilderStep::SelectSender;
        self.tx_commands.clear();
        self.tx_cmd_selected = 0;
        self.tx_gas_budget = "10000000".into();
        self.tx_edit_field = 0;
        self.tx_edit_buffers = vec![];
        self.tx_adding_cmd = None;
        self.tx_dry_run = None;
        self.tx_dry_running = false;
        self.tx_dry_run_dirty = true;
        self.tx_gas_edited = false;
    }

    pub fn load_error_log(&mut self) {
        let path = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("iota-wallet-tui")
            .join("error.log");
        self.error_log_lines = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .rev()
            .take(100)
            .map(|s| s.to_string())
            .collect();
    }

    pub fn active_key(&self) -> Option<&KeyDisplay> {
        self.keys.iter().find(|k| k.is_active)
    }

    /// Number of key-derived entries shown at the top of the address book.
    pub fn key_entry_count(&self) -> usize {
        self.keys.len()
    }

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

    /// Returns true if the current form field accepts an address (alias-completable).
    pub fn is_address_field(&self) -> bool {
        let Some(ct) = self.tx_adding_cmd else {
            return false;
        };
        matches!(
            (ct, self.tx_edit_field),
            (AddCommandType::TransferIota, 0)
                | (AddCommandType::TransferObjects, 0)
                | (AddCommandType::Stake, 1)
        )
    }

    /// Returns true if the current form field accepts an object ID.
    pub fn is_object_field(&self) -> bool {
        let Some(ct) = self.tx_adding_cmd else {
            return false;
        };
        matches!(
            (ct, self.tx_edit_field),
            (AddCommandType::TransferObjects, 1)
                | (AddCommandType::SplitCoins, 0)
                | (AddCommandType::MergeCoins, 0)
                | (AddCommandType::MergeCoins, 1)
                | (AddCommandType::Unstake, 0)
        )
    }

    /// Returns true if the current object field should suggest coins specifically.
    fn is_coin_field(&self) -> bool {
        let Some(ct) = self.tx_adding_cmd else {
            return false;
        };
        matches!(
            (ct, self.tx_edit_field),
            (AddCommandType::SplitCoins, 0)
                | (AddCommandType::MergeCoins, 0)
                | (AddCommandType::MergeCoins, 1)
        )
    }

    /// Compute autocomplete suggestions based on current input.
    pub fn update_autocomplete(&mut self) {
        let is_addr = self.is_address_field();
        let is_obj = self.is_object_field();

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
            if self.is_coin_field() {
                for coin in &self.coins {
                    let label = format!("{} ({})", coin.symbol, coin.balance_display);
                    if label.to_lowercase().contains(&query)
                        || coin.object_id.to_lowercase().contains(&query)
                    {
                        matches.push((label, coin.object_id.clone()));
                    }
                }
            } else {
                for obj in &self.objects {
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
        if let Some(idx) = self.autocomplete_idx {
            if idx >= matches.len() {
                self.autocomplete_idx = if matches.is_empty() {
                    None
                } else {
                    Some(matches.len() - 1)
                };
            }
        }
        self.autocomplete = matches;
    }

    /// Accept the currently highlighted autocomplete suggestion.
    /// Returns true if a suggestion was accepted.
    /// For address fields, inserts the alias (resolved later). For object fields, inserts the ID.
    pub fn accept_autocomplete(&mut self) -> bool {
        if self.autocomplete.is_empty() {
            return false;
        }
        let idx = self.autocomplete_idx.unwrap_or(0);
        let is_obj = self.is_object_field();
        if let Some((label, value)) = self.autocomplete.get(idx) {
            self.input_buffer = if is_obj { value.clone() } else { label.clone() };
            self.input_cursor = self.input_buffer.len();
            self.autocomplete.clear();
            self.autocomplete_idx = None;
            return true;
        }
        false
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
            _ => ("Details", vec![]),
        }
    }

    /// Adjust a scroll offset so that `selected` is visible within `visible_rows`.
    pub fn scroll_into_view(selected: usize, offset: &mut usize, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if selected < *offset {
            *offset = selected;
        } else if selected >= *offset + visible_rows {
            *offset = selected - visible_rows + 1;
        }
    }

    pub fn input_insert(&mut self, ch: char) {
        self.input_buffer.insert(self.input_cursor, ch);
        self.input_cursor += ch.len_utf8();
    }

    pub fn input_backspace(&mut self) {
        if self.input_cursor > 0 {
            let prev = self.input_buffer[..self.input_cursor]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.input_cursor -= prev;
            self.input_buffer.remove(self.input_cursor);
        }
    }

    pub fn input_delete(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_buffer.remove(self.input_cursor);
        }
    }

    pub fn input_left(&mut self) {
        if self.input_cursor > 0 {
            let prev = self.input_buffer[..self.input_cursor]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.input_cursor -= prev;
        }
    }

    pub fn input_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            let next = self.input_buffer[self.input_cursor..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.input_cursor += next;
        }
    }

    pub fn input_clear(&mut self) {
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    pub fn start_input(&mut self, initial: &str) {
        self.input_mode = InputMode::Editing;
        self.input_buffer = initial.to_string();
        self.input_cursor = initial.len();
    }

    pub fn stop_input(&mut self) -> String {
        self.input_mode = InputMode::Normal;
        let val = self.input_buffer.clone();
        self.input_clear();
        val
    }
}

// ── Address book persistence ───────────────────────────────────────

fn address_book_path() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("iota-wallet-tui")
        .join("address_book.json")
}

fn load_address_book() -> Vec<AddressEntry> {
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

// ── Helpers ────────────────────────────────────────────────────────

/// Format a raw balance (in smallest unit) as a human-readable string.
fn format_balance(raw: u128, decimals: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let whole = raw / divisor;
    let frac = raw % divisor;
    if decimals == 0 {
        return whole.to_string();
    }
    let frac_str = format!("{:0>width$}", frac, width = decimals as usize);
    let trimmed = frac_str.trim_end_matches('0');
    let display_frac = if trimmed.len() < 2 {
        &frac_str[..2]
    } else {
        trimmed
    };
    format!("{}.{}", whole, display_frac)
}

fn parse_address(hex: &str) -> Option<iota_sdk::types::Address> {
    iota_sdk::types::Address::from_hex(hex).ok()
}

/// Parse an IOTA amount string (decimal IOTA or raw nanos) into nanos.
fn parse_iota_amount(s: &str) -> Option<u64> {
    if let Ok(f) = s.parse::<f64>() {
        Some((f * 1_000_000_000.0) as u64)
    } else {
        s.parse::<u64>().ok()
    }
}

/// Format nanos as a human-readable IOTA amount.
fn format_iota(nanos: u128) -> String {
    let whole = nanos / 1_000_000_000;
    let frac = nanos % 1_000_000_000;
    if frac == 0 {
        format!("{}", whole)
    } else {
        let frac_str = format!("{:09}", frac);
        let trimmed = frac_str.trim_end_matches('0');
        format!("{}.{}", whole, trimmed)
    }
}
