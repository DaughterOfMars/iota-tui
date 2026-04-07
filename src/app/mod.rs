//! Application state and logic for the TUI.

mod address_book;
mod autocomplete;
mod detail;
mod explorer;
mod export;
mod filtering;
pub mod formatting;
mod input;
mod portfolio;
mod tx_builder;
mod types;

pub use address_book::{load_address_book, save_address_book};
pub use explorer::ExplorerState;
pub use formatting::{format_iota, parse_address, parse_iota_amount};
pub use tx_builder::TxBuilderState;
pub use types::*;

use formatting::format_balance;
use tokio::sync::mpsc;

use crate::wallet::{StoredKey, WalletCmd, WalletEvent};

/// Central application state shared between the event handler and UI renderer.
pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub popup: Option<Popup>,

    // ── Grid view state ──────────────────────────────────────────
    /// Which box is focused in the grid view.
    pub focused_section: Section,
    /// Which section is open as a full overlay (None = grid view).
    pub section_open: Option<Section>,
    /// Active context menu, if any.
    pub context_menu: Option<ContextMenu>,
    /// Whether the search bar has input focus.
    pub search_focused: bool,
    /// Search bar text buffer.
    pub search_buffer: String,
    /// Viewing an external entity's data (None = own address).
    pub exploring: Option<String>,
    /// Section overlay to return to when closing the explorer.
    pub return_to_section: Option<Section>,
    /// Settings popup tab (when settings popup is open).
    pub settings_tab: SettingsTab,
    /// Whether the tx builder overlay is open.
    pub tx_builder_open: bool,
    /// Grid box areas for mouse hit-testing (updated each frame).
    pub grid_box_areas: Vec<(Section, ratatui::layout::Rect)>,
    /// Search bar area for mouse hit-testing.
    pub search_bar_area: ratatui::layout::Rect,
    /// Network tag area for mouse hit-testing.
    pub net_tag_area: ratatui::layout::Rect,
    /// Context menu rendered area for mouse hit-testing (updated each frame).
    pub context_menu_area: ratatui::layout::Rect,
    /// Last click position and time for double-click detection.
    pub last_click: Option<(u16, u16, std::time::Instant)>,

    // Network state
    pub connected: bool,
    pub network_name: String,
    pub loading: bool,

    // Wallet command channel
    pub cmd_tx: mpsc::Sender<WalletCmd>,

    // Per-screen data
    pub coins: Vec<CoinDisplay>,
    pub coins_selected: usize,
    pub coins_offset: usize,
    pub total_balance_iota: u128,
    pub coins_filter: Option<String>,

    pub objects: Vec<ObjectDisplay>,
    pub objects_selected: usize,
    pub objects_offset: usize,
    pub objects_filter: Option<String>,

    pub transactions: Vec<TransactionDisplay>,
    pub transactions_selected: usize,
    pub transactions_offset: usize,
    pub transactions_filter: Option<String>,

    pub packages_selected: usize,
    pub packages_offset: usize,
    pub pkg_view: PackageBrowserView,
    pub pkg_selected_id: String,
    pub pkg_selected_module: String,
    pub pkg_modules: Vec<PackageModuleDisplay>,
    pub pkg_modules_selected: usize,
    pub pkg_modules_offset: usize,
    pub pkg_functions: Vec<ModuleFunctionDisplay>,
    pub pkg_functions_selected: usize,
    pub pkg_functions_offset: usize,

    pub stakes: Vec<StakeDisplay>,
    pub stakes_selected: usize,
    pub stakes_offset: usize,

    pub address_book: Vec<AddressEntry>,
    pub address_selected: usize,
    pub address_offset: usize,
    pub address_edit_field: usize,
    pub address_edit_buffers: [String; 3],

    pub keys: Vec<KeyDisplay>,
    pub keys_selected: usize,
    pub keys_offset: usize,
    pub keys_show_private: bool,
    pub keys_gen_scheme: Option<String>,

    pub tx: TxBuilderState,

    // Explorer state
    pub explorer: ExplorerState,

    // Autocomplete state for address/object fields
    pub autocomplete: Vec<(String, String)>, // (alias/label, address/object_id)
    pub autocomplete_idx: Option<usize>,

    // Error log content (loaded on demand)
    pub error_log_lines: Vec<String>,

    // Popup scroll state
    pub popup_scroll: usize,
    // Which element is focused in input popups (field, submit, or cancel)
    pub popup_focus: PopupFocus,

    // Clickable action hint areas in the status bar: (rect, action_id)
    pub hint_areas: Vec<(ratatui::layout::Rect, &'static str)>,
    // Selected row in the actions drop-down menu
    pub action_menu_selected: usize,

    // Visible rows in the content area (updated each frame)
    pub content_visible_rows: usize,
    // Absolute Y position of the content area (updated each frame)
    pub content_area_y: u16,
    // Full content area Rect (updated each frame, for icon-column hit-testing)
    pub content_area: ratatui::layout::Rect,
    // Full frame area (updated each frame, for popup area computation in mouse handler)
    pub frame_area: ratatui::layout::Rect,

    pub nav_idx: usize,
    pub color_phase: u32,

    // Toast notification (auto-dismisses after 2s)
    pub clipboard_toast: Option<(String, std::time::Instant)>,

    // Coin management popup state
    pub quick_transfer_field: usize, // 0 = recipient, 1 = amount
    pub quick_transfer_buffers: [String; 2],
    /// Merge popup: list of (object_id, symbol, balance_display, selected).
    pub merge_candidates: Vec<(String, String, String, bool)>,
    /// Merge popup: cursor position in the candidate list.
    pub merge_cursor: usize,

    // Portfolio summary mode (aggregated view)
    pub coins_summary_mode: bool,
    pub portfolio_summary: Vec<PortfolioSummary>,
    pub portfolio_selected: usize,
    pub portfolio_offset: usize,
    pub portfolio_expanded: Option<usize>,
}

