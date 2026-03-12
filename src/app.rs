use tokio::sync::mpsc;

use crate::wallet::{StoredKey, WalletCmd, WalletEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Coins,
    Objects,
    Packages,
    AddressBook,
    Keys,
    TxBuilder,
}

impl Screen {
    pub const ALL: [Screen; 6] = [
        Screen::Coins,
        Screen::Objects,
        Screen::Packages,
        Screen::AddressBook,
        Screen::Keys,
        Screen::TxBuilder,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Screen::Coins => "Coins",
            Screen::Objects => "Objects",
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
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ObjectDisplay {
    pub object_id: String,
    pub type_name: String,
    pub version: String,
    pub digest: String,
    pub owner: String,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxBuilderStep {
    SelectSender,
    AddRecipients,
    SetGas,
    Review,
}

impl TxBuilderStep {
    pub const ALL: [TxBuilderStep; 4] = [
        TxBuilderStep::SelectSender,
        TxBuilderStep::AddRecipients,
        TxBuilderStep::SetGas,
        TxBuilderStep::Review,
    ];

    pub fn title(self) -> &'static str {
        match self {
            TxBuilderStep::SelectSender => "Sender",
            TxBuilderStep::AddRecipients => "Recipients",
            TxBuilderStep::SetGas => "Gas",
            TxBuilderStep::Review => "Review",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TxRecipient {
    pub address: String,
    pub amount: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popup {
    Help,
    Confirm,
    AddAddress,
    EditAddress,
    GenerateKey,
    ImportKey,
    AddRecipient,
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
    pub total_balance_iota: u128,

    pub objects: Vec<ObjectDisplay>,
    pub objects_selected: usize,

    pub address_book: Vec<AddressEntry>,
    pub address_selected: usize,
    pub address_edit_field: usize,
    pub address_edit_buffers: [String; 3],

    pub keys: Vec<KeyDisplay>,
    pub keys_selected: usize,
    pub keys_show_private: bool,

    pub tx_step: TxBuilderStep,
    pub tx_sender: usize,
    pub tx_recipients: Vec<TxRecipient>,
    pub tx_recipient_selected: usize,
    pub tx_gas_budget: String,
    pub tx_edit_field: usize,
    pub tx_edit_buffers: [String; 2],

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
            total_balance_iota: 0,

            objects: vec![],
            objects_selected: 0,

            address_book,
            address_selected: 0,
            address_edit_field: 0,
            address_edit_buffers: [String::new(), String::new(), String::new()],

            keys,
            keys_selected: 0,
            keys_show_private: false,

            tx_step: TxBuilderStep::SelectSender,
            tx_sender: 0,
            tx_recipients: vec![],
            tx_recipient_selected: 0,
            tx_gas_budget: "10000000".into(),
            tx_edit_field: 0,
            tx_edit_buffers: [String::new(), String::new()],

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
                        self.total_balance_iota = b.total_balance;
                    }
                }
            }
            WalletEvent::Coins(coins) => {
                self.coins = coins
                    .into_iter()
                    .map(|c| CoinDisplay {
                        balance_display: format_balance(c.balance, 9),
                        coin_type: c.coin_type,
                        symbol: c.symbol,
                        balance: c.balance,
                        object_id: c.object_id,
                    })
                    .collect();
                if self.coins_selected >= self.coins.len() {
                    self.coins_selected = self.coins.len().saturating_sub(1);
                }
            }
            WalletEvent::Objects(objects) => {
                self.objects = objects
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
                    })
                    .collect();
                if self.objects_selected >= self.objects.len() {
                    self.objects_selected = self.objects.len().saturating_sub(1);
                }
            }
            WalletEvent::KeyGenerated {
                alias,
                address,
                scheme,
            }
            | WalletEvent::KeyImported {
                alias,
                address,
                scheme,
            } => {
                let is_first = self.keys.is_empty();
                self.keys.push(KeyDisplay {
                    alias: alias.clone(),
                    address,
                    scheme,
                    is_active: is_first,
                });
                self.set_status(format!("Key '{}' ready", alias));
                // If this is the first key, refresh data
                if is_first {
                    self.request_refresh();
                }
            }
            WalletEvent::TxSubmitted { digest } => {
                self.set_status(format!(
                    "Tx submitted: {}..{}",
                    &digest[..8],
                    &digest[digest.len().saturating_sub(6)..]
                ));
                self.popup = Some(Popup::Confirm);
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

    /// Request a data refresh for the active key's address
    pub fn request_refresh(&self) {
        if let Some(key) = self.active_key() {
            if let Some(addr) = parse_address(&key.address) {
                self.send_cmd(WalletCmd::RefreshCoins(addr));
                self.send_cmd(WalletCmd::RefreshObjects(addr));
                self.send_cmd(WalletCmd::RefreshBalances(addr));
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
    }

    pub fn active_key(&self) -> Option<&KeyDisplay> {
        self.keys.iter().find(|k| k.is_active)
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
