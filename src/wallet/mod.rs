//! Wallet backend — handles network communication, key management, and transaction execution.

mod handlers;
mod helpers;
mod lookup;

use std::path::PathBuf;

use iota_sdk::crypto::simple::SimpleKeypair;
use iota_sdk::graphql_client::{Client, faucet::FaucetClient};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use helpers::log_error;

// ── Types for the TUI ──────────────────────────────────────────────

/// Raw coin data returned from the network.
#[derive(Debug, Clone)]
pub struct CoinInfo {
    pub coin_type: String,
    pub symbol: String,
    pub balance: u128,
    pub object_id: String,
}

/// Raw object data returned from the network.
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub object_id: String,
    pub type_name: String,
    pub version: Option<u64>,
    pub digest: String,
    pub owner: String,
}

/// Aggregated balance for a coin type.
#[derive(Debug, Clone)]
pub struct BalanceInfo {
    pub coin_type: String,
    pub total_balance: u128,
}

/// A key persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKey {
    pub alias: String,
    pub scheme: String,
    pub private_key_bytes: Vec<u8>,
    pub address: String,
    pub is_active: bool,
}

/// IOTA network to connect to.
#[derive(Debug, Clone)]
pub enum Network {
    Mainnet,
    Testnet,
    Devnet,
    Custom(String),
}

impl Network {
    pub fn name(&self) -> &str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
            Network::Devnet => "devnet",
            Network::Custom(url) => url.as_str(),
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "mainnet" => Network::Mainnet,
            "testnet" => Network::Testnet,
            "devnet" => Network::Devnet,
            url => Network::Custom(url.to_string()),
        }
    }
}

fn network_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("iota-wallet-tui")
        .join("network.txt")
}

pub fn save_network(network: &Network) {
    let path = network_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, network.name());
}

pub fn load_network() -> Network {
    let path = network_config_path();
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| Network::from_name(s.trim()))
        .unwrap_or(Network::Testnet)
}

fn theme_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("iota-wallet-tui")
        .join("theme.txt")
}

pub fn save_theme(active: bool) {
    let path = theme_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, if active { "1" } else { "0" });
}

pub fn load_theme() -> u32 {
    let path = theme_config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| if s.trim() == "1" { Some(1) } else { None })
        .unwrap_or(0)
}

fn sidebar_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("iota-wallet-tui")
        .join("sidebar.txt")
}

pub fn save_sidebar_collapsed(collapsed: bool) {
    let path = sidebar_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, if collapsed { "1" } else { "0" });
}

pub fn load_sidebar_collapsed() -> bool {
    let path = sidebar_config_path();
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim() == "1")
        .unwrap_or(false)
}

// ── Commands and Responses ─────────────────────────────────────────

/// Commands sent from the UI to the wallet backend.
#[derive(Debug)]
pub enum WalletCmd {
    Connect(Network),
    RefreshBalances(iota_sdk::types::Address),
    RefreshCoins {
        addr: iota_sdk::types::Address,
        alias: String,
    },
    RefreshObjects {
        addr: iota_sdk::types::Address,
        alias: String,
    },
    RefreshTransactions(iota_sdk::types::Address),
    GenerateKey {
        scheme: String,
        alias: String,
    },
    ImportKey {
        scheme: String,
        private_key_hex: String,
        alias: String,
    },
    ExecutePtb {
        sender_idx: usize,
        commands: Vec<crate::app::PtbCommand>,
        gas_budget: u64,
    },
    DryRun {
        sender_idx: usize,
        commands: Vec<crate::app::PtbCommand>,
    },
    DeleteKey(usize),
    SetActiveKey(usize),
    RenameKey {
        idx: usize,
        new_alias: String,
    },
    RequestFaucet(iota_sdk::types::Address),
    LookupIotaName {
        name: String,
        label: String,
        notes: String,
    },
    // Explorer commands
    RefreshNetworkOverview,
    RefreshCheckpoints {
        cursor: Option<String>,
    },
    RefreshValidators,
    LookupAddress(String),
    SearchObjectsByType {
        type_filter: String,
        cursor: Option<String>,
    },
    LookupAddressPage {
        address: String,
        obj_cursor: Option<String>,
        tx_cursor: Option<String>,
    },
    RefreshStakes(iota_sdk::types::Address),
    FetchPackageModules {
        package_addr: String,
    },
    FetchModuleFunctions {
        package_addr: String,
        module_name: String,
    },
}