impl App {
    pub fn new(cmd_tx: mpsc::Sender<WalletCmd>, initial_keys: Vec<StoredKey>) -> Self {
        let keys: Vec<KeyDisplay> = initial_keys
            .iter()
            .map(|k| KeyDisplay {
                alias: k.alias.clone(),
                address: k.address.clone(),
                scheme: k.scheme.clone(),
                is_active: k.is_active,
                visible: true,
                private_key_hex: hex::encode(&k.private_key_bytes),
            })
            .collect();

        let address_book = load_address_book();

        App {
            running: true,
            screen: Screen::Coins,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_cursor: 0,
            popup: None,

            // Grid view state
            focused_section: Section::Coins,
            section_open: None,
            context_menu: None,
            search_focused: false,
            search_buffer: String::new(),
            exploring: None,
            return_to_section: None,
            settings_tab: SettingsTab::Keys,
            tx_builder_open: false,
            grid_box_areas: vec![],
            search_bar_area: ratatui::layout::Rect::default(),
            net_tag_area: ratatui::layout::Rect::default(),
            context_menu_area: ratatui::layout::Rect::default(),
            last_click: None,

            connected: false,
            network_name: "disconnected".into(),
            loading: false,

            cmd_tx,

            coins: vec![],
            coins_selected: 0,
            coins_offset: 0,
            total_balance_iota: 0,
            coins_filter: None,

            objects: vec![],
            objects_selected: 0,
            objects_offset: 0,
            objects_filter: None,

            transactions: vec![],
            transactions_selected: 0,
            transactions_offset: 0,
            transactions_filter: None,

            packages_selected: 0,
            packages_offset: 0,
            pkg_view: PackageBrowserView::List,
            pkg_selected_id: String::new(),
            pkg_selected_module: String::new(),
            pkg_modules: vec![],
            pkg_modules_selected: 0,
            pkg_modules_offset: 0,
            pkg_functions: vec![],
            pkg_functions_selected: 0,
            pkg_functions_offset: 0,

            stakes: vec![],
            stakes_selected: 0,
            stakes_offset: 0,

            address_book,
            address_selected: 0,
            address_offset: 0,
            address_edit_field: 0,
            address_edit_buffers: [String::new(), String::new(), String::new()],

            keys,
            keys_selected: 0,
            keys_offset: 0,
            keys_show_private: false,
            keys_gen_scheme: None,

            tx: TxBuilderState::default(),

            explorer: ExplorerState::default(),

            autocomplete: vec![],
            autocomplete_idx: None,

            error_log_lines: vec![],

            popup_scroll: 0,
            popup_focus: PopupFocus::Fields,

            hint_areas: vec![],
            action_menu_selected: 0,

            content_visible_rows: 20,
            content_area_y: 2,
            content_area: ratatui::layout::Rect::default(),
            frame_area: ratatui::layout::Rect::default(),

            nav_idx: 0,
            color_phase: crate::wallet::load_theme(),

            clipboard_toast: None,

            quick_transfer_field: 0,
            quick_transfer_buffers: [String::new(), String::new()],
            merge_candidates: vec![],
            merge_cursor: 0,

            coins_summary_mode: false,
            portfolio_summary: vec![],
            portfolio_selected: 0,
            portfolio_offset: 0,
            portfolio_expanded: None,
        }
    }

