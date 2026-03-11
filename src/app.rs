use rand::prelude::*;

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

#[derive(Debug, Clone)]
pub struct Coin {
    pub name: String,
    pub symbol: String,
    pub balance: f64,
    pub usd_value: f64,
    pub change_24h: f64,
    pub object_id: String,
}

#[derive(Debug, Clone)]
pub struct OwnedObject {
    pub object_id: String,
    pub type_name: String,
    pub version: u64,
    pub digest: String,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub package_id: String,
    pub name: String,
    pub version: u64,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AddressEntry {
    pub label: String,
    pub address: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
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

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub popup: Option<Popup>,
    pub status_message: Option<(String, std::time::Instant)>,

    // Per-screen state
    pub coins: Vec<Coin>,
    pub coins_selected: usize,

    pub objects: Vec<OwnedObject>,
    pub objects_selected: usize,
    pub _objects_scroll: usize,

    pub packages: Vec<Package>,
    pub packages_selected: usize,
    pub packages_expanded: Option<usize>,

    pub address_book: Vec<AddressEntry>,
    pub address_selected: usize,
    pub address_edit_field: usize,
    pub address_edit_buffers: [String; 3],

    pub keys: Vec<KeyEntry>,
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

pub fn random_hex_pub(len: usize) -> String {
    random_hex(len)
}

fn random_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| format!("{:x}", rng.gen_range(0..16u8))).collect()
}

fn mock_address() -> String {
    format!("0x{}", random_hex(64))
}

fn mock_object_id() -> String {
    format!("0x{}", random_hex(64))
}

fn mock_digest() -> String {
    random_hex(44)
}

impl App {
    pub fn new() -> Self {
        let coins = vec![
            Coin {
                name: "IOTA".into(),
                symbol: "IOTA".into(),
                balance: 15_420.50,
                usd_value: 4_312.94,
                change_24h: 3.24,
                object_id: mock_object_id(),
            },
            Coin {
                name: "Deepbook Token".into(),
                symbol: "DEEP".into(),
                balance: 2_500.0,
                usd_value: 125.00,
                change_24h: -1.82,
                object_id: mock_object_id(),
            },
            Coin {
                name: "USD Coin".into(),
                symbol: "USDC".into(),
                balance: 1_000.0,
                usd_value: 1_000.0,
                change_24h: 0.01,
                object_id: mock_object_id(),
            },
            Coin {
                name: "Wrapped ETH".into(),
                symbol: "wETH".into(),
                balance: 0.85,
                usd_value: 2_720.00,
                change_24h: 1.45,
                object_id: mock_object_id(),
            },
        ];

        let objects = vec![
            OwnedObject {
                object_id: mock_object_id(),
                type_name: "0x2::coin::Coin<0x2::iota::IOTA>".into(),
                version: 42,
                digest: mock_digest(),
            },
            OwnedObject {
                object_id: mock_object_id(),
                type_name: "0x2::kiosk::Kiosk".into(),
                version: 18,
                digest: mock_digest(),
            },
            OwnedObject {
                object_id: mock_object_id(),
                type_name: "0xdee9::clob_v2::Pool<0x2::iota::IOTA, 0xusdc::usdc::USDC>".into(),
                version: 103,
                digest: mock_digest(),
            },
            OwnedObject {
                object_id: mock_object_id(),
                type_name: "0x2::display::Display<0xnft::collection::NFT>".into(),
                version: 7,
                digest: mock_digest(),
            },
            OwnedObject {
                object_id: mock_object_id(),
                type_name: "0x2::coin::Coin<0xdeep::deep::DEEP>".into(),
                version: 55,
                digest: mock_digest(),
            },
            OwnedObject {
                object_id: mock_object_id(),
                type_name: "0x2::token::Token<0xusdc::usdc::USDC>".into(),
                version: 12,
                digest: mock_digest(),
            },
        ];

        let packages = vec![
            Package {
                package_id: mock_object_id(),
                name: "my_defi_protocol".into(),
                version: 3,
                modules: vec!["pool".into(), "router".into(), "oracle".into()],
            },
            Package {
                package_id: mock_object_id(),
                name: "nft_collection".into(),
                version: 1,
                modules: vec!["collection".into(), "mint".into(), "metadata".into(), "royalty".into()],
            },
            Package {
                package_id: mock_object_id(),
                name: "governance".into(),
                version: 2,
                modules: vec!["proposal".into(), "voting".into(), "treasury".into()],
            },
        ];

        let address_book = vec![
            AddressEntry {
                label: "My Main Wallet".into(),
                address: mock_address(),
                notes: "Primary trading account".into(),
            },
            AddressEntry {
                label: "Cold Storage".into(),
                address: mock_address(),
                notes: "Long-term holdings".into(),
            },
            AddressEntry {
                label: "Alice".into(),
                address: mock_address(),
                notes: "Friend's address".into(),
            },
            AddressEntry {
                label: "DEX Router".into(),
                address: mock_address(),
                notes: "DeepBook router contract".into(),
            },
        ];

        let keys = vec![
            KeyEntry {
                alias: "default".into(),
                address: mock_address(),
                scheme: "ed25519".into(),
                is_active: true,
            },
            KeyEntry {
                alias: "trading".into(),
                address: mock_address(),
                scheme: "secp256k1".into(),
                is_active: false,
            },
            KeyEntry {
                alias: "cold".into(),
                address: mock_address(),
                scheme: "ed25519".into(),
                is_active: false,
            },
        ];

        App {
            running: true,
            screen: Screen::Coins,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_cursor: 0,
            popup: None,
            status_message: None,

            coins,
            coins_selected: 0,

            objects,
            objects_selected: 0,
            _objects_scroll: 0,

            packages,
            packages_selected: 0,
            packages_expanded: None,

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

    pub fn active_key(&self) -> Option<&KeyEntry> {
        self.keys.iter().find(|k| k.is_active)
    }

    pub fn total_usd_value(&self) -> f64 {
        self.coins.iter().map(|c| c.usd_value).sum()
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