/// Events sent from the wallet backend back to the UI.
#[derive(Debug)]
pub enum WalletEvent {
    Connected(String),
    Balances(Vec<BalanceInfo>),
    Coins {
        coins: Vec<CoinInfo>,
        owner_alias: String,
    },
    Objects {
        objects: Vec<ObjectInfo>,
        owner_alias: String,
    },
    Transactions(Vec<crate::app::TransactionDisplay>),
    KeyGenerated {
        alias: String,
        address: String,
        scheme: String,
        private_key_hex: String,
    },
    KeyImported {
        alias: String,
        address: String,
        scheme: String,
        private_key_hex: String,
    },
    DryRunResult(crate::app::DryRunInfo),
    TxSubmitted,
    FaucetRequested(String),
    IotaNameResolved {
        name: String,
        label: String,
        notes: String,
        address: Option<String>,
    },
    // Explorer events
    NetworkOverview {
        chain_id: String,
        epoch: String,
        gas_price: String,
        latest_checkpoint: String,
        total_transactions: String,
    },
    Checkpoints {
        checkpoints: Vec<crate::app::CheckpointDisplay>,
        cursor: Option<String>,
        has_next: bool,
    },
    Validators(Vec<crate::app::ValidatorDisplay>),
    ExplorerLookupResult(crate::app::LookupResult),
    AddressLookupPage {
        result: crate::app::LookupResult,
        obj_cursor: Option<String>,
        obj_has_next: bool,
        tx_cursor: Option<String>,
        tx_has_next: bool,
    },
    ObjectSearchResults {
        objects: Vec<crate::app::ObjectDisplay>,
        has_next_page: bool,
        end_cursor: Option<String>,
    },
    Stakes(Vec<crate::app::StakeDisplay>),
    PackageModules {
        package_addr: String,
        modules: Vec<crate::app::PackageModuleDisplay>,
    },
    ModuleFunctions {
        module_name: String,
        functions: Vec<crate::app::ModuleFunctionDisplay>,
    },
    Error(String),
}

// ── Wallet Backend ─────────────────────────────────────────────────

pub struct WalletBackend {
    pub(super) client: Option<Client>,
    pub(super) faucet: Option<FaucetClient>,
    pub(super) keys: Vec<StoredKey>,
    pub(super) keypairs: Vec<SimpleKeypair>,
    pub(super) keystore_path: PathBuf,
    pub(super) cmd_rx: mpsc::Receiver<WalletCmd>,
    pub(super) event_tx: mpsc::Sender<WalletEvent>,
}

impl WalletBackend {
    pub fn new(cmd_rx: mpsc::Receiver<WalletCmd>, event_tx: mpsc::Sender<WalletEvent>) -> Self {
        let keystore_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("iota-wallet-tui")
            .join("keystore.json");

        let mut backend = WalletBackend {
            client: None,
            faucet: None,
            keys: Vec::new(),
            keypairs: Vec::new(),
            keystore_path,
            cmd_rx,
            event_tx,
        };
        backend.load_keys();
        backend
    }

    pub async fn run(mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            let result = match cmd {
                WalletCmd::Connect(network) => self.handle_connect(network).await,
                WalletCmd::RefreshBalances(addr) => self.handle_balances(addr).await,
                WalletCmd::RefreshCoins { addr, alias } => self.handle_coins(addr, alias).await,
                WalletCmd::RefreshObjects { addr, alias } => self.handle_objects(addr, alias).await,
                WalletCmd::RefreshTransactions(addr) => self.handle_transactions(addr).await,
                WalletCmd::GenerateKey { scheme, alias } => {
                    self.handle_generate_key(&scheme, &alias)
                }
                WalletCmd::ImportKey {
                    scheme,
                    private_key_hex,
                    alias,
                } => self.handle_import_key(&scheme, &private_key_hex, &alias),
                WalletCmd::ExecutePtb {
                    sender_idx,
                    commands,
                    gas_budget,
                } => {
                    self.handle_execute_ptb(sender_idx, commands, gas_budget)
                        .await
                }
                WalletCmd::DryRun {
                    sender_idx,
                    commands,
                } => self.handle_dry_run(sender_idx, commands).await,
                WalletCmd::DeleteKey(idx) => self.handle_delete_key(idx),
                WalletCmd::SetActiveKey(idx) => self.handle_set_active_key(idx),
                WalletCmd::RenameKey { idx, new_alias } => self.handle_rename_key(idx, &new_alias),
                WalletCmd::RequestFaucet(addr) => self.handle_faucet(addr).await,
                WalletCmd::LookupIotaName { name, label, notes } => {
                    self.handle_iota_name_lookup(&name, &label, &notes).await
                }
                WalletCmd::RefreshNetworkOverview => self.handle_network_overview().await,
                WalletCmd::RefreshCheckpoints { cursor } => self.handle_checkpoints(cursor).await,
                WalletCmd::RefreshValidators => self.handle_validators().await,
                WalletCmd::LookupAddress(query) => self.handle_lookup(&query).await,
                WalletCmd::LookupAddressPage {
                    address,
                    obj_cursor,
                    tx_cursor,
                } => {
                    self.handle_address_page(&address, obj_cursor, tx_cursor)
                        .await
                }
                WalletCmd::SearchObjectsByType {
                    type_filter,
                    cursor,
                } => {
                    self.handle_search_objects_by_type(&type_filter, cursor)
                        .await
                }
                WalletCmd::RefreshStakes(addr) => self.handle_stakes(addr).await,
                WalletCmd::FetchPackageModules { package_addr } => {
                    self.handle_fetch_package_modules(&package_addr).await
                }
                WalletCmd::FetchModuleFunctions {
                    package_addr,
                    module_name,
                } => {
                    self.handle_fetch_module_functions(&package_addr, &module_name)
                        .await
                }
            };

            if let Err(e) = result {
                log_error(&e.to_string());
                let _ = self.event_tx.send(WalletEvent::Error(e.to_string())).await;
            }
        }
    }

    pub fn stored_keys(&self) -> &[StoredKey] {
        &self.keys
    }
}
