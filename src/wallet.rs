//! Wallet backend — handles network communication, key management, and transaction execution.

use std::fmt::Write;
use std::path::PathBuf;

use base64ct::Encoding;
use iota_sdk::crypto::{
    ToFromBytes, ed25519::Ed25519PrivateKey, secp256k1::Secp256k1PrivateKey,
    secp256r1::Secp256r1PrivateKey, simple::SimpleKeypair,
};
use iota_sdk::graphql_client::{
    Client, Direction, PaginationFilter, faucet::FaucetClient, query_types::ObjectFilter,
};
use iota_sdk::transaction_builder::TransactionBuilder;
use iota_sdk::types::{Address, ObjectType, StructTag, TypeTag};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

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

// ── Commands and Responses ─────────────────────────────────────────

/// Commands sent from the UI to the wallet backend.
#[derive(Debug)]
pub enum WalletCmd {
    Connect(Network),
    RefreshBalances(Address),
    RefreshCoins {
        addr: Address,
        alias: String,
    },
    RefreshObjects {
        addr: Address,
        alias: String,
    },
    RefreshTransactions(Address),
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
    RequestFaucet(Address),
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
    TxSubmitted {
        digest: String,
    },
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
    Error(String),
}

// ── Wallet Backend ─────────────────────────────────────────────────