    /// Handle a response from the wallet backend.
    pub fn handle_wallet_event(&mut self, event: WalletEvent) {
        self.loading = false;
        match event {
            WalletEvent::Connected(network) => {
                self.connected = true;
                self.network_name = network;
                self.request_refresh();
                // Refresh network overview for the new network
                self.explorer.overview = None;
                self.send_cmd(WalletCmd::RefreshNetworkOverview);
                // Re-run explorer lookup/search if one was active
                if let Some(query) = self.explorer.lookup_query.clone() {
                    self.send_cmd(WalletCmd::LookupAddress(query));
                }
                if self.explorer.search_mode {
                    let type_filter = self.explorer.search_type.clone();
                    if !type_filter.is_empty() {
                        self.explorer.search_cursors.clear();
                        self.explorer.search_cursor = None;
                        self.send_cmd(WalletCmd::SearchObjectsByType {
                            type_filter,
                            cursor: None,
                        });
                    }
                }
            }
            WalletEvent::Balances(balances) => {
                for b in &balances {
                    if b.coin_type.contains("IOTA") {
                        if self.visible_key_count() > 1 {
                            self.total_balance_iota += b.total_balance;
                        } else {
                            self.total_balance_iota = b.total_balance;
                        }
                    }
                }
            }
            WalletEvent::Coins { coins, owner_alias } => {
                let new_coins: Vec<CoinDisplay> = coins
                    .into_iter()
                    .map(|c| CoinDisplay {
                        balance_display: format_balance(c.balance, 9),
                        coin_type: c.coin_type,
                        symbol: c.symbol,
                        balance: c.balance,
                        object_id: c.object_id,
                        owner_alias: owner_alias.clone(),
                    })
                    .collect();
                if self.visible_key_count() > 1 {
                    self.coins.extend(new_coins);
                } else {
                    self.coins = new_coins;
                }
                if self.coins_selected >= self.coins.len() {
                    self.coins_selected = self.coins.len().saturating_sub(1);
                }
                if self.coins_summary_mode {
                    self.compute_portfolio_summary();
                }
            }
            WalletEvent::Transactions(txs) => {
                self.transactions = txs;
                if self.transactions_selected >= self.transactions.len() {
                    self.transactions_selected = self.transactions.len().saturating_sub(1);
                }
            }
            WalletEvent::Objects {
                objects,
                owner_alias,
            } => {
                let new_objects: Vec<ObjectDisplay> = objects
                    .into_iter()
                    .map(|o| ObjectDisplay {
                        object_id: o.object_id,
                        type_name: o.type_name,
                        version: o
                            .version
                            .map(|v| format!("v{}", v))
                            .unwrap_or_else(|| "?".into()),
                        digest: o.digest,
                        owner: o.owner,
                        owner_alias: owner_alias.clone(),
                    })
                    .collect();
                if self.visible_key_count() > 1 {
                    self.objects.extend(new_objects);
                } else {
                    self.objects = new_objects;
                }
                if self.objects_selected >= self.objects.len() {
                    self.objects_selected = self.objects.len().saturating_sub(1);
                }
            }
            WalletEvent::KeyGenerated {
                alias,
                address,
                scheme,
                private_key_hex,
            }
            | WalletEvent::KeyImported {
                alias,
                address,
                scheme,
                private_key_hex,
            } => {
                let is_first = self.keys.is_empty();
                self.keys.push(KeyDisplay {
                    alias: alias.clone(),
                    address,
                    scheme,
                    is_active: is_first,
                    visible: true,
                    private_key_hex,
                });
                if is_first {
                    // Close welcome popup if it's open
                    if self.popup == Some(Popup::Welcome) {
                        self.popup = None;
                    }
                    self.request_refresh();
                }
            }
            WalletEvent::DryRunResult(info) => {
                self.tx.dry_running = false;
                if !self.tx.gas_edited
                    && let Some(gas) = info.estimated_gas
                {
                    self.tx.gas_budget = gas.to_string();
                }
                self.tx.dry_run = Some(info);
            }
            WalletEvent::TxSubmitted => {
                self.tx.reset();
                self.tx_builder_open = false;
                self.section_open = Some(Section::Transactions);
                self.request_refresh();
            }
            WalletEvent::IotaNameResolved {
                name,
                label,
                notes,
                address,
            } => {
                if let Some(addr) = address {
                    let display_label = if label.is_empty() {
                        name.clone()
                    } else {
                        label
                    };
                    let display_notes = if notes.is_empty() {
                        "IOTA-Name".into()
                    } else {
                        notes
                    };
                    self.address_book.push(AddressEntry {
                        label: display_label,
                        address: addr,
                        notes: display_notes,
                    });
                    save_address_book(&self.address_book);
                }
            }
            WalletEvent::FaucetRequested(_msg) => {
                self.request_refresh();
            }
            WalletEvent::NetworkOverview {
                chain_id,
                epoch,
                gas_price,
                latest_checkpoint,
                total_transactions,
            } => {
                self.explorer.overview = Some(NetworkOverview {
                    chain_id,
                    epoch,
                    gas_price,
                    latest_checkpoint,
                    total_txs: total_transactions,
                });
            }
            WalletEvent::ExplorerLookupResult(mut result) => {
                self.explorer.lookup_selected = 0;
                self.explorer.lookup_offset = 0;
                self.explorer.lookup_section = 0;
                self.explorer.lookup_depth = 0;
                self.explorer.lookup_field_idx = 0;
                self.explorer.lookup_address = None;
                // Expand the first section by default
                if let Some(s) = result.sections_mut().first_mut() {
                    s.collapsed = false;
                }
                self.explorer.lookup_result = Some(result);
            }
            WalletEvent::AddressLookupPage {
                mut result,
                obj_cursor,
                obj_has_next,
                tx_cursor,
                tx_has_next,
            } => {
                self.explorer.lookup_selected = 0;
                self.explorer.lookup_offset = 0;
                self.explorer.lookup_section = 0;
                self.explorer.lookup_depth = 0;
                self.explorer.lookup_field_idx = 0;
                // Expand the first section by default
                if let Some(s) = result.sections_mut().first_mut() {
                    s.collapsed = false;
                }
                self.explorer.lookup_obj_cursor = obj_cursor;
                self.explorer.lookup_obj_has_next = obj_has_next;
                self.explorer.lookup_tx_cursor = tx_cursor;
                self.explorer.lookup_tx_has_next = tx_has_next;
                self.explorer.lookup_result = Some(result);
            }
            WalletEvent::ObjectSearchResults {
                objects,
                has_next_page,
                end_cursor,
            } => {
                self.explorer.search_results = objects;
                self.explorer.search_selected = 0;
                self.explorer.search_offset = 0;
                self.explorer.search_has_next = has_next_page;
                self.explorer.search_cursor = end_cursor;
            }
            WalletEvent::Stakes(stakes) => {
                self.stakes = stakes;
                if self.stakes_selected >= self.stakes.len() {
                    self.stakes_selected = self.stakes.len().saturating_sub(1);
                }
            }
            WalletEvent::PackageModules {
                package_addr,
                modules,
            } => {
                self.pkg_selected_id = package_addr;
                self.pkg_modules = modules;
                self.pkg_modules_selected = 0;
                self.pkg_modules_offset = 0;
                self.pkg_view = PackageBrowserView::Modules;
            }
            WalletEvent::ModuleFunctions {
                module_name,
                functions,
            } => {
                self.pkg_selected_module = module_name;
                self.pkg_functions = functions;
                self.pkg_functions_selected = 0;
                self.pkg_functions_offset = 0;
                self.pkg_view = PackageBrowserView::Functions;
            }
            WalletEvent::Error(_e) => {}
        }
    }

