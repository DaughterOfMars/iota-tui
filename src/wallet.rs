use std::path::PathBuf;

use base64ct::Encoding;
use iota_sdk::crypto::{
    ToFromBytes, ed25519::Ed25519PrivateKey, secp256k1::Secp256k1PrivateKey,
    secp256r1::Secp256r1PrivateKey, simple::SimpleKeypair,
};
use iota_sdk::graphql_client::{
    Client, PaginationFilter, faucet::FaucetClient, query_types::ObjectFilter,
};
use iota_sdk::transaction_builder::TransactionBuilder;
use iota_sdk::types::{Address, ObjectType};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// ── Types for the TUI ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CoinInfo {
    pub coin_type: String,
    pub symbol: String,
    pub balance: u128,
    pub object_id: String,
}

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub object_id: String,
    pub type_name: String,
    pub version: Option<u64>,
    pub digest: String,
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct BalanceInfo {
    pub coin_type: String,
    pub total_balance: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKey {
    pub alias: String,
    pub scheme: String,
    pub private_key_bytes: Vec<u8>,
    pub address: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
}

// ── Commands and Responses ─────────────────────────────────────────

#[derive(Debug)]
pub enum WalletCmd {
    Connect(Network),
    RefreshBalances(Address),
    RefreshCoins(Address),
    RefreshObjects(Address),
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
    DeleteKey(usize),
    RenameKey {
        idx: usize,
        new_alias: String,
    },
    RequestFaucet(Address),
}

#[derive(Debug)]
pub enum WalletEvent {
    Connected(String),
    Balances(Vec<BalanceInfo>),
    Coins(Vec<CoinInfo>),
    Objects(Vec<ObjectInfo>),
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
    TxSubmitted {
        digest: String,
    },
    FaucetRequested(String),
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
                WalletCmd::RefreshCoins(addr) => self.handle_coins(addr).await,
                WalletCmd::RefreshObjects(addr) => self.handle_objects(addr).await,
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
                WalletCmd::DeleteKey(idx) => self.handle_delete_key(idx),
                WalletCmd::RenameKey { idx, new_alias } => self.handle_rename_key(idx, &new_alias),
                WalletCmd::RequestFaucet(addr) => self.handle_faucet(addr).await,
            };

            if let Err(e) = result {
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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref().ok_or("Not connected")?;

        let page = client
            .coins(addr, None, PaginationFilter::default())
            .await?;
        let coins: Vec<CoinInfo> = page
            .data()
            .iter()
            .map(|c| CoinInfo {
                coin_type: c.coin_type().to_string(),
                symbol: extract_symbol(&c.coin_type().to_string()),
                balance: c.balance() as u128,
                object_id: c.id().to_string(),
            })
            .collect();

        self.event_tx.send(WalletEvent::Coins(coins)).await?;
        Ok(())
    }

    async fn handle_objects(
        &self,
        addr: Address,
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
                    ObjectType::Struct(s) => s.to_string(),
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

        self.event_tx.send(WalletEvent::Objects(objects)).await?;
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
                    let nanos = parse_iota_amount(&amount)?;
                    let validator_addr = Address::from_hex(&validator)?;
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

        let digest = effects.as_v1().transaction_digest.to_string();
        self.event_tx
            .send(WalletEvent::TxSubmitted { digest })
            .await?;
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

    fn load_keys(&mut self) {
        if let Ok(data) = std::fs::read_to_string(&self.keystore_path) {
            if let Ok(keys) = serde_json::from_str::<Vec<StoredKey>>(&data) {
                for key in &keys {
                    if let Ok(kp) = SimpleKeypair::from_bytes(&key.private_key_bytes) {
                        self.keypairs.push(kp);
                    }
                }
                self.keys = keys;
            }
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
}

// ── Helpers ────────────────────────────────────────────────────────

fn generate_keypair(
    scheme: &str,
) -> Result<(SimpleKeypair, String), Box<dyn std::error::Error + Send + Sync>> {
    let mut rng = rand::rngs::OsRng;
    match scheme {
        "ed25519" => {
            let sk = Ed25519PrivateKey::generate(&mut rng);
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        "secp256k1" => {
            let sk = Secp256k1PrivateKey::generate(&mut rng);
            let addr = sk.public_key().derive_address();
            let kp = SimpleKeypair::from(sk);
            Ok((kp, addr.to_string()))
        }
        "secp256r1" => {
            let sk = Secp256r1PrivateKey::generate(&mut rng);
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
