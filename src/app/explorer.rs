//! Explorer sub-state extracted from the main App struct.

use tokio::sync::mpsc;

use crate::app::*;
use crate::wallet::WalletCmd;

/// All explorer-related state, grouped into a single sub-struct.
pub struct ExplorerState {
    pub view: ExplorerView,
    pub overview: Option<NetworkOverview>,
    pub checkpoints: Vec<CheckpointDisplay>,
    pub checkpoints_selected: usize,
    pub checkpoints_offset: usize,
    pub checkpoints_sort_asc: bool,
    pub checkpoints_filter: Option<String>,
    pub checkpoints_cursor: Option<String>,
    pub checkpoints_cursors: Vec<Option<String>>,
    pub checkpoints_has_next: bool,
    pub checkpoints_page: usize,
    pub validators: Vec<ValidatorDisplay>,
    pub validators_selected: usize,
    pub validators_offset: usize,
    pub lookup_result: Option<LookupResult>,
    pub search_results: Vec<ObjectDisplay>,
    pub search_selected: usize,
    pub search_offset: usize,
    pub search_mode: bool,
    pub search_type: String,
    pub search_cursor: Option<String>,
    pub search_has_next: bool,
    pub search_cursors: Vec<Option<String>>,
    pub lookup_selected: usize,
    pub lookup_offset: usize,
    pub lookup_query: Option<String>,
    pub lookup_address: Option<String>,
    pub lookup_obj_cursor: Option<String>,
    pub lookup_obj_cursors: Vec<Option<String>>,
    pub lookup_obj_has_next: bool,
    pub lookup_obj_page: usize,
    pub lookup_tx_cursor: Option<String>,
    pub lookup_tx_cursors: Vec<Option<String>>,
    pub lookup_tx_has_next: bool,
    pub lookup_tx_page: usize,
    pub visible_rows: usize,
    /// Y position of the pagination row (set during rendering, used for mouse hit-testing).
    pub pagination_row_y: u16,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            view: ExplorerView::Overview,
            overview: None,
            checkpoints: vec![],
            checkpoints_selected: 0,
            checkpoints_offset: 0,
            checkpoints_sort_asc: false,
            checkpoints_filter: None,
            checkpoints_cursor: None,
            checkpoints_cursors: vec![],
            checkpoints_has_next: false,
            checkpoints_page: 0,
            validators: vec![],
            validators_selected: 0,
            validators_offset: 0,
            lookup_result: None,
            search_results: vec![],
            search_selected: 0,
            search_offset: 0,
            search_mode: false,
            search_type: String::new(),
            search_cursor: None,
            search_has_next: false,
            search_cursors: vec![],
            lookup_selected: 0,
            lookup_offset: 0,
            lookup_query: None,
            lookup_address: None,
            lookup_obj_cursor: None,
            lookup_obj_cursors: vec![],
            lookup_obj_has_next: false,
            lookup_obj_page: 0,
            lookup_tx_cursor: None,
            lookup_tx_cursors: vec![],
            lookup_tx_has_next: false,
            lookup_tx_page: 0,
            visible_rows: 15,
            pagination_row_y: 0,
        }
    }
}

impl ExplorerState {
    /// Return checkpoint indices matching the current filter and sort order.
    pub fn filtered_checkpoints(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = if let Some(ref q) = self.checkpoints_filter {
            self.checkpoints
                .iter()
                .enumerate()
                .filter(|(_, cp)| cp.sequence.to_string().contains(q))
                .map(|(i, _)| i)
                .collect()
        } else {
            (0..self.checkpoints.len()).collect()
        };
        if self.checkpoints_sort_asc {
            indices.sort_by(|a, b| {
                self.checkpoints[*a]
                    .sequence
                    .cmp(&self.checkpoints[*b].sequence)
            });
        } else {
            indices.sort_by(|a, b| {
                self.checkpoints[*b]
                    .sequence
                    .cmp(&self.checkpoints[*a].sequence)
            });
        }
        indices
    }

    /// Refresh explorer data for the current sub-view.
    pub fn refresh_explorer(&mut self, cmd_tx: &mpsc::Sender<WalletCmd>) {
        match self.view {
            ExplorerView::Overview => {
                let _ = cmd_tx.try_send(WalletCmd::RefreshNetworkOverview);
            }
            ExplorerView::Checkpoints => {
                self.checkpoints_cursor = None;
                self.checkpoints_cursors.clear();
                self.checkpoints_has_next = false;
                self.checkpoints_page = 0;
                let _ = cmd_tx.try_send(WalletCmd::RefreshCheckpoints { cursor: None });
            }
            ExplorerView::Validators => {
                let _ = cmd_tx.try_send(WalletCmd::RefreshValidators);
            }
            ExplorerView::Lookup => {}
        }
    }
}
