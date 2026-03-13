//! Display types, enums, and data structures used across the TUI.

/// Which screen/tab is currently active.
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

/// A coin balance row displayed in the Coins screen.
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

/// An object row displayed in the Objects screen.
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

/// A transaction row displayed in the Transactions screen.
#[derive(Debug, Clone)]
pub struct TransactionDisplay {
    pub digest: String,
    pub status: String,
    pub gas_used: String,
    pub epoch: String,
}

/// Result of a dry-run simulation for the transaction builder.
#[derive(Debug, Clone)]
pub struct DryRunInfo {
    pub status: String,
    pub estimated_gas: Option<u64>,
    pub error: Option<String>,
}

/// A saved entry in the address book.
#[derive(Debug, Clone)]
pub struct AddressEntry {
    pub label: String,
    pub address: String,
    pub notes: String,
}

/// A key displayed in the Keys screen.
#[derive(Debug, Clone)]
pub struct KeyDisplay {
    pub alias: String,
    pub address: String,
    pub scheme: String,
    pub is_active: bool,
    pub visible: bool,
    pub private_key_hex: String,
}

/// Steps in the transaction builder wizard.
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

/// A visual PTB command in the transaction builder.
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

/// Which command type is being added in the popup.
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

/// Whether the user is in normal mode or editing a text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Which popup overlay is currently shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popup {
    Help,
    AddAddress,
    EditAddress,
    GenerateKey,
    GenerateKeyAlias,
    ImportKey,
    AddCommand,
    AddCommandForm,
    RenameKey,
    SwitchNetwork,
    Detail,
    ConfirmDeleteAddress,
    ConfirmDeleteKey,
    ConfirmQuit,
    LookupIotaName,
    ErrorLog,
}

// ── Serde impls for AddressEntry (persistence) ────────────────────

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
