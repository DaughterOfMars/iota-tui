//! Display types, enums, and data structures used across the TUI.

// ── Grid sections (the box grid on the main view) ─────────────────

/// A section in the main box grid view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Coins,
    Objects,
    Staking,
    Transactions,
    Packages,
}

impl Section {
    pub fn title(self) -> &'static str {
        match self {
            Section::Coins => "Coins",
            Section::Objects => "Objects",
            Section::Staking => "Staking",
            Section::Transactions => "Transactions",
            Section::Packages => "Packages",
        }
    }

    /// Map a section to the legacy Screen variant used for full overlay rendering.
    pub fn to_screen(self) -> Screen {
        match self {
            Section::Coins => Screen::Coins,
            Section::Objects => Screen::Objects,
            Section::Staking => Screen::Staking,
            Section::Transactions => Screen::Transactions,
            Section::Packages => Screen::Packages,
        }
    }
}

// ── Context menu ──────────────────────────────────────────────────

/// An action available in the context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextAction {
    Send,
    Merge,
    Split,
    Transfer,
    Unstake,
    CopyId,
    Explore,
    ExplorePackage,
    ViewDetails,
    CopyDigest,
}

impl ContextAction {
    pub fn label(self) -> &'static str {
        match self {
            ContextAction::Send => "Send",
            ContextAction::Merge => "Merge",
            ContextAction::Split => "Split",
            ContextAction::Transfer => "Transfer",
            ContextAction::Unstake => "Unstake",
            ContextAction::CopyId => "Copy ID",
            ContextAction::Explore => "Explore",
            ContextAction::ExplorePackage => "Explore Package",
            ContextAction::ViewDetails => "View Details",
            ContextAction::CopyDigest => "Copy Digest",
        }
    }

    /// Shortcut character for quick selection in the menu.
    pub fn shortcut(self) -> char {
        match self {
            ContextAction::Send => 's',
            ContextAction::Merge => 'm',
            ContextAction::Split => 'p',
            ContextAction::Transfer => 't',
            ContextAction::Unstake => 'u',
            ContextAction::CopyId => 'c',
            ContextAction::Explore => 'e',
            ContextAction::ExplorePackage => 'e',
            ContextAction::ViewDetails => 'v',
            ContextAction::CopyDigest => 'c',
        }
    }
}

/// Returns the context menu actions available for a given section.
pub fn actions_for(section: Section, is_own: bool) -> Vec<ContextAction> {
    match section {
        Section::Coins => {
            let mut actions = vec![];
            if is_own {
                actions.extend_from_slice(&[
                    ContextAction::Send,
                    ContextAction::Merge,
                    ContextAction::Split,
                ]);
            }
            actions.push(ContextAction::CopyId);
            actions.push(ContextAction::Explore);
            actions
        }
        Section::Objects => {
            let mut actions = vec![];
            if is_own {
                actions.push(ContextAction::Transfer);
            }
            actions.push(ContextAction::CopyId);
            actions.push(ContextAction::Explore);
            actions
        }
        Section::Staking => {
            let mut actions = vec![];
            if is_own {
                actions.push(ContextAction::Unstake);
            }
            actions.push(ContextAction::CopyId);
            actions
        }
        Section::Transactions => {
            vec![ContextAction::ViewDetails, ContextAction::CopyDigest]
        }
        Section::Packages => {
            vec![ContextAction::ExplorePackage, ContextAction::CopyId]
        }
    }
}

/// State of an active context menu.
#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub section: Section,
    pub actions: Vec<ContextAction>,
    pub selected: usize,
    pub anchor_row: u16,
    pub anchor_col: u16,
}

// ── Settings popup tabs ───────────────────────────────────────────

