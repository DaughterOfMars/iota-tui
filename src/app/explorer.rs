//! Explorer sub-state extracted from the main App struct.

use crate::app::*;

/// All explorer-related state, grouped into a single sub-struct.
pub struct ExplorerState {
    pub overview: Option<NetworkOverview>,
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
    /// Which section the cursor is on (index into sections vec).
    pub lookup_section: usize,
    /// 0 = cursor is on section heading, 1 = cursor is on a field within the section.
    pub lookup_depth: usize,
    /// Which field within the current section (only meaningful when lookup_depth == 1).
    pub lookup_field_idx: usize,
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

impl ExplorerState {
    /// Reset state for a new lookup query (address/object/transaction).
    pub fn reset_for_lookup(&mut self, query: &str) {
        self.search_mode = false;
        self.lookup_result = None;
        self.lookup_selected = 0;
        self.lookup_offset = 0;
        self.lookup_section = 0;
        self.lookup_depth = 0;
        self.lookup_field_idx = 0;
        self.lookup_query = Some(query.to_string());
        self.lookup_address = Some(query.to_string());
        self.lookup_obj_cursor = None;
        self.lookup_obj_cursors.clear();
        self.lookup_obj_has_next = false;
        self.lookup_obj_page = 0;
        self.lookup_tx_cursor = None;
        self.lookup_tx_cursors.clear();
        self.lookup_tx_has_next = false;
        self.lookup_tx_page = 0;
        self.search_results.clear();
        self.search_has_next = false;
        self.search_cursor = None;
        self.search_cursors.clear();
    }

    /// Reset state for a new type search.
    pub fn reset_for_search(&mut self, type_filter: &str) {
        self.search_mode = true;
        self.lookup_result = None;
        self.lookup_section = 0;
        self.lookup_depth = 0;
        self.lookup_field_idx = 0;
        self.search_results.clear();
        self.search_selected = 0;
        self.search_offset = 0;
        self.search_has_next = false;
        self.search_cursor = None;
        self.search_cursors.clear();
        self.search_type = type_filter.to_string();
    }
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            overview: None,
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
            lookup_section: 0,
            lookup_depth: 0,
            lookup_field_idx: 0,
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