    /// Send a command to the wallet backend (non-blocking).
    pub fn send_cmd(&self, cmd: WalletCmd) {
        if let Err(tokio::sync::mpsc::error::TrySendError::Full(cmd)) = self.cmd_tx.try_send(cmd) {
            // Channel full — log and retry once after a short yield.
            // This can happen when polling floods the channel.
            let tx = self.cmd_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(cmd).await;
            });
        }
    }

    /// Number of keys with visibility enabled.
    pub fn visible_key_count(&self) -> usize {
        self.keys.iter().filter(|k| k.visible).count()
    }

    /// Returns true when multiple keys are visible (show Owner column).
    pub fn show_multiple_owners(&self) -> bool {
        self.visible_key_count() > 1
    }

    /// Request a data refresh for visible keys' addresses.
    pub fn request_refresh(&mut self) {
        self.loading = true;
        let visible_keys: Vec<KeyDisplay> =
            self.keys.iter().filter(|k| k.visible).cloned().collect();

        if visible_keys.len() > 1 {
            self.coins.clear();
            self.objects.clear();
            self.total_balance_iota = 0;
            for key in &visible_keys {
                if let Some(addr) = parse_address(&key.address) {
                    self.send_cmd(WalletCmd::RefreshCoins {
                        addr,
                        alias: key.alias.clone(),
                    });
                    self.send_cmd(WalletCmd::RefreshObjects {
                        addr,
                        alias: key.alias.clone(),
                    });
                    self.send_cmd(WalletCmd::RefreshBalances(addr));
                }
            }
            if let Some(key) = self.active_key().cloned()
                && let Some(addr) = parse_address(&key.address)
            {
                self.send_cmd(WalletCmd::RefreshTransactions(addr));
                self.send_cmd(WalletCmd::RefreshStakes(addr));
            }
        } else if let Some(key) = visible_keys.first().or(self.active_key()).cloned()
            && let Some(addr) = parse_address(&key.address)
        {
            self.send_cmd(WalletCmd::RefreshCoins {
                addr,
                alias: key.alias.clone(),
            });
            self.send_cmd(WalletCmd::RefreshObjects {
                addr,
                alias: key.alias.clone(),
            });
            self.send_cmd(WalletCmd::RefreshBalances(addr));
            self.send_cmd(WalletCmd::RefreshTransactions(addr));
            self.send_cmd(WalletCmd::RefreshStakes(addr));
        }
    }

    pub fn navigate(&mut self, screen: Screen) {
        self.screen = screen;
        self.input_mode = InputMode::Normal;
        self.popup = None;
        self.popup_scroll = 0;
        // Clear filters when switching screens
        self.coins_filter = None;
        self.objects_filter = None;
        self.transactions_filter = None;
        if screen == Screen::Explorer {
            self.send_cmd(WalletCmd::RefreshNetworkOverview);
        }
    }

    /// Navigate to Explorer > Lookup and immediately submit a lookup query.
    pub fn explore_item(&mut self, query: String) {
        // Remember which overlay to return to
        if self.return_to_section.is_none() {
            self.return_to_section = self.section_open;
        }
        // Close any open overlay/popup and switch to grid explore mode
        self.section_open = None;
        self.popup = None;
        self.popup_scroll = 0;
        self.input_mode = InputMode::Normal;
        self.context_menu = None;

        self.screen = Screen::Explorer;
        self.explorer.reset_for_lookup(&query);

        // Update search bar to show what we're exploring
        self.search_buffer = query.clone();
        self.exploring = Some(query.clone());
        self.send_cmd(WalletCmd::LookupAddress(query));
    }

    /// Navigate to Explorer > Lookup and immediately submit a type search.
    pub fn explore_type(&mut self, type_filter: String) {
        self.screen = Screen::Explorer;
        self.input_mode = InputMode::Normal;
        self.popup = None;
        self.popup_scroll = 0;
        self.explorer.reset_for_search(&type_filter);
        self.send_cmd(WalletCmd::SearchObjectsByType {
            type_filter,
            cursor: None,
        });
    }

    /// Activate the currently selected coin (explore it).
    pub fn activate_selected_coin(&mut self) {
        if let Some(coin) = self.coins.get(self.coins_selected) {
            let id = coin.object_id.clone();
            self.explore_item(id);
        }
    }

    /// Activate the currently selected object (explore it).
    pub fn activate_selected_object(&mut self) {
        if let Some(obj) = self.objects.get(self.objects_selected) {
            let id = obj.object_id.clone();
            self.explore_item(id);
        }
    }

    /// Activate the currently selected transaction (explore it).
    pub fn activate_selected_transaction(&mut self) {
        if let Some(tx) = self.transactions.get(self.transactions_selected) {
            let digest = tx.digest.clone();
            self.explore_item(digest);
        }
    }

    /// Activate the currently selected package (explore it).
    pub fn activate_selected_package(&mut self) {
        let packages = self.package_indices();
        if let Some(&obj_idx) = packages.get(self.packages_selected) {
            let id = self.objects[obj_idx].object_id.clone();
            self.explore_item(id);
        }
    }

    /// Browse modules of the currently selected package.
    pub fn browse_selected_package(&mut self) {
        let packages = self.package_indices();
        if let Some(&obj_idx) = packages.get(self.packages_selected) {
            let id = self.objects[obj_idx].object_id.clone();
            self.loading = true;
            self.send_cmd(WalletCmd::FetchPackageModules { package_addr: id });
        }
    }

    /// Browse functions of the currently selected module.
    pub fn browse_selected_module(&mut self) {
        if let Some(module) = self.pkg_modules.get(self.pkg_modules_selected) {
            let module_name = module.name.clone();
            self.loading = true;
            self.send_cmd(WalletCmd::FetchModuleFunctions {
                package_addr: self.pkg_selected_id.clone(),
                module_name,
            });
        }
    }

    /// Jump to TxBuilder with a MoveCall pre-filled from the selected function.
    pub fn jump_to_move_call(&mut self) {
        if let Some(func) = self.pkg_functions.get(self.pkg_functions_selected) {
            self.tx.reset();
            self.tx.commands.push(PtbCommand::MoveCall {
                package: self.pkg_selected_id.clone(),
                module: self.pkg_selected_module.clone(),
                function: func.name.clone(),
                type_args: vec!["".to_string(); func.type_param_count],
                args: vec!["".to_string(); func.param_types.len()],
            });
            self.tx.step = TxBuilderStep::EditCommands;
            self.section_open = None;
            self.tx_builder_open = true;
            self.screen = Screen::TxBuilder;
            self.input_mode = InputMode::Normal;
            self.popup = None;
        }
    }

    /// Activate the currently selected address book entry (explore it).
    pub fn activate_selected_address(&mut self) {
        let combined = self.combined_address_book();
        if let Some(entry) = combined.get(self.address_selected) {
            let addr = entry.address.clone();
            self.explore_item(addr);
        }
    }

    /// Activate the currently selected key (set it as active).
    pub fn activate_selected_key(&mut self) {
        let idx = self.keys_selected;
        if idx < self.keys.len() {
            for (i, k) in self.keys.iter_mut().enumerate() {
                k.is_active = i == idx;
            }
            self.send_cmd(WalletCmd::SetActiveKey(idx));
            self.request_refresh();
        }
    }

    /// Open the merge coin popup, populating candidates with all coins of the
    /// same type as the currently selected coin.
    pub fn merge_coins_for_selected(&mut self) {
        let Some(coin) = self.coins.get(self.coins_selected) else {
            return;
        };
        let coin_type = coin.coin_type.clone();
        let selected_id = coin.object_id.clone();
        let candidates: Vec<(String, String, String, bool)> = self
            .coins
            .iter()
            .filter(|c| c.coin_type == coin_type)
            .map(|c| {
                (
                    c.object_id.clone(),
                    c.symbol.clone(),
                    c.balance_display.clone(),
                    // Pre-select all except the primary (first selected) coin
                    c.object_id != selected_id,
                )
            })
            .collect();
        if candidates.len() < 2 {
            self.clipboard_toast = Some((
                "Only one coin of this type — nothing to merge".into(),
                std::time::Instant::now(),
            ));
            return;
        }
        self.merge_candidates = candidates;
        self.merge_cursor = 0;
        self.open_popup(Popup::MergeCoin);
    }

    /// Execute merge for the coins selected in the merge popup.
    /// The first selected coin becomes the primary; the rest are sources.
    pub fn finalize_merge_coins(&mut self) {
        let selected: Vec<String> = self
            .merge_candidates
            .iter()
            .filter(|(_, _, _, sel)| *sel)
            .map(|(id, _, _, _)| id.clone())
            .collect();
        if selected.len() < 2 {
            self.clipboard_toast = Some((
                "Select at least 2 coins to merge".into(),
                std::time::Instant::now(),
            ));
            return;
        }
        let primary = selected[0].clone();
        let sources = selected[1..].to_vec();
        self.tx.reset();
        self.tx
            .commands
            .push(PtbCommand::MergeCoins { primary, sources });
        self.tx.step = TxBuilderStep::Review;
        self.section_open = None;
        self.tx_builder_open = true;
        self.screen = Screen::TxBuilder;
        self.input_mode = InputMode::Normal;
        self.popup = None;
        self.send_cmd(WalletCmd::DryRun {
            sender_idx: self.tx.sender,
            commands: self.tx.commands.clone(),
        });
        self.tx.dry_running = true;
        self.tx.dry_run_dirty = false;
    }

    /// Build a SplitCoins PTB from the selected coin, splitting into `n` equal parts.
    pub fn split_selected_coin(&mut self, n: usize) {
        let Some(coin) = self.coins.get(self.coins_selected) else {
            return;
        };
        if n < 2 {
            self.clipboard_toast = Some((
                "Need at least 2 parts to split".into(),
                std::time::Instant::now(),
            ));
            return;
        }
        let amount_per = coin.balance / n as u128;
        if amount_per == 0 {
            self.clipboard_toast = Some((
                "Coin balance too small to split".into(),
                std::time::Instant::now(),
            ));
            return;
        }
        let amounts: Vec<String> = (0..n - 1).map(|_| amount_per.to_string()).collect();
        self.tx.reset();
        self.tx.commands.push(PtbCommand::SplitCoins {
            coin: coin.object_id.clone(),
            amounts,
        });
        self.tx.step = TxBuilderStep::Review;
        self.section_open = None;
        self.tx_builder_open = true;
        self.screen = Screen::TxBuilder;
        self.input_mode = InputMode::Normal;
        self.popup = None;
        self.send_cmd(WalletCmd::DryRun {
            sender_idx: self.tx.sender,
            commands: self.tx.commands.clone(),
        });
        self.tx.dry_running = true;
        self.tx.dry_run_dirty = false;
    }

    /// Build a TransferIota PTB from the quick transfer popup fields.
    pub fn finalize_quick_transfer(&mut self) {
        let [ref recipient, ref amount] = self.quick_transfer_buffers;
        if recipient.is_empty() || amount.is_empty() {
            return;
        }
        let resolved = self.resolve_address(recipient);
        let Some(_nanos) = parse_iota_amount(amount) else {
            self.clipboard_toast = Some(("Invalid amount".into(), std::time::Instant::now()));
            return;
        };
        self.tx.reset();
        self.tx.commands.push(PtbCommand::TransferIota {
            recipient: resolved,
            amount: amount.clone(),
        });
        self.tx.step = TxBuilderStep::Review;
        self.section_open = None;
        self.tx_builder_open = true;
        self.screen = Screen::TxBuilder;
        self.input_mode = InputMode::Normal;
        self.popup = None;
        self.send_cmd(WalletCmd::DryRun {
            sender_idx: self.tx.sender,
            commands: self.tx.commands.clone(),
        });
        self.tx.dry_running = true;
        self.tx.dry_run_dirty = false;
    }

    pub fn finalize_object_transfer(&mut self) {
        let recipient = self.input_buffer.clone();
        if recipient.is_empty() {
            return;
        }
        let filtered = self.filtered_objects();
        let Some(&real_idx) = filtered.get(self.objects_selected) else {
            return;
        };
        let obj = &self.objects[real_idx];
        let object_id = obj.object_id.clone();
        let resolved = self.resolve_address(&recipient);
        self.tx.reset();
        self.tx.commands.push(PtbCommand::TransferObjects {
            recipient: resolved,
            object_ids: vec![object_id],
        });
        self.tx.step = TxBuilderStep::Review;
        self.section_open = None;
        self.tx_builder_open = true;
        self.screen = Screen::TxBuilder;
        self.input_mode = InputMode::Normal;
        self.popup = None;
        self.send_cmd(WalletCmd::DryRun {
            sender_idx: self.tx.sender,
            commands: self.tx.commands.clone(),
        });
        self.tx.dry_running = true;
        self.tx.dry_run_dirty = false;
    }

    pub fn open_popup(&mut self, popup: Popup) {
        self.popup = Some(popup);
        self.popup_scroll = 0;
        self.popup_focus = PopupFocus::Fields;
    }

    /// Validate that available balance covers transfers + gas.
    pub fn validate_balance(&self) -> Result<(), String> {
        let gas_budget: u64 = self.tx.gas_budget.parse().unwrap_or(10_000_000);
        let transfer_total = self.tx.total_transfer_nanos();
        let required = transfer_total as u128 + gas_budget as u128;
        if required > self.total_balance_iota {
            Err(format!(
                "Insufficient balance: need {} IOTA but have {}",
                format_iota(required),
                format_iota(self.total_balance_iota),
            ))
        } else {
            Ok(())
        }
    }

    pub fn load_error_log(&mut self) {
        let path = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("iota-wallet-tui")
            .join("error.log");
        self.error_log_lines = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .rev()
            .take(100)
            .map(|s| s.to_string())
            .collect();
    }

    pub fn active_key(&self) -> Option<&KeyDisplay> {
        self.keys.iter().find(|k| k.is_active)
    }

    pub fn key_entry_count(&self) -> usize {
        self.keys.len()
    }

    pub fn scroll_into_view(selected: usize, offset: &mut usize, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if selected < *offset {
            *offset = selected;
        } else if selected >= *offset + visible_rows {
            *offset = selected - visible_rows + 1;
        }
    }
}