/// Which sub-tab is active in the Settings popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Keys,
    AddressBook,
    Network,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 3] = [
        SettingsTab::Keys,
        SettingsTab::AddressBook,
        SettingsTab::Network,
    ];

    pub fn title(self) -> &'static str {
        match self {
            SettingsTab::Keys => "Keys",
            SettingsTab::AddressBook => "Address Book",
            SettingsTab::Network => "Network",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SettingsTab::Keys => SettingsTab::AddressBook,
            SettingsTab::AddressBook => SettingsTab::Network,
            SettingsTab::Network => SettingsTab::Keys,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SettingsTab::Keys => SettingsTab::Network,
            SettingsTab::AddressBook => SettingsTab::Keys,
            SettingsTab::Network => SettingsTab::AddressBook,
        }
    }
}

// ── Legacy Screen enum (used for overlay rendering) ───────────────

/// Which screen/tab is currently active (legacy — used inside section overlays).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Coins,
    Objects,
    Transactions,
    Staking,
    Packages,
    AddressBook,
    Keys,
    TxBuilder,
    Explorer,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Screen::Coins => "Coins",
            Screen::Objects => "Objects",
            Screen::Transactions => "Transactions",
            Screen::Staking => "Staking",
            Screen::Packages => "Packages",
            Screen::AddressBook => "Address Book",
            Screen::Keys => "Keys",
            Screen::TxBuilder => "Tx Builder",
            Screen::Explorer => "Explorer",
        }
    }
}

// ── Display types (what the UI renders) ────────────────────────────

/// A coin balance row displayed in the Coins screen.
#[derive(Debug, Clone)]
pub struct CoinDisplay {
    pub coin_type: String,
    pub symbol: String,
    pub balance: u128,
    pub balance_display: String,
    pub object_id: String,
    pub owner_alias: String,
}

/// Aggregated portfolio summary row (one per coin type across all accounts).
#[derive(Debug, Clone)]
pub struct PortfolioSummary {
    pub coin_type: String,
    pub symbol: String,
    pub total_balance_display: String,
    pub per_account: Vec<(String, String)>, // (alias, balance_display)
}

/// An object row displayed in the Objects screen.
#[derive(Debug, Clone)]
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
    pub tx_kind: String,
    // Extended fields for detail pane
    pub gas_computation: String,
    pub gas_storage: String,
    pub gas_rebate: String,
    pub changed_objects: usize,
}

/// A staked IOTA object displayed in the Staking screen.
#[derive(Debug, Clone)]
pub struct StakeDisplay {
    pub object_id: String,
    pub principal: String,
    pub principal_display: String,
    pub validator_address: String,
    pub activation_epoch: String,
    pub status: String,
}

/// Which sub-view of the Package Browser is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageBrowserView {
    List,
    Modules,
    Functions,
}

/// A module within a published package.
#[derive(Debug, Clone)]
pub struct PackageModuleDisplay {
    pub name: String,
    pub function_count: usize,
    pub struct_count: usize,
}

/// A function within a module.
#[derive(Debug, Clone)]
pub struct ModuleFunctionDisplay {
    pub name: String,
    pub visibility: String,
    pub is_entry: bool,
    pub type_param_count: usize,
    pub param_types: Vec<String>,
    pub return_types: Vec<String>,
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
            PtbCommand::Unstake { staked_iota_id } => truncate_id(staked_iota_id, 20),
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

/// Which element is focused within an input popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupFocus {
    /// An input field is focused (typing goes to the input buffer).
    Fields,
    /// The submit/save button is focused.
    Submit,
    /// The cancel button is focused.
    Cancel,
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
    ConfirmClearTx,
    ConfirmQuit,
    LookupIotaName,
    ErrorLog,
    ActionsMenu,
    SplitCoin,
    QuickTransfer,
    ObjectTransfer,
    Settings,
    Welcome,
}

// ── Explorer types ─────────────────────────────────────────────────

/// Which sub-view of the Explorer screen is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerView {
    Overview,
    Checkpoints,
    Validators,
    Lookup,
}

impl ExplorerView {
    pub const ALL: [ExplorerView; 4] = [
        ExplorerView::Overview,
        ExplorerView::Checkpoints,
        ExplorerView::Validators,
        ExplorerView::Lookup,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&v| v == self).unwrap_or(0)
    }
}

