//! Core handler methods for `WalletBackend`.

use iota_sdk::crypto::simple::SimpleKeypair;
use iota_sdk::graphql_client::{Direction, PaginationFilter, query_types::ObjectFilter};
use iota_sdk::transaction_builder::TransactionBuilder;
use iota_sdk::types::{Address, ObjectType};

use super::helpers::{
    decode_private_key, extract_symbol, format_gas, format_timestamp_ms, generate_keypair,
    keypair_address, parse_iota_amount, prettify_struct, prettify_type,
};
use super::{
    BalanceInfo, CoinInfo, Network, ObjectInfo, StoredKey, WalletBackend, WalletEvent, save_network,
};

impl WalletBackend {
    pub(super) async fn handle_connect(
        &mut self,
        network: Network,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use iota_sdk::graphql_client::{Client, faucet::FaucetClient};
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

    pub(super) async fn handle_balances(
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

    pub(super) async fn handle_coins(
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

    pub(super) async fn handle_objects(
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

    pub(super) async fn handle_transactions(
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

    pub(super) fn handle_generate_key(
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

    pub(super) fn handle_import_key(
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

    pub(super) fn handle_rename_key(
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

    pub(super) fn handle_delete_key(
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

    pub(super) fn handle_set_active_key(
        &mut self,
        idx: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (i, key) in self.keys.iter_mut().enumerate() {
            key.is_active = i == idx;
        }
        self.save_keys();
        Ok(())
    }

    pub(super) async fn handle_execute_ptb(
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
        let _effects = builder.execute(keypair, None).await?;

        self.event_tx.send(WalletEvent::TxSubmitted).await?;
        Ok(())
    }

    pub(super) async fn handle_dry_run(
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

    pub(super) async fn handle_faucet(
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

    pub(super) async fn handle_iota_name_lookup(
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

    pub(super) async fn handle_network_overview(
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

    pub(super) async fn handle_checkpoints(
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

    pub(super) async fn handle_validators(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    pub(super) async fn handle_search_objects_by_type(
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

    pub(super) fn load_keys(&mut self) {
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

    pub(super) fn save_keys(&self) {
        if let Some(parent) = self.keystore_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self.keys) {
            let _ = std::fs::write(&self.keystore_path, data);
        }
    }
}
