//! Transaction builder sub-state extracted from the main App struct.

use crate::app::{AddCommandType, DryRunInfo, PtbCommand, TxBuilderStep, parse_iota_amount};

/// All transaction-builder-related state, grouped into a single sub-struct.
pub struct TxBuilderState {
    pub step: TxBuilderStep,
    pub sender: usize,
    pub commands: Vec<PtbCommand>,
    pub cmd_selected: usize,
    pub gas_budget: String,
    pub edit_field: usize,
    pub edit_buffers: Vec<String>,
    pub adding_cmd: Option<AddCommandType>,
    pub dry_run: Option<DryRunInfo>,
    pub dry_running: bool,
    pub dry_run_dirty: bool,
    pub gas_edited: bool,
    pub multi_values: Vec<String>,
}

impl Default for TxBuilderState {
    fn default() -> Self {
        Self {
            step: TxBuilderStep::SelectSender,
            sender: 0,
            commands: vec![],
            cmd_selected: 0,
            gas_budget: "10000000".into(),
            edit_field: 0,
            edit_buffers: vec![],
            adding_cmd: None,
            dry_run: None,
            dry_running: false,
            dry_run_dirty: true,
            gas_edited: false,
            multi_values: vec![],
        }
    }
}

impl TxBuilderState {
    pub fn reset(&mut self) {
        self.step = TxBuilderStep::SelectSender;
        self.commands.clear();
        self.cmd_selected = 0;
        self.gas_budget = "10000000".into();
        self.edit_field = 0;
        self.edit_buffers = vec![];
        self.adding_cmd = None;
        self.multi_values.clear();
        self.dry_run = None;
        self.dry_running = false;
        self.dry_run_dirty = true;
        self.gas_edited = false;
    }

    /// Calculate total IOTA nanos being transferred by all TransferIota commands.
    pub fn total_transfer_nanos(&self) -> u64 {
        self.commands
            .iter()
            .filter_map(|cmd| {
                if let PtbCommand::TransferIota { amount, .. } = cmd {
                    parse_iota_amount(amount)
                } else {
                    None
                }
            })
            .sum()
    }

    /// Returns true if the current form field accepts an address (alias-completable).
    pub fn is_address_field(&self) -> bool {
        let Some(ct) = self.adding_cmd else {
            return false;
        };
        matches!(
            (ct, self.edit_field),
            (AddCommandType::TransferIota, 0)
                | (AddCommandType::TransferObjects, 0)
                | (AddCommandType::Stake, 1)
        )
    }

    /// Returns true if the current form field accepts an object ID.
    pub fn is_object_field(&self) -> bool {
        let Some(ct) = self.adding_cmd else {
            return false;
        };
        matches!(
            (ct, self.edit_field),
            (AddCommandType::TransferObjects, 1)
                | (AddCommandType::SplitCoins, 0)
                | (AddCommandType::MergeCoins, 0)
                | (AddCommandType::MergeCoins, 1)
                | (AddCommandType::Unstake, 0)
        )
    }

    /// Returns true if the current object field should suggest coins specifically.
    pub fn is_coin_field(&self) -> bool {
        let Some(ct) = self.adding_cmd else {
            return false;
        };
        matches!(
            (ct, self.edit_field),
            (AddCommandType::SplitCoins, 0)
                | (AddCommandType::MergeCoins, 0)
                | (AddCommandType::MergeCoins, 1)
        )
    }

    /// Returns true if the current field accepts multiple values (added one at a time).
    pub fn is_multi_value_field(&self) -> bool {
        let Some(ct) = self.adding_cmd else {
            return false;
        };
        matches!(
            (ct, self.edit_field),
            (AddCommandType::TransferObjects, 1)
                | (AddCommandType::SplitCoins, 1)
                | (AddCommandType::MergeCoins, 1)
        )
    }
}
