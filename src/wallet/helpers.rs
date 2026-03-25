//! Free helper functions used across the wallet module.

use std::fmt::Write;
use std::path::PathBuf;

use base64ct::Encoding;
use iota_sdk::crypto::{
    ToFromBytes, ed25519::Ed25519PrivateKey, secp256k1::Secp256k1PrivateKey,
    secp256r1::Secp256r1PrivateKey, simple::SimpleKeypair,
};
use iota_sdk::types::{Address, StructTag, TypeTag};

pub(super) fn generate_keypair(
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

pub(super) fn import_keypair_from_raw(
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

pub(super) fn parse_iota_amount(s: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
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

pub(super) fn format_timestamp_ms(ms: u64) -> String {
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

pub(super) fn format_gas(nanos: u64) -> String {
    if nanos >= 1_000_000_000 {
        format!("{:.4} IOTA", nanos as f64 / 1_000_000_000.0)
    } else {
        format!("{} NANOS", nanos)
    }
}

pub(super) fn prettify_type(tag: &TypeTag) -> String {
    match tag {
        TypeTag::Vector(type_tag) => {
            format!("vector<{}>", prettify_type(type_tag))
        }
        TypeTag::Struct(struct_tag) => prettify_struct(struct_tag),
        _ => tag.to_string(),
    }
}

pub(super) fn prettify_struct(tag: &StructTag) -> String {
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

pub(super) fn extract_symbol(coin_type: &str) -> String {
    coin_type
        .rsplit("::")
        .next()
        .unwrap_or(coin_type)
        .to_string()
}

/// Auto-detect encoding format and decode a private key.
/// Supports: bech32 (iotaprivkey1...), base64 (flagged bytes), hex (raw or 0x-prefixed).
/// Returns (keypair, scheme_name).
pub(super) fn decode_private_key(
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

pub(super) fn scheme_name_from_keypair(kp: &SimpleKeypair) -> String {
    use iota_sdk::types::SignatureScheme;
    match kp.scheme() {
        SignatureScheme::Ed25519 => "ed25519".into(),
        SignatureScheme::Secp256k1 => "secp256k1".into(),
        SignatureScheme::Secp256r1 => "secp256r1".into(),
        other => format!("{:?}", other),
    }
}

pub(super) fn keypair_address(kp: &SimpleKeypair) -> String {
    use iota_sdk::types::MultisigMemberPublicKey;
    match kp.public_key() {
        MultisigMemberPublicKey::Ed25519(pk) => pk.derive_address().to_string(),
        MultisigMemberPublicKey::Secp256k1(pk) => pk.derive_address().to_string(),
        MultisigMemberPublicKey::Secp256r1(pk) => pk.derive_address().to_string(),
        _ => "unknown".to_string(),
    }
}

pub(super) fn build_tx_sections_v1(
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

    // Derive transaction kind from the transaction data
    if let iota_sdk::types::Transaction::V1(tx_v1) = &signed_tx.transaction {
        let (kind_name, detail) = match &tx_v1.kind {
            iota_sdk::types::TransactionKind::ProgrammableTransaction(ptx) => (
                "Programmable Transaction".to_string(),
                Some(summarize_transaction(&ptx.commands)),
            ),
            iota_sdk::types::TransactionKind::Genesis(_) => ("Genesis".to_string(), None),
            iota_sdk::types::TransactionKind::ConsensusCommitPrologueV1(_) => {
                ("Consensus Commit Prologue".to_string(), None)
            }
            iota_sdk::types::TransactionKind::AuthenticatorStateUpdateV1(_) => {
                ("Authenticator State Update".to_string(), None)
            }
            iota_sdk::types::TransactionKind::RandomnessStateUpdate(_) => {
                ("Randomness State Update".to_string(), None)
            }
            iota_sdk::types::TransactionKind::EndOfEpoch(_) => ("End of Epoch".to_string(), None),
            _ => ("System".to_string(), None),
        };
        overview.push(LookupField {
            key: "Kind".into(),
            value: kind_name,
            action: None,
        });
        if let Some(d) = detail {
            overview.push(LookupField {
                key: "Summary".into(),
                value: d,
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

                // Commands — each command gets its own section for clarity
                for (i, cmd) in ptx.commands.iter().enumerate() {
                    let cmd_section = format_command_section(i, cmd);
                    sections.push(cmd_section);
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

pub(super) fn format_owner(owner: &iota_sdk::types::Owner) -> String {
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

pub(super) fn owner_action(owner: &iota_sdk::types::Owner) -> Option<crate::app::LookupAction> {
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

pub(super) fn format_json_value(val: &serde_json::Value) -> String {
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
pub(super) fn guess_action_from_value(val: &str) -> Option<crate::app::LookupAction> {
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

/// Format a PTB command as its own section with broken-out fields.
fn format_command_section(idx: usize, cmd: &iota_sdk::types::Command) -> crate::app::LookupSection {
    use crate::app::{LookupAction, LookupField, LookupSection};

    match cmd {
        iota_sdk::types::Command::MoveCall(mc) => {
            let pkg = mc.package.to_string();
            let mut fields = vec![
                LookupField {
                    key: "Package".into(),
                    value: pkg.clone(),
                    action: Some(LookupAction::Explore(pkg)),
                },
                LookupField {
                    key: "Function".into(),
                    value: format!("{}::{}", mc.module, mc.function),
                    action: None,
                },
            ];
            if !mc.type_arguments.is_empty() {
                let types: Vec<String> = mc.type_arguments.iter().map(prettify_type).collect();
                fields.push(LookupField {
                    key: "Type Args".into(),
                    value: types.join(", "),
                    action: None,
                });
            }
            if !mc.arguments.is_empty() {
                fields.push(LookupField {
                    key: "Arguments".into(),
                    value: format_arguments(&mc.arguments),
                    action: None,
                });
            }
            LookupSection {
                title: format!("Cmd {} — MoveCall", idx),
                fields,
            }
        }
        iota_sdk::types::Command::TransferObjects(t) => LookupSection {
            title: format!("Cmd {} — TransferObjects", idx),
            fields: vec![
                LookupField {
                    key: "Objects".into(),
                    value: format_arguments(&t.objects),
                    action: None,
                },
                LookupField {
                    key: "Recipient".into(),
                    value: format_argument(&t.address),
                    action: None,
                },
            ],
        },
        iota_sdk::types::Command::SplitCoins(s) => LookupSection {
            title: format!("Cmd {} — SplitCoins", idx),
            fields: vec![
                LookupField {
                    key: "Coin".into(),
                    value: format_argument(&s.coin),
                    action: None,
                },
                LookupField {
                    key: "Amounts".into(),
                    value: format_arguments(&s.amounts),
                    action: None,
                },
            ],
        },
        iota_sdk::types::Command::MergeCoins(m) => LookupSection {
            title: format!("Cmd {} — MergeCoins", idx),
            fields: vec![
                LookupField {
                    key: "Target".into(),
                    value: format_argument(&m.coin),
                    action: None,
                },
                LookupField {
                    key: "Sources".into(),
                    value: format_arguments(&m.coins_to_merge),
                    action: None,
                },
            ],
        },
        iota_sdk::types::Command::Publish(p) => LookupSection {
            title: format!("Cmd {} — Publish", idx),
            fields: vec![
                LookupField {
                    key: "Modules".into(),
                    value: format!("{}", p.modules.len()),
                    action: None,
                },
                LookupField {
                    key: "Dependencies".into(),
                    value: format!("{}", p.dependencies.len()),
                    action: None,
                },
            ],
        },
        iota_sdk::types::Command::MakeMoveVector(v) => {
            let type_str = v
                .type_
                .as_ref()
                .map(prettify_type)
                .unwrap_or_else(|| "?".into());
            LookupSection {
                title: format!("Cmd {} — MakeMoveVector", idx),
                fields: vec![
                    LookupField {
                        key: "Type".into(),
                        value: type_str,
                        action: None,
                    },
                    LookupField {
                        key: "Elements".into(),
                        value: format_arguments(&v.elements),
                        action: None,
                    },
                ],
            }
        }
        iota_sdk::types::Command::Upgrade(u) => {
            let pkg = u.package.to_string();
            LookupSection {
                title: format!("Cmd {} — Upgrade", idx),
                fields: vec![
                    LookupField {
                        key: "Package".into(),
                        value: pkg.clone(),
                        action: Some(LookupAction::Explore(pkg)),
                    },
                    LookupField {
                        key: "Modules".into(),
                        value: format!("{}", u.modules.len()),
                        action: None,
                    },
                    LookupField {
                        key: "Dependencies".into(),
                        value: format!("{}", u.dependencies.len()),
                        action: None,
                    },
                ],
            }
        }
        _ => LookupSection {
            title: format!("Cmd {}", idx),
            fields: vec![LookupField {
                key: "Type".into(),
                value: "Unsupported command type".into(),
                action: None,
            }],
        },
    }
}

/// Derive a human-readable transaction type from the list of PTB commands.
/// Derive a human-readable label for a transaction kind.
pub(super) fn summarize_tx_kind(tx: &iota_sdk::types::SignedTransaction) -> String {
    match &tx.transaction {
        iota_sdk::types::Transaction::V1(tx_v1) => match &tx_v1.kind {
            iota_sdk::types::TransactionKind::ProgrammableTransaction(ptx) => {
                summarize_transaction(&ptx.commands)
            }
            iota_sdk::types::TransactionKind::Genesis(_) => "Genesis".to_string(),
            iota_sdk::types::TransactionKind::ConsensusCommitPrologueV1(_) => {
                "Consensus Commit".to_string()
            }
            iota_sdk::types::TransactionKind::AuthenticatorStateUpdateV1(_) => {
                "Auth State Update".to_string()
            }
            iota_sdk::types::TransactionKind::RandomnessStateUpdate(_) => {
                "Randomness Update".to_string()
            }
            iota_sdk::types::TransactionKind::EndOfEpoch(_) => "End of Epoch".to_string(),
            _ => "System".to_string(),
        },
        _ => "Unknown".to_string(),
    }
}

/// Derive a human-readable transaction type from the list of PTB commands.
pub(super) fn summarize_transaction(commands: &[iota_sdk::types::Command]) -> String {
    // Collect the "verb" of each command
    let mut has_transfer = false;
    let mut has_merge = false;
    let mut has_publish = false;
    let mut has_upgrade = false;
    let mut move_calls: Vec<String> = Vec::new();

    for cmd in commands {
        match cmd {
            iota_sdk::types::Command::TransferObjects(_) => has_transfer = true,
            iota_sdk::types::Command::SplitCoins(_) => {}
            iota_sdk::types::Command::MergeCoins(_) => has_merge = true,
            iota_sdk::types::Command::Publish(_) => has_publish = true,
            iota_sdk::types::Command::Upgrade(_) => has_upgrade = true,
            iota_sdk::types::Command::MoveCall(mc) => {
                let label = format!("{}::{}", mc.module, mc.function);
                if !move_calls.contains(&label) {
                    move_calls.push(label);
                }
            }
            _ => {}
        }
    }

    // Publish / Upgrade are distinctive enough to lead
    if has_publish {
        return "Package Publish".to_string();
    }
    if has_upgrade {
        return "Package Upgrade".to_string();
    }

    // Check for well-known Move calls
    for mc in &move_calls {
        if mc.contains("request_add_stake") || mc.contains("request_add_delegation") {
            return "Stake".to_string();
        }
        if mc.contains("request_withdraw_stake") {
            return "Unstake".to_string();
        }
    }

    // Simple transfer pattern: SplitCoins + TransferObjects with no other Move calls
    if has_transfer && move_calls.is_empty() {
        return "Transfer".to_string();
    }

    // Merge only
    if has_merge && move_calls.is_empty() && !has_transfer {
        return "Merge".to_string();
    }

    // Move call(s) — just say "Move Call" for brevity
    if !move_calls.is_empty() {
        return "Move Call".to_string();
    }

    "PTB".to_string()
}

pub(super) fn log_error(msg: &str) {
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