pub struct WalletBackend {
    client: Option<Client>,
    faucet: Option<FaucetClient>,
    keys: Vec<StoredKey>,
    keypairs: Vec<SimpleKeypair>,
    keystore_path: PathBuf,
    cmd_rx: mpsc::Receiver<WalletCmd>,
    event_tx: mpsc::Sender<WalletEvent>,
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
            };

            if let Err(e) = result {
                log_error(&e.to_string());
                let _ = self.event_tx.send(WalletEvent::Error(e.to_string())).await;
            }
        }
    }

    async fn handle_connect(
        &mut self,
        network: Network,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (client, faucet) = match &network {
            Network::Mainnet => (Client::new_mainnet(), None),
            Network::Testnet => (Client::new_testnet(), Some(FaucetClient::new_testnet())),
            Network::Devnet => (Client::new_devnet(), Some(FaucetClient::new_devnet())),
            Network::Custom(url) => (Client::new(url)?, None),
        };
        self.client = Some(client);
        self.faucet = faucet;
        save_network(&network);
        self.event_tx
            .send(WalletEvent::Connected(network.name().to_string()))
            .await?;
        Ok(())
    }

    async fn handle_balances(
        &self,
        addr: Address,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let iota_balance = client.balance(addr, None::<String>).await?;
        let balances = vec![BalanceInfo {
            coin_type: "0x2::iota::IOTA".into(),
            total_balance: iota_balance.unwrap_or(0) as u128,
        }];

        self.event_tx.send(WalletEvent::Balances(balances)).await?;
        Ok(())
    }

    async fn handle_coins(
        &self,
        addr: Address,
        alias: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let page = client
            .coins(addr, None, PaginationFilter::default())
            .await?;
        let coins: Vec<CoinInfo> = page
            .data()
            .iter()
            .map(|c| CoinInfo {
                coin_type: prettify_type(c.coin_type()),
                symbol: extract_symbol(&c.coin_type().to_string()),
                balance: c.balance() as u128,
                object_id: c.id().to_string(),
            })
            .collect();

        self.event_tx
            .send(WalletEvent::Coins {
                coins,
                owner_alias: alias,
            })
            .await?;
        Ok(())
    }

    async fn handle_objects(
        &self,
        addr: Address,
        alias: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let filter = ObjectFilter {
            owner: Some(addr),
            type_: None,
            object_ids: None,
        };

        let page = client.objects(filter, PaginationFilter::default()).await?;
        let objects: Vec<ObjectInfo> = page
            .data()
            .iter()
            .map(|obj| {
                let type_name = match obj.object_type() {
                    ObjectType::Struct(s) => prettify_struct(&s),
                    ObjectType::Package => "Package".into(),
                };
                let owner = format!("{:?}", obj.owner);

                ObjectInfo {
                    object_id: obj.object_id().to_string(),
                    type_name,
                    version: Some(obj.version()),
                    digest: obj.previous_transaction.to_string(),
                    owner,
                }
            })
            .collect();

        self.event_tx
            .send(WalletEvent::Objects {
                objects,
                owner_alias: alias,
            })
            .await?;
        Ok(())
    }

    async fn handle_transactions(
        &self,
        addr: Address,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use iota_sdk::graphql_client::query_types::TransactionsFilter;

        let client = self.client.as_ref().ok_or("Not connected")?;

        let filter = TransactionsFilter {
            sign_address: Some(addr),
            ..Default::default()
        };

        let page = client
            .transactions_effects(
                filter,
                PaginationFilter {
                    direction: Direction::Backward,
                    ..PaginationFilter::default()
                },
            )
            .await?;

        let txs: Vec<crate::app::TransactionDisplay> = page
            .data()
            .iter()
            .map(|effects| match effects {
                iota_sdk::types::TransactionEffects::V1(v1) => {
                    let status = match &v1.status {
                        iota_sdk::types::ExecutionStatus::Success => "Success".to_string(),
                        iota_sdk::types::ExecutionStatus::Failure { error, .. } => {
                            format!("Failed: {:?}", error)
                        }
                        _ => "Unknown (unsupported status variant)".to_string(),
                    };
                    let gas = &v1.gas_used;
                    let total_gas = gas.computation_cost + gas.storage_cost
                        - gas.storage_rebate.min(gas.storage_cost);
                    crate::app::TransactionDisplay {
                        digest: v1.transaction_digest.to_string(),
                        status,
                        gas_used: format_gas(total_gas),
                        epoch: format!("{}", v1.epoch),
                    }
                }
                _ => crate::app::TransactionDisplay {
                    digest: "?".into(),
                    status: "Unsupported effects version".into(),
                    gas_used: "?".into(),
                    epoch: "?".into(),
                },
            })
            .collect();

        self.event_tx.send(WalletEvent::Transactions(txs)).await?;
        Ok(())
    }

    fn handle_generate_key(
        &mut self,
        scheme: &str,
        alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (keypair, address_str) = generate_keypair(scheme)?;
        let bytes = keypair.to_bytes();
        let key_hex = hex::encode(&bytes);

        let stored = StoredKey {
            alias: alias.to_string(),
            scheme: scheme.to_string(),
            private_key_bytes: bytes,
            address: address_str.clone(),
            is_active: self.keys.is_empty(),
        };

        self.keys.push(stored);
        self.keypairs.push(keypair);
        self.save_keys();

        let event = WalletEvent::KeyGenerated {
            alias: alias.to_string(),
            address: address_str,
            scheme: scheme.to_string(),
            private_key_hex: key_hex,
        };
        let _ = self.event_tx.try_send(event);
        Ok(())
    }

    fn handle_import_key(
        &mut self,
        scheme: &str,
        key_input: &str,
        alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (keypair, detected_scheme) = decode_private_key(scheme, key_input)?;
        let address_str = keypair_address(&keypair);
        let stored_bytes = keypair.to_bytes();
        let key_hex = hex::encode(&stored_bytes);

        let stored = StoredKey {
            alias: alias.to_string(),
            scheme: detected_scheme.clone(),
            private_key_bytes: stored_bytes,
            address: address_str.clone(),
            is_active: self.keys.is_empty(),
        };

        self.keys.push(stored);
        self.keypairs.push(keypair);
        self.save_keys();

        let event = WalletEvent::KeyImported {
            alias: alias.to_string(),
            address: address_str,
            scheme: detected_scheme,
            private_key_hex: key_hex,
        };
        let _ = self.event_tx.try_send(event);
        Ok(())
    }

    fn handle_rename_key(
        &mut self,
        idx: usize,
        new_alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(key) = self.keys.get_mut(idx) {
            key.alias = new_alias.to_string();
            self.save_keys();
        }
        Ok(())
    }

    fn handle_delete_key(
        &mut self,
        idx: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if idx < self.keys.len() {
            self.keys.remove(idx);
            self.keypairs.remove(idx);
            self.save_keys();
        }
        Ok(())
    }

    fn handle_set_active_key(
        &mut self,
        idx: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (i, key) in self.keys.iter_mut().enumerate() {
            key.is_active = i == idx;
        }
        self.save_keys();
        Ok(())
    }

    async fn handle_execute_ptb(
        &self,
        sender_idx: usize,
        commands: Vec<crate::app::PtbCommand>,
        gas_budget: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;
        let keypair = self
            .keypairs
            .get(sender_idx)
            .ok_or("Invalid sender key index")?;
        let sender_addr = Address::from_hex(&self.keys[sender_idx].address)?;

        let mut builder = TransactionBuilder::new(sender_addr).with_client(client);

        for cmd in &commands {
            match cmd {
                crate::app::PtbCommand::TransferIota { recipient, amount } => {
                    let addr = Address::from_hex(recipient)?;
                    let nanos = parse_iota_amount(amount)?;
                    builder.send_iota(addr, nanos);
                }
                crate::app::PtbCommand::TransferObjects {
                    recipient,
                    object_ids,
                } => {
                    let addr = Address::from_hex(recipient)?;
                    let ids: Result<Vec<iota_sdk::types::ObjectId>, _> = object_ids
                        .iter()
                        .map(|id| id.parse::<iota_sdk::types::ObjectId>())
                        .collect();
                    builder.transfer_objects(addr, ids?);
                }
                crate::app::PtbCommand::MoveCall {
                    package,
                    module,
                    function,
                    type_args,
                    args,
                } => {
                    let pkg_addr = Address::from_hex(package)?;
                    let call = builder.move_call(pkg_addr, module.as_str(), function.as_str());
                    if !type_args.is_empty() {
                        let tags: Vec<iota_sdk::types::TypeTag> = type_args
                            .iter()
                            .map(|s| s.parse::<iota_sdk::types::TypeTag>())
                            .collect::<Result<Vec<_>, _>>()?;
                        call.type_tags(tags);
                    }
                    // TODO: argument parsing for move calls is complex;
                    // for now we pass no extra args (works for 0-arg functions)
                    let _ = args;
                }
                crate::app::PtbCommand::SplitCoins { coin, amounts } => {
                    let coin_id: iota_sdk::types::ObjectId = coin.parse()?;
                    let parsed: Result<Vec<u64>, _> =
                        amounts.iter().map(|a| parse_iota_amount(a)).collect();
                    builder.split_coins(coin_id, parsed?);
                }
                crate::app::PtbCommand::MergeCoins { primary, sources } => {
                    let primary_id: iota_sdk::types::ObjectId = primary.parse()?;
                    let source_ids: Result<Vec<iota_sdk::types::ObjectId>, _> = sources
                        .iter()
                        .map(|id| id.parse::<iota_sdk::types::ObjectId>())
                        .collect();
                    builder.merge_coins(primary_id, source_ids?);
                }
                crate::app::PtbCommand::Stake { amount, validator } => {
                    let nanos = parse_iota_amount(amount)?;
                    let validator_addr = Address::from_hex(validator)?;
                    builder.stake(nanos, validator_addr);
                }
                crate::app::PtbCommand::Unstake { staked_iota_id } => {
                    let obj_id: iota_sdk::types::ObjectId = staked_iota_id.parse()?;
                    builder.unstake(obj_id);
                }
            }
        }

        builder.gas_budget(gas_budget);
        let effects = builder.execute(keypair, None).await?;

        let digest = match &effects {
            iota_sdk::types::TransactionEffects::V1(v1) => v1.transaction_digest.to_string(),
            _ => "unknown".to_string(),
        };
        self.event_tx
            .send(WalletEvent::TxSubmitted { digest })
            .await?;
        Ok(())
    }

    async fn handle_dry_run(
        &self,
        sender_idx: usize,
        commands: Vec<crate::app::PtbCommand>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;
        let sender_addr = Address::from_hex(&self.keys[sender_idx].address)?;

        let mut builder = TransactionBuilder::new(sender_addr).with_client(client);

        for cmd in &commands {
            match cmd {
                crate::app::PtbCommand::TransferIota { recipient, amount } => {
                    let addr = Address::from_hex(recipient)?;
                    let nanos = parse_iota_amount(amount)?;
                    builder.send_iota(addr, nanos);
                }
                crate::app::PtbCommand::TransferObjects {
                    recipient,
                    object_ids,
                } => {
                    let addr = Address::from_hex(recipient)?;
                    let ids: Result<Vec<iota_sdk::types::ObjectId>, _> = object_ids
                        .iter()
                        .map(|id| id.parse::<iota_sdk::types::ObjectId>())
                        .collect();
                    builder.transfer_objects(addr, ids?);
                }
                crate::app::PtbCommand::MoveCall {
                    package,
                    module,
                    function,
                    type_args,
                    args,
                } => {
                    let pkg_addr = Address::from_hex(package)?;
                    let call = builder.move_call(pkg_addr, module.as_str(), function.as_str());
                    if !type_args.is_empty() {
                        let tags: Vec<iota_sdk::types::TypeTag> = type_args
                            .iter()
                            .map(|s| s.parse::<iota_sdk::types::TypeTag>())
                            .collect::<Result<Vec<_>, _>>()?;
                        call.type_tags(tags);
                    }
                    let _ = args;
                }
                crate::app::PtbCommand::SplitCoins { coin, amounts } => {
                    let coin_id: iota_sdk::types::ObjectId = coin.parse()?;
                    let parsed: Result<Vec<u64>, _> =
                        amounts.iter().map(|a| parse_iota_amount(a)).collect();
                    builder.split_coins(coin_id, parsed?);
                }
                crate::app::PtbCommand::MergeCoins { primary, sources } => {
                    let primary_id: iota_sdk::types::ObjectId = primary.parse()?;
                    let source_ids: Result<Vec<iota_sdk::types::ObjectId>, _> = sources
                        .iter()
                        .map(|id| id.parse::<iota_sdk::types::ObjectId>())
                        .collect();
                    builder.merge_coins(primary_id, source_ids?);
                }
                crate::app::PtbCommand::Stake { amount, validator } => {
                    let nanos = parse_iota_amount(amount)?;
                    let validator_addr = Address::from_hex(validator)?;
                    builder.stake(nanos, validator_addr);
                }
                crate::app::PtbCommand::Unstake { staked_iota_id } => {
                    let obj_id: iota_sdk::types::ObjectId = staked_iota_id.parse()?;
                    builder.unstake(obj_id);
                }
            }
        }

        let result = builder.dry_run(false).await;
        let info = match result {
            Ok(dry_run) => {
                let estimated_gas = dry_run.effects.as_ref().and_then(|e| match e {
                    iota_sdk::types::TransactionEffects::V1(v1) => {
                        let gas = &v1.gas_used;
                        let cost = gas.computation_cost + gas.storage_cost;
                        let rebate = gas.storage_rebate.min(gas.storage_cost);
                        Some(cost - rebate)
                    }
                    _ => None,
                });
                let error = dry_run.error.clone();
                let status = if error.is_some() {
                    "Failed".to_string()
                } else {
                    "Success".to_string()
                };
                crate::app::DryRunInfo {
                    status,
                    estimated_gas,
                    error,
                }
            }
            Err(e) => crate::app::DryRunInfo {
                status: "Error".to_string(),
                estimated_gas: None,
                error: Some(e.to_string()),
            },
        };

        self.event_tx.send(WalletEvent::DryRunResult(info)).await?;
        Ok(())
    }

    async fn handle_faucet(
        &self,
        addr: Address,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let faucet = self
            .faucet
            .as_ref()
            .ok_or("Faucet not available on this network")?;
        let receipt = faucet.request_and_wait(addr).await?;
        let msg = match receipt {
            Some(_r) => "Faucet: received coins".to_string(),
            None => "Faucet request sent (no receipt)".into(),
        };
        self.event_tx
            .send(WalletEvent::FaucetRequested(msg))
            .await?;
        Ok(())
    }

    async fn handle_iota_name_lookup(
        &self,
        name: &str,
        label: &str,
        notes: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;
        // Auto-append .iota if the name doesn't contain a TLD
        let qualified = if name.ends_with(".iota") {
            name.to_string()
        } else {
            format!("{name}.iota")
        };
        let result = client.iota_names_lookup(&qualified).await?;
        let address = result.map(|a| a.to_string());
        self.event_tx
            .send(WalletEvent::IotaNameResolved {
                name: name.to_string(),
                label: label.to_string(),
                notes: notes.to_string(),
                address,
            })
            .await?;
        Ok(())
    }

    // ── Explorer handlers ──────────────────────────────────────────

    async fn handle_network_overview(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let chain_id = client.chain_id().await.unwrap_or_else(|_| "?".into());

        let epoch_str = match client.epoch(None).await {
            Ok(Some(e)) => e.epoch_id.to_string(),
            _ => "?".into(),
        };

        let gas_price = match client.reference_gas_price(None).await {
            Ok(Some(g)) => format!("{} NANOS", g),
            _ => "?".into(),
        };

        let latest_cp = match client.latest_checkpoint_sequence_number().await {
            Ok(Some(seq)) => seq.to_string(),
            _ => "?".into(),
        };

        let total_txs = match client.total_transaction_blocks().await {
            Ok(Some(n)) => n.to_string(),
            _ => "?".into(),
        };

        self.event_tx
            .send(WalletEvent::NetworkOverview {
                chain_id,
                epoch: epoch_str,
                gas_price,
                latest_checkpoint: latest_cp,
                total_transactions: total_txs,
            })
            .await?;
        Ok(())
    }

    async fn handle_checkpoints(
        &self,
        cursor: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let page = client
            .checkpoints(PaginationFilter {
                direction: Direction::Backward,
                cursor,
                limit: None,
            })
            .await?;

        let next_cursor = page.page_info().start_cursor.clone();
        let has_next = page.page_info().has_previous_page;

        let checkpoints: Vec<crate::app::CheckpointDisplay> = page
            .data()
            .iter()
            .map(|cp| {
                let ts = format_timestamp_ms(cp.timestamp_ms);
                crate::app::CheckpointDisplay {
                    sequence: cp.sequence_number,
                    digest: cp.content_digest.to_string(),
                    timestamp: ts,
                    tx_count: cp.network_total_transactions,
                }
            })
            .collect();

        self.event_tx
            .send(WalletEvent::Checkpoints {
                checkpoints,
                cursor: next_cursor,
                has_next,
            })
            .await?;
        Ok(())
    }

    async fn handle_validators(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let page = client
            .active_validators(None, PaginationFilter::default())
            .await?;

        let validators: Vec<crate::app::ValidatorDisplay> = page
            .data()
            .iter()
            .map(|v| {
                let name = v.name.clone().unwrap_or_else(|| "Unknown".into());
                let address = v.address.address.to_string();
                let stake = v
                    .voting_power
                    .map(|p| format!("{}%", p as f64 / 100.0))
                    .unwrap_or_else(|| "?".into());
                crate::app::ValidatorDisplay {
                    name,
                    address,
                    stake,
                }
            })
            .collect();

        self.event_tx
            .send(WalletEvent::Validators(validators))
            .await?;
        Ok(())
    }

    async fn handle_lookup(
        &self,
        query: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::app::{LookupAction, LookupField, LookupResult, LookupSection};
        let client = self.client.as_ref().ok_or("Not connected")?;

        let hex_query = if query.starts_with("0x") {
            query.to_string()
        } else {
            format!("0x{}", query)
        };

        // Try as object first
        if let Ok(addr) = iota_sdk::types::Address::from_hex(&hex_query) {
            let obj_id: iota_sdk::types::ObjectId = addr.into();
            if let Ok(Some(obj)) = client.object(obj_id, None).await {
                let type_name = match obj.object_type() {
                    ObjectType::Struct(s) => prettify_struct(&s),
                    ObjectType::Package => "Package".into(),
                };
                let type_raw = match obj.object_type() {
                    ObjectType::Struct(s) => s.to_string(),
                    ObjectType::Package => "Package".into(),
                };
                let owner_str = format_owner(&obj.owner);

                let info_fields = vec![
                    LookupField {
                        key: "Object ID".into(),
                        value: obj.object_id().to_string(),
                        action: None,
                    },
                    LookupField {
                        key: "Version".into(),
                        value: format!("v{}", obj.version()),
                        action: None,
                    },
                    LookupField {
                        key: "Type".into(),
                        value: type_name,
                        action: Some(LookupAction::TypeSearch(type_raw)),
                    },
                    LookupField {
                        key: "Owner".into(),
                        value: owner_str.clone(),
                        action: owner_action(&obj.owner),
                    },
                    LookupField {
                        key: "Previous Tx".into(),
                        value: obj.previous_transaction.to_string(),
                        action: Some(LookupAction::Explore(obj.previous_transaction.to_string())),
                    },
                ];

                let mut sections = vec![LookupSection {
                    title: "Object".into(),
                    fields: info_fields,
                }];

                // Fetch move object content (JSON fields) into a separate section
                if let Ok(Some(json)) = client.move_object_contents(obj_id, None).await
                    && let Some(map) = json.as_object()
                {
                    let field_entries: Vec<LookupField> = map
                        .iter()
                        .map(|(k, v)| {
                            let val_str = format_json_value(v);
                            let action = guess_action_from_value(&val_str);
                            LookupField {
                                key: k.clone(),
                                value: val_str,
                                action,
                            }
                        })
                        .collect();
                    if !field_entries.is_empty() {
                        sections.push(LookupSection {
                            title: format!("Fields ({})", field_entries.len()),
                            fields: field_entries,
                        });
                    }
                }

                // Fetch dynamic fields
                let df_page = client
                    .dynamic_fields(addr, PaginationFilter::default())
                    .await;
                if let Ok(df_page) = df_page {
                    let dfs = df_page.data();
                    if !dfs.is_empty() {
                        let mut df_fields: Vec<LookupField> = Vec::new();
                        for df in dfs {
                            let name_str = df
                                .name
                                .json
                                .as_ref()
                                .map(format_json_value)
                                .unwrap_or_else(|| format!("{}", df.name.type_));
                            let val_str = df
                                .value_as_json
                                .as_ref()
                                .map(format_json_value)
                                .unwrap_or_else(|| "?".into());
                            let action = guess_action_from_value(&val_str);
                            df_fields.push(LookupField {
                                key: name_str,
                                value: val_str,
                                action,
                            });
                        }
                        sections.push(LookupSection {
                            title: format!("Dynamic Fields ({})", df_fields.len()),
                            fields: df_fields,
                        });
                    }
                }

                self.event_tx
                    .send(WalletEvent::ExplorerLookupResult(LookupResult::Object {
                        sections,
                    }))
                    .await?;
                return Ok(());
            }
        }

        // Try as transaction digest
        if let Ok(digest) = hex_query
            .parse::<iota_sdk::types::Digest>()
            .or_else(|_| query.parse::<iota_sdk::types::Digest>())
            && let Ok(Some(td)) = client.transaction_data_effects(digest).await
        {
            let sections = match &td.effects {
                iota_sdk::types::TransactionEffects::V1(v1) => build_tx_sections_v1(v1, &td.tx),
                _ => vec![LookupSection {
                    title: "Transaction".into(),
                    fields: vec![LookupField {
                        key: "Note".into(),
                        value: "Unsupported transaction effects version".into(),
                        action: None,
                    }],
                }],
            };

            self.event_tx
                .send(WalletEvent::ExplorerLookupResult(
                    LookupResult::Transaction { sections },
                ))
                .await?;
            return Ok(());
        }

        // Try as address (look up owned objects + balance + transactions)
        if let Ok(addr) = iota_sdk::types::Address::from_hex(&hex_query) {
            let balance = client.balance(addr, None).await.unwrap_or(None);

            let obj_page = client
                .objects(
                    ObjectFilter {
                        owner: Some(addr),
                        type_: None,
                        object_ids: None,
                    },
                    PaginationFilter {
                        direction: Direction::Backward,
                        cursor: None,
                        limit: Some(20),
                    },
                )
                .await?;

            let has_data = !obj_page.data().is_empty() || balance.is_some();

            if has_data {
                let (sections, obj_cursor, obj_has_next, tx_cursor, tx_has_next) =
                    Self::build_address_sections(client, &hex_query, addr, balance, obj_page, None)
                        .await;

                self.event_tx
                    .send(WalletEvent::AddressLookupPage {
                        result: LookupResult::Address { sections },
                        obj_cursor,
                        obj_has_next,
                        tx_cursor,
                        tx_has_next,
                    })
                    .await?;
                return Ok(());
            }
        }

        self.event_tx
            .send(WalletEvent::ExplorerLookupResult(LookupResult::NotFound(
                format!("Nothing found for '{}'", query),
            )))
            .await?;
        Ok(())
    }

    async fn handle_search_objects_by_type(
        &self,
        type_filter: &str,
        cursor: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let filter = ObjectFilter {
            type_: Some(type_filter.to_string()),
            owner: None,
            object_ids: None,
        };

        let pagination = PaginationFilter {
            cursor,
            ..PaginationFilter::default()
        };

        let page = client.objects(filter, pagination).await?;
        let page_info = page.page_info().clone();

        let objects: Vec<crate::app::ObjectDisplay> = page
            .data()
            .iter()
            .map(|obj| {
                let type_name = match obj.object_type() {
                    ObjectType::Struct(s) => prettify_struct(&s),
                    ObjectType::Package => "Package".into(),
                };
                crate::app::ObjectDisplay {
                    object_id: obj.object_id().to_string(),
                    type_name,
                    version: format!("v{}", obj.version()),
                    digest: obj.previous_transaction.to_string(),
                    owner: format!("{:?}", obj.owner),
                    owner_alias: String::new(),
                }
            })
            .collect();

        self.event_tx
            .send(WalletEvent::ObjectSearchResults {
                objects,
                has_next_page: page_info.has_next_page,
                end_cursor: page_info.end_cursor,
            })
            .await?;
        Ok(())
    }

    fn load_keys(&mut self) {
        if let Ok(data) = std::fs::read_to_string(&self.keystore_path)
            && let Ok(keys) = serde_json::from_str::<Vec<StoredKey>>(&data)
        {
            for key in &keys {
                if let Ok(kp) = SimpleKeypair::from_bytes(&key.private_key_bytes) {
                    self.keypairs.push(kp);
                }
            }
            self.keys = keys;
        }
    }

    fn save_keys(&self) {
        if let Some(parent) = self.keystore_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self.keys) {
            let _ = std::fs::write(&self.keystore_path, data);
        }
    }

    pub fn stored_keys(&self) -> &[StoredKey] {
        &self.keys
    }

    async fn build_address_sections(
        client: &Client,
        hex_query: &str,
        addr: iota_sdk::types::Address,
        balance: Option<u64>,
        obj_page: iota_sdk::graphql_client::Page<iota_sdk::types::Object>,
        tx_cursor: Option<String>,
    ) -> (
        Vec<crate::app::LookupSection>,
        Option<String>,
        bool,
        Option<String>,
        bool,
    ) {
        use crate::app::{LookupAction, LookupField, LookupSection};

        let balance_str = balance.map(format_gas).unwrap_or_else(|| "0".into());
        let overview = vec![
            LookupField {
                key: "Address".into(),
                value: hex_query.to_string(),
                action: None,
            },
            LookupField {
                key: "IOTA Balance".into(),
                value: balance_str,
                action: None,
            },
        ];
        let mut sections = vec![LookupSection {
            title: "Address".into(),
            fields: overview,
        }];

        // Objects section
        let obj_cursor_out = obj_page.page_info().start_cursor.clone();
        let obj_has_next = obj_page.page_info().has_previous_page;
        if !obj_page.data().is_empty() {
            let obj_fields: Vec<LookupField> = obj_page
                .data()
                .iter()
                .enumerate()
                .map(|(i, obj)| {
                    let id_str = obj.object_id().to_string();
                    LookupField {
                        key: format!("{}", i),
                        value: id_str.clone(),
                        action: Some(LookupAction::Explore(id_str)),
                    }
                })
                .collect();
            sections.push(LookupSection {
                title: format!("Objects ({})", obj_fields.len()),
                fields: obj_fields,
            });
        }

        // Transactions section
        let mut tx_cursor_out: Option<String> = None;
        let mut tx_has_next = false;
        {
            use iota_sdk::graphql_client::query_types::TransactionsFilter;
            let tx_filter = TransactionsFilter {
                sign_address: Some(addr),
                ..Default::default()
            };
            if let Ok(tx_page) = client
                .transactions_effects(
                    tx_filter,
                    PaginationFilter {
                        direction: Direction::Backward,
                        cursor: tx_cursor,
                        limit: Some(20),
                    },
                )
                .await
            {
                tx_cursor_out = tx_page.page_info().start_cursor.clone();
                tx_has_next = tx_page.page_info().has_previous_page;
                if !tx_page.data().is_empty() {
                    let tx_fields: Vec<LookupField> = tx_page
                        .data()
                        .iter()
                        .map(|effects| match effects {
                            iota_sdk::types::TransactionEffects::V1(ev1) => {
                                let status = match &ev1.status {
                                    iota_sdk::types::ExecutionStatus::Success => "OK",
                                    _ => "FAIL",
                                };
                                let digest_str = ev1.transaction_digest.to_string();
                                LookupField {
                                    key: status.into(),
                                    value: digest_str.clone(),
                                    action: Some(LookupAction::Explore(digest_str)),
                                }
                            }
                            _ => LookupField {
                                key: "?".into(),
                                value: "Unsupported effects version".into(),
                                action: None,
                            },
                        })
                        .collect();
                    sections.push(LookupSection {
                        title: format!("Transactions ({})", tx_fields.len()),
                        fields: tx_fields,
                    });
                }
            }
        }

        (
            sections,
            obj_cursor_out,
            obj_has_next,
            tx_cursor_out,
            tx_has_next,
        )
    }

    async fn handle_address_page(
        &self,
        address: &str,
        obj_cursor: Option<String>,
        tx_cursor: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::app::LookupResult;
        let client = self.client.as_ref().ok_or("Not connected")?;
        let hex_addr = if address.starts_with("0x") {
            address.to_string()
        } else {
            format!("0x{}", address)
        };
        let addr = iota_sdk::types::Address::from_hex(&hex_addr)?;

        let balance = client.balance(addr, None).await.unwrap_or(None);
        let obj_page = client
            .objects(
                ObjectFilter {
                    owner: Some(addr),
                    type_: None,
                    object_ids: None,
                },
                PaginationFilter {
                    direction: Direction::Backward,
                    cursor: obj_cursor,
                    limit: Some(20),
                },
            )
            .await?;

        let (sections, obj_cursor_out, obj_has_next, tx_cursor_out, tx_has_next) =
            Self::build_address_sections(client, &hex_addr, addr, balance, obj_page, tx_cursor)
                .await;

        self.event_tx
            .send(WalletEvent::AddressLookupPage {
                result: LookupResult::Address { sections },
                obj_cursor: obj_cursor_out,
                obj_has_next,
                tx_cursor: tx_cursor_out,
                tx_has_next,
            })
            .await?;
        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn generate_keypair(
    scheme: &str,
) -> Result<(SimpleKeypair, String), Box<dyn std::error::Error + Send + Sync>> {
    let rng = rand::rngs::OsRng;
    match scheme {
        "ed25519" => {
            let sk = Ed25519PrivateKey::generate(rng);
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        "secp256k1" => {
            let sk = Secp256k1PrivateKey::generate(rng);
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        "secp256r1" => {
            let sk = Secp256r1PrivateKey::generate(rng);
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        _ => Err(format!("Unknown scheme: {}", scheme).into()),
    }
}

fn import_keypair_from_raw(
    scheme: &str,
    raw_bytes: &[u8],
) -> Result<(SimpleKeypair, String), Box<dyn std::error::Error + Send + Sync>> {
    match scheme {
        "ed25519" => {
            let sk = Ed25519PrivateKey::from_bytes(raw_bytes)
                .map_err(|e| format!("Invalid ed25519 key: {}", e))?;
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        "secp256k1" => {
            let sk = Secp256k1PrivateKey::from_bytes(raw_bytes)
                .map_err(|e| format!("Invalid secp256k1 key: {}", e))?;
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        "secp256r1" => {
            let sk = Secp256r1PrivateKey::from_bytes(raw_bytes)
                .map_err(|e| format!("Invalid secp256r1 key: {}", e))?;
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        _ => Err(format!("Unknown scheme: {}", scheme).into()),
    }
}

fn parse_iota_amount(s: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    // Try as decimal IOTA first (e.g. "1.5" -> 1_500_000_000 nanos)
    if let Ok(f) = s.parse::<f64>() {
        return Ok((f * 1_000_000_000.0) as u64);
    }
    // Try as raw integer nanos
    if let Ok(n) = s.parse::<u64>() {
        return Ok(n);
    }
    Err(format!("Invalid amount: {}", s).into())
}

fn format_timestamp_ms(ms: u64) -> String {
    // Simple UTC timestamp formatting without chrono dependency
    let secs = ms / 1000;
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;

    // Convert days since epoch to Y-M-D
    let (year, month, day) = days_to_ymd(days);
    format!("{year}-{month:02}-{day:02} {hours:02}:{minutes:02}")
}

fn days_to_ymd(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn format_gas(nanos: u64) -> String {
    if nanos >= 1_000_000_000 {
        format!("{:.4} IOTA", nanos as f64 / 1_000_000_000.0)
    } else if nanos >= 1_000_000 {
        format!("{:.2}M", nanos as f64 / 1_000_000.0)
    } else if nanos >= 1_000 {
        format!("{:.1}K", nanos as f64 / 1_000.0)
    } else {
        format!("{}", nanos)
    }
}

fn prettify_type(tag: &TypeTag) -> String {
    match tag {
        TypeTag::Vector(type_tag) => {
            format!("vector<{}>", prettify_type(type_tag))
        }
        TypeTag::Struct(struct_tag) => prettify_struct(struct_tag),
        _ => tag.to_string(),
    }
}

fn prettify_struct(tag: &StructTag) -> String {
    const SYSTEM_ADDRESSES: &[Address] = &[Address::STD, Address::FRAMEWORK, Address::SYSTEM];
    const SYSTEM_ADDRESS_NAMES: &[&str] = &["std", "framework", "system"];
    if let Some(pos) = SYSTEM_ADDRESSES.iter().position(|a| a == &tag.address()) {
        let mut s = String::new();
        write!(
            &mut s,
            "{}::{}::{}",
            SYSTEM_ADDRESS_NAMES[pos],
            tag.module(),
            tag.name()
        )
        .unwrap();
        if !tag.type_params().is_empty() {
            let params: Vec<String> = tag.type_params().iter().map(prettify_type).collect();
            write!(&mut s, "<{}>", params.join(", ")).unwrap();
        }
        s
    } else {
        tag.to_string()
    }
}

fn extract_symbol(coin_type: &str) -> String {
    coin_type
        .rsplit("::")
        .next()
        .unwrap_or(coin_type)
        .to_string()
}

/// Auto-detect encoding format and decode a private key.
/// Supports: bech32 (iotaprivkey1...), base64 (flagged bytes), hex (raw or 0x-prefixed).
/// Returns (keypair, scheme_name).
fn decode_private_key(
    fallback_scheme: &str,
    input: &str,
) -> Result<(SimpleKeypair, String), Box<dyn std::error::Error + Send + Sync>> {
    use iota_sdk::crypto::{ToFromBech32, ToFromFlaggedBytes};

    let input = input.trim();

    // 1. Bech32: starts with "iotaprivkey1"
    if input.starts_with("iotaprivkey1") {
        let kp =
            SimpleKeypair::from_bech32(input).map_err(|e| format!("Invalid bech32 key: {}", e))?;
        let scheme = scheme_name_from_keypair(&kp);
        return Ok((kp, scheme));
    }

    // 2. Hex: starts with "0x" or is all hex chars
    let stripped = input.strip_prefix("0x").unwrap_or(input);
    if !stripped.is_empty() && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        let raw = hex::decode(stripped).map_err(|e| format!("Invalid hex: {}", e))?;

        // If 33 bytes, treat as flagged (flag + 32-byte key)
        if raw.len() == 33 {
            let kp = SimpleKeypair::from_flagged_bytes(&raw)
                .map_err(|e| format!("Invalid flagged key: {}", e))?;
            let scheme = scheme_name_from_keypair(&kp);
            return Ok((kp, scheme));
        }

        // Otherwise treat as raw key bytes using the fallback scheme
        let (kp, _addr) = import_keypair_from_raw(fallback_scheme, &raw)?;
        return Ok((kp, fallback_scheme.to_string()));
    }

    // 3. Base64: try decoding as base64 (flagged bytes format)
    if let Ok(decoded) = base64ct::Base64::decode_vec(input) {
        if decoded.len() == 33 {
            let kp = SimpleKeypair::from_flagged_bytes(&decoded)
                .map_err(|e| format!("Invalid base64 key: {}", e))?;
            let scheme = scheme_name_from_keypair(&kp);
            return Ok((kp, scheme));
        }
        // Raw 32-byte key in base64
        if decoded.len() == 32 {
            let (kp, _addr) = import_keypair_from_raw(fallback_scheme, &decoded)?;
            return Ok((kp, fallback_scheme.to_string()));
        }
    }

    Err("Could not decode key. Supported formats: bech32 (iotaprivkey1...), hex, base64".into())
}

fn scheme_name_from_keypair(kp: &SimpleKeypair) -> String {
    use iota_sdk::types::SignatureScheme;
    match kp.scheme() {
        SignatureScheme::Ed25519 => "ed25519".into(),
        SignatureScheme::Secp256k1 => "secp256k1".into(),
        SignatureScheme::Secp256r1 => "secp256r1".into(),
        other => format!("{:?}", other),
    }
}

fn keypair_address(kp: &SimpleKeypair) -> String {
    use iota_sdk::types::MultisigMemberPublicKey;
    match kp.public_key() {
        MultisigMemberPublicKey::Ed25519(pk) => pk.derive_address().to_string(),
        MultisigMemberPublicKey::Secp256k1(pk) => pk.derive_address().to_string(),
        MultisigMemberPublicKey::Secp256r1(pk) => pk.derive_address().to_string(),
        _ => "unknown".to_string(),
    }
}

fn build_tx_sections_v1(
    v1: &iota_sdk::types::TransactionEffectsV1,
    signed_tx: &iota_sdk::types::SignedTransaction,
) -> Vec<crate::app::LookupSection> {
    use crate::app::{LookupAction, LookupField, LookupSection};

    let status = match &v1.status {
        iota_sdk::types::ExecutionStatus::Success => "Success".to_string(),
        iota_sdk::types::ExecutionStatus::Failure { error, .. } => {
            format!("Failed: {:?}", error)
        }
        _ => "Unknown (unsupported status variant)".to_string(),
    };
    let gas = &v1.gas_used;

    // Overview section
    let mut overview = vec![
        LookupField {
            key: "Digest".into(),
            value: v1.transaction_digest.to_string(),
            action: None,
        },
        LookupField {
            key: "Status".into(),
            value: status,
            action: None,
        },
        LookupField {
            key: "Epoch".into(),
            value: format!("{}", v1.epoch),
            action: None,
        },
    ];

    // Add sender from transaction data
    match &signed_tx.transaction {
        iota_sdk::types::Transaction::V1(tx_v1) => {
            overview.push(LookupField {
                key: "Sender".into(),
                value: tx_v1.sender.to_string(),
                action: Some(LookupAction::Explore(tx_v1.sender.to_string())),
            });
        }
        _ => {
            overview.push(LookupField {
                key: "Transaction Data".into(),
                value: "Unsupported transaction version".into(),
                action: None,
            });
        }
    }

    let mut sections = vec![LookupSection {
        title: "Transaction".into(),
        fields: overview,
    }];

    // Gas section (display in NANOS)
    let total_gas =
        gas.computation_cost + gas.storage_cost - gas.storage_rebate.min(gas.storage_cost);
    sections.push(LookupSection {
        title: "Gas".into(),
        fields: vec![
            LookupField {
                key: "Total".into(),
                value: format!("{} NANOS", total_gas),
                action: None,
            },
            LookupField {
                key: "Computation".into(),
                value: format!("{} NANOS", gas.computation_cost),
                action: None,
            },
            LookupField {
                key: "Storage".into(),
                value: format!("{} NANOS", gas.storage_cost),
                action: None,
            },
            LookupField {
                key: "Rebate".into(),
                value: format!("{} NANOS", gas.storage_rebate),
                action: None,
            },
        ],
    });

    // Signatures section
    let sig_fields: Vec<LookupField> = signed_tx
        .signatures
        .iter()
        .enumerate()
        .map(|(i, sig)| {
            let desc = match sig {
                iota_sdk::types::UserSignature::Simple(s) => {
                    format!("{:?}", s.scheme())
                }
                _ => "Unsupported signature type".to_string(),
            };
            LookupField {
                key: format!("Sig {}", i),
                value: desc,
                action: None,
            }
        })
        .collect();
    if !sig_fields.is_empty() {
        sections.push(LookupSection {
            title: "Signatures".into(),
            fields: sig_fields,
        });
    }

    // Inputs & Commands from transaction data
    match &signed_tx.transaction {
        iota_sdk::types::Transaction::V1(tx_v1) => {
            if let iota_sdk::types::TransactionKind::ProgrammableTransaction(ref ptx) = tx_v1.kind {
                // Inputs
                let input_fields: Vec<LookupField> = ptx
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(i, input)| {
                        let (desc, action) = format_input(input);
                        LookupField {
                            key: format!("Input {}", i),
                            value: desc,
                            action,
                        }
                    })
                    .collect();
                if !input_fields.is_empty() {
                    sections.push(LookupSection {
                        title: "Inputs".into(),
                        fields: input_fields,
                    });
                }

                // Commands
                let cmd_fields: Vec<LookupField> = ptx
                    .commands
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| {
                        let (desc, action) = format_command(cmd);
                        LookupField {
                            key: format!("Cmd {}", i),
                            value: desc,
                            action,
                        }
                    })
                    .collect();
                if !cmd_fields.is_empty() {
                    sections.push(LookupSection {
                        title: "Commands".into(),
                        fields: cmd_fields,
                    });
                }
            }
        }
        _ => {
            sections.push(LookupSection {
                title: "Inputs & Commands".into(),
                fields: vec![LookupField {
                    key: "Note".into(),
                    value: "Unsupported transaction version — cannot display details".into(),
                    action: None,
                }],
            });
        }
    }

    // Changed objects from effects
    let changed_fields: Vec<LookupField> = v1
        .changed_objects
        .iter()
        .map(|co| {
            let id_str = co.object_id.to_string();
            let op = format!("{:?}", co.id_operation);
            LookupField {
                key: op,
                value: id_str.clone(),
                action: Some(LookupAction::Explore(id_str)),
            }
        })
        .collect();
    if !changed_fields.is_empty() {
        sections.push(LookupSection {
            title: format!("Changed Objects ({})", changed_fields.len()),
            fields: changed_fields,
        });
    }

    sections
}

fn format_owner(owner: &iota_sdk::types::Owner) -> String {
    match owner {
        iota_sdk::types::Owner::Address(a) => a.to_string(),
        iota_sdk::types::Owner::Object(id) => format!("Object({})", id),
        iota_sdk::types::Owner::Shared(version) => {
            format!("Shared(v{})", version)
        }
        iota_sdk::types::Owner::Immutable => "Immutable".into(),
        _ => "Unsupported owner type".to_string(),
    }
}

fn owner_action(owner: &iota_sdk::types::Owner) -> Option<crate::app::LookupAction> {
    match owner {
        iota_sdk::types::Owner::Address(a) => {
            Some(crate::app::LookupAction::Explore(a.to_string()))
        }
        iota_sdk::types::Owner::Object(id) => {
            Some(crate::app::LookupAction::Explore(id.to_string()))
        }
        _ => None,
    }
}

fn format_json_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".into(),
        serde_json::Value::Array(arr) => {
            if arr.len() <= 3 {
                format!(
                    "[{}]",
                    arr.iter()
                        .map(format_json_value)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                format!("[{} items]", arr.len())
            }
        }
        serde_json::Value::Object(map) => {
            let pairs: Vec<String> = map
                .iter()
                .take(3)
                .map(|(k, v)| format!("{}: {}", k, format_json_value(v)))
                .collect();
            if map.len() > 3 {
                format!("{{ {}, ... }}", pairs.join(", "))
            } else {
                format!("{{ {} }}", pairs.join(", "))
            }
        }
    }
}

/// Guess whether a string value looks like an address/object ID or digest.
fn guess_action_from_value(val: &str) -> Option<crate::app::LookupAction> {
    let trimmed = val.trim();
    if trimmed.starts_with("0x") && trimmed.len() >= 40 {
        Some(crate::app::LookupAction::Explore(trimmed.to_string()))
    } else {
        None
    }
}

fn format_input(input: &iota_sdk::types::Input) -> (String, Option<crate::app::LookupAction>) {
    match input {
        iota_sdk::types::Input::Pure { value } => {
            let hex = if value.len() <= 32 {
                hex::encode(value)
            } else {
                format!("{}... ({} bytes)", hex::encode(&value[..16]), value.len())
            };
            (format!("Pure({})", hex), None)
        }
        iota_sdk::types::Input::ImmutableOrOwned(obj_ref) => {
            let id = obj_ref.object_id().to_string();
            (
                format!("Object({})", &id),
                Some(crate::app::LookupAction::Explore(id)),
            )
        }
        iota_sdk::types::Input::Shared { object_id, .. } => {
            let id = object_id.to_string();
            (
                format!("Shared({})", &id),
                Some(crate::app::LookupAction::Explore(id)),
            )
        }
        iota_sdk::types::Input::Receiving(obj_ref) => {
            let id = obj_ref.object_id().to_string();
            (
                format!("Receiving({})", &id),
                Some(crate::app::LookupAction::Explore(id)),
            )
        }
        _ => ("Unsupported input type".to_string(), None),
    }
}

fn format_argument(arg: &iota_sdk::types::Argument) -> String {
    match arg {
        iota_sdk::types::Argument::Gas => "Gas".into(),
        iota_sdk::types::Argument::Input(i) => format!("Input({})", i),
        iota_sdk::types::Argument::Result(i) => format!("Result({})", i),
        iota_sdk::types::Argument::NestedResult(i, j) => format!("Result({}).{}", i, j),
        _ => "?".into(),
    }
}

fn format_arguments(args: &[iota_sdk::types::Argument]) -> String {
    args.iter()
        .map(format_argument)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_command(cmd: &iota_sdk::types::Command) -> (String, Option<crate::app::LookupAction>) {
    match cmd {
        iota_sdk::types::Command::MoveCall(mc) => {
            let pkg = mc.package.to_string();
            let mut desc = format!(
                "MoveCall {}::{}::{}",
                &pkg[..10.min(pkg.len())],
                mc.module,
                mc.function
            );
            if !mc.type_arguments.is_empty() {
                let types: Vec<String> = mc.type_arguments.iter().map(prettify_type).collect();
                write!(desc, "<{}>", types.join(", ")).ok();
            }
            if !mc.arguments.is_empty() {
                write!(desc, "({})", format_arguments(&mc.arguments)).ok();
            }
            (desc, Some(crate::app::LookupAction::Explore(pkg)))
        }
        iota_sdk::types::Command::TransferObjects(t) => {
            let desc = format!(
                "TransferObjects [{}] -> {}",
                format_arguments(&t.objects),
                format_argument(&t.address)
            );
            (desc, None)
        }
        iota_sdk::types::Command::SplitCoins(s) => {
            let desc = format!(
                "SplitCoins {} -> [{}]",
                format_argument(&s.coin),
                format_arguments(&s.amounts)
            );
            (desc, None)
        }
        iota_sdk::types::Command::MergeCoins(m) => {
            let desc = format!(
                "MergeCoins {} <- [{}]",
                format_argument(&m.coin),
                format_arguments(&m.coins_to_merge)
            );
            (desc, None)
        }
        iota_sdk::types::Command::Publish(p) => {
            let desc = format!(
                "Publish ({} modules, {} deps)",
                p.modules.len(),
                p.dependencies.len()
            );
            (desc, None)
        }
        iota_sdk::types::Command::MakeMoveVector(v) => {
            let type_str = v
                .type_
                .as_ref()
                .map(prettify_type)
                .unwrap_or_else(|| "?".into());
            let desc = format!(
                "MakeMoveVector<{}> [{}]",
                type_str,
                format_arguments(&v.elements)
            );
            (desc, None)
        }
        iota_sdk::types::Command::Upgrade(u) => {
            let pkg = u.package.to_string();
            let desc = format!(
                "Upgrade {} ({} modules, {} deps)",
                &pkg[..10.min(pkg.len())],
                u.modules.len(),
                u.dependencies.len()
            );
            (desc, Some(crate::app::LookupAction::Explore(pkg)))
        }
        _ => ("Unsupported command type".to_string(), None),
    }
}

fn log_error(msg: &str) {
    use std::io::Write;
    let path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("iota-wallet-tui")
        .join("error.log");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(file, "[{}] {}", secs, msg);
    }
}