/// Network overview stats displayed in the Explorer Overview sub-view.
#[derive(Debug, Clone, Default)]
pub struct NetworkOverview {
    pub chain_id: String,
    pub epoch: String,
    pub gas_price: String,
    pub latest_checkpoint: String,
    pub total_txs: String,
}

/// A checkpoint row in the Explorer Checkpoints sub-view.
#[derive(Debug, Clone)]
pub struct CheckpointDisplay {
    pub sequence: u64,
    pub digest: String,
    pub timestamp: String,
    pub tx_count: u64,
}

/// A validator row in the Explorer Validators sub-view.
#[derive(Debug, Clone)]
pub struct ValidatorDisplay {
    pub name: String,
    pub address: String,
    pub stake: String,
}

/// What happens when a user presses Enter on a lookup field.
#[derive(Debug, Clone)]
pub enum LookupAction {
    /// Navigate to explorer lookup for this value.
    Explore(String),
    /// Search objects by type.
    TypeSearch(String),
}

/// A single key-value field in a lookup result, optionally navigable.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LookupField {
    pub key: String,
    pub value: String,
    pub action: Option<LookupAction>,
}

/// A titled section in the lookup result (collapsible tree node).
#[derive(Debug, Clone)]
pub struct LookupSection {
    pub title: String,
    pub fields: Vec<LookupField>,
    pub collapsed: bool,
}

/// Result of a lookup query in the Explorer Lookup sub-view.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum LookupResult {
    Object { sections: Vec<LookupSection> },
    Address { sections: Vec<LookupSection> },
    Transaction { sections: Vec<LookupSection> },
    NotFound(String),
}

impl LookupResult {
    pub fn sections(&self) -> &[LookupSection] {
        match self {
            LookupResult::Object { sections }
            | LookupResult::Address { sections }
            | LookupResult::Transaction { sections } => sections,
            LookupResult::NotFound(_) => &[],
        }
    }

    pub fn sections_mut(&mut self) -> &mut [LookupSection] {
        match self {
            LookupResult::Object { sections }
            | LookupResult::Address { sections }
            | LookupResult::Transaction { sections } => sections,
            LookupResult::NotFound(_) => &mut [],
        }
    }

    /// Total visible lines accounting for collapsed state.
    pub fn total_visible_lines(&self) -> usize {
        self.sections()
            .iter()
            .map(|s| {
                if s.collapsed {
                    1 // just the heading
                } else {
                    1 + s.fields.len() // heading + fields
                }
            })
            .sum()
    }

    /// Convert a tree cursor (section, depth, field_idx) to a visible line index.
    pub fn cursor_to_line(&self, section: usize, depth: usize, field_idx: usize) -> usize {
        let mut line = 0;
        for (i, s) in self.sections().iter().enumerate() {
            if i == section {
                if depth == 0 {
                    return line; // on the heading
                }
                return line + 1 + field_idx; // heading + offset into fields
            }
            line += 1; // heading
            if !s.collapsed {
                line += s.fields.len();
            }
        }
        line
    }

    /// Convert a visible line index to a tree cursor (section, depth, field_idx).
    /// Returns None if the line is out of range.
    #[allow(dead_code)]
    pub fn line_to_cursor(&self, line: usize) -> Option<(usize, usize, usize)> {
        let mut current_line = 0;
        for (si, section) in self.sections().iter().enumerate() {
            if current_line == line {
                return Some((si, 0, 0)); // heading
            }
            current_line += 1;
            if !section.collapsed {
                for fi in 0..section.fields.len() {
                    if current_line == line {
                        return Some((si, 1, fi)); // field
                    }
                    current_line += 1;
                }
            }
        }
        None
    }

    /// Scroll offset so that the cursor line is visible.
    pub fn scroll_cursor_into_view(
        &self,
        section: usize,
        depth: usize,
        field_idx: usize,
        offset: &mut usize,
        visible_lines: usize,
    ) {
        let target = self.cursor_to_line(section, depth, field_idx);
        if target < *offset {
            *offset = target;
        } else if target >= *offset + visible_lines {
            *offset = target + 1 - visible_lines;
        }
    }
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
