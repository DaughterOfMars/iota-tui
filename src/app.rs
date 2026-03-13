use tokio::sync::mpsc;

use crate::wallet::{StoredKey, WalletCmd, WalletEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Coins,
    Objects,
    Transactions,
    Packages,
    AddressBook,
    Keys,
    TxBuilder,
}

impl Screen {
    pub const ALL: [Screen; 7] = [
        Screen::Coins,
        Screen::Objects,
        Screen::Transactions,
        Screen::Packages,
        Screen::AddressBook,
        Screen::Keys,
        Screen::TxBuilder,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Screen::Coins => "Coins",
            Screen::Objects => "Objects",
            Screen::Transactions => "Transactions",
            Screen::Packages => "Packages",
            Screen::AddressBook => "Address Book",
            Screen::Keys => "Keys",
            Screen::TxBuilder => "Tx Builder",
        }
    }

    pub fn index(self) -> usize {
        Screen::ALL.iter().position(|&s| s == self).unwrap_or(0)
    }
}

// ── Display types (what the UI renders) ────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CoinDisplay {
    pub coin_type: String,
    pub symbol: String,
    pub balance: u128,
    pub balance_display: String,
    pub object_id: String,
    pub owner_alias: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ObjectDisplay {
    pub object_id: String,
    pub type_name: String,
    pub version: String,
    pub digest: String,
    pub owner: String,
    pub owner_alias: String,
}

#[derive(Debug, Clone)]
pub struct TransactionDisplay {
    pub digest: String,
    pub status: String,
    pub gas_used: String,
    pub epoch: String,
}

#[derive(Debug, Clone)]
pub struct DryRunInfo {
    pub status: String,
    pub estimated_gas: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AddressEntry {
    pub label: String,
    pub address: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct KeyDisplay {
    pub alias: String,
    pub address: String,
    pub scheme: String,
    pub is_active: bool,
    pub private_key_hex: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxBuilderStep {
    SelectSender,
    EditCommands,
    SetGas,
    Review,
}

impl TxBuilderStep {
    pub const ALL: [TxBuilderStep; 4] = [
        TxBuilderStep::SelectSender,
        TxBuilderStep::EditCommands,
        TxBuilderStep::SetGas,
        TxBuilderStep::Review,
    ];

    pub fn title(self) -> &'static str {
        match self {
            TxBuilderStep::SelectSender => "Sender",
            TxBuilderStep::EditCommands => "Commands",
            TxBuilderStep::SetGas => "Gas",
            TxBuilderStep::Review => "Review",
        }
    }
}

/// A visual PTB command in the transaction builder
#[derive(Debug, Clone)]
pub enum PtbCommand {
    TransferIota {
        recipient: String,
        amount: String,
    },
    TransferObjects {
        recipient: String,
        object_ids: Vec<String>,
    },
    MoveCall {
        package: String,
        module: String,
        function: String,
        type_args: Vec<String>,
        args: Vec<String>,
    },
    SplitCoins {
        coin: String,
        amounts: Vec<String>,
    },
    MergeCoins {
        primary: String,
        sources: Vec<String>,
    },
    Stake {
        amount: String,
        validator: String,
    },
    Unstake {
        staked_iota_id: String,
    },
}

impl PtbCommand {
    pub fn label(&self) -> &'static str {
        match self {
            PtbCommand::TransferIota { .. } => "TransferIota",
            PtbCommand::TransferObjects { .. } => "TransferObjects",
            PtbCommand::MoveCall { .. } => "MoveCall",
            PtbCommand::SplitCoins { .. } => "SplitCoins",
            PtbCommand::MergeCoins { .. } => "MergeCoins",
            PtbCommand::Stake { .. } => "Stake",
            PtbCommand::Unstake { .. } => "Unstake",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            PtbCommand::TransferIota { recipient, amount } => {
                format!("{} IOTA -> {}", amount, truncate_id(recipient, 16))
            }
            PtbCommand::TransferObjects {
                recipient,
                object_ids,
            } => {
                format!(
                    "{} objs -> {}",
                    object_ids.len(),
                    truncate_id(recipient, 16)
                )
            }
            PtbCommand::MoveCall {
                package,
                module,
                function,
                ..
            } => {
                format!("{}::{}::{}", truncate_id(package, 8), module, function)
            }
            PtbCommand::SplitCoins { coin, amounts } => {
                format!("{} into {} parts", truncate_id(coin, 12), amounts.len())
            }
            PtbCommand::MergeCoins { primary, sources } => {
                format!("{} + {} coins", truncate_id(primary, 12), sources.len())
            }
            PtbCommand::Stake { amount, validator } => {
                format!("{} IOTA -> {}", amount, truncate_id(validator, 16))
            }
            PtbCommand::Unstake { staked_iota_id } => {
                format!("{}", truncate_id(staked_iota_id, 20))
            }
        }
    }
}

fn truncate_id(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}..{}", &s[..max / 2], &s[s.len() - max / 2..])
    }
}

/// Which command type is being added in the popup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddCommandType {
    TransferIota,
    TransferObjects,
    MoveCall,
    SplitCoins,
    MergeCoins,
    Stake,
    Unstake,
}

impl AddCommandType {
    pub fn label(self) -> &'static str {
        match self {
            AddCommandType::TransferIota => "Transfer IOTA",
            AddCommandType::TransferObjects => "Transfer Objects",
            AddCommandType::MoveCall => "Move Call",
            AddCommandType::SplitCoins => "Split Coins",
            AddCommandType::MergeCoins => "Merge Coins",
            AddCommandType::Stake => "Stake",
            AddCommandType::Unstake => "Unstake",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popup {
    Help,
    AddAddress,
    EditAddress,
    GenerateKey,
    ImportKey,
    AddCommand,
    AddCommandForm,
    RenameKey,
    SwitchNetwork,
    Detail,
    ConfirmDeleteAddress,
    ConfirmDeleteKey,
    ConfirmQuit,
}

// ── App State ──────────────────────────────────────────────────────

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

    // Show data from all owned addresses
    pub show_all_addresses: bool,

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
                private_key_hex: hex::encode(&k.private_key_bytes),
            })
            .collect();

        // Load saved address book
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

            show_all_addresses: false,

            popup_scroll: 0,

            tab_areas: vec![],
        }
    }

    /// Handle a response from the wallet backend
    pub fn handle_wallet_event(&mut self, event: WalletEvent) {
        self.loading = false;
        match event {
            WalletEvent::Connected(network) => {
                self.connected = true;
                self.network_name = network;
                self.set_status("Connected");
                // Auto-refresh if we have an active key
                self.request_refresh();
            }
            WalletEvent::Balances(balances) => {
                for b in &balances {
                    if b.coin_type.contains("IOTA") {
                        if self.show_all_addresses {
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
                if self.show_all_addresses {
                    // Append (data was cleared at refresh start)
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
                if self.show_all_addresses {
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
                    private_key_hex,
                });
                self.set_status(format!("Key '{}' ready", alias));
                // If this is the first key, refresh data
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
                // Clear the transaction builder
                self.reset_tx_builder();
                // Navigate to transaction screen to see the result
                self.navigate(Screen::Transactions);
                // Refresh after transaction
                self.request_refresh();
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

    /// Send a command to the wallet backend (non-blocking)
    pub fn send_cmd(&self, cmd: WalletCmd) {
        let _ = self.cmd_tx.try_send(cmd);
    }

    /// Request a data refresh for the active key's address (or all keys)
    pub fn request_refresh(&mut self) {
        self.loading = true;
        if self.show_all_addresses {
            // Clear existing data before fetching from all addresses
            self.coins.clear();
            self.objects.clear();
            self.total_balance_iota = 0;
            for key in self.keys.clone() {
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
            // Transactions only for active key
            if let Some(key) = self.active_key().cloned() {
                if let Some(addr) = parse_address(&key.address) {
                    self.send_cmd(WalletCmd::RefreshTransactions(addr));
                }
            }
        } else if let Some(key) = self.active_key().cloned() {
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

    /// Compute autocomplete suggestions based on current input.
    pub fn update_autocomplete(&mut self) {
        if !self.is_address_field() || self.input_buffer.is_empty() {
            self.autocomplete.clear();
            self.autocomplete_idx = None;
            return;
        }

        // Don't suggest if it looks like a raw address
        if self.input_buffer.starts_with("0x") {
            self.autocomplete.clear();
            self.autocomplete_idx = None;
            return;
        }

        let query = self.input_buffer.to_lowercase();
        let mut matches: Vec<(String, String)> = Vec::new();

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
    pub fn accept_autocomplete(&mut self) -> bool {
        if self.autocomplete.is_empty() {
            return false;
        }
        let idx = self.autocomplete_idx.unwrap_or(0);
        if let Some((label, _)) = self.autocomplete.get(idx) {
            self.input_buffer = label.clone();
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
        // Check key aliases
        for key in &self.keys {
            if key.alias.to_lowercase() == input_lower {
                return key.address.clone();
            }
        }
        // Check address book labels
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
                    if self.show_all_addresses {
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
                    if self.show_all_addresses {
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

// ── Persistence for address book ───────────────────────────────────

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

/// Format a raw balance (in smallest unit) as a human-readable string
fn format_balance(raw: u128, decimals: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let whole = raw / divisor;
    let frac = raw % divisor;
    if decimals == 0 {
        return whole.to_string();
    }
    let frac_str = format!("{:0>width$}", frac, width = decimals as usize);
    // Trim trailing zeros but keep at least 2 decimal places
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

// Make AddressEntry serializable for persistence
impl serde::Serialize for AddressEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AddressEntry", 3)?;
        s.serialize_field("label", &self.label)?;
        s.serialize_field("address", &self.address)?;
        s.serialize_field("notes", &self.notes)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for AddressEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Helper {
            label: String,
            address: String,
            notes: String,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(AddressEntry {
            label: h.label,
            address: h.address,
            notes: h.notes,
        })
    }
}
