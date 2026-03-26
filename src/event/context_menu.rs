//! Context menu event handling — navigate and execute actions.

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, ContextAction, Popup, Section};
use crate::wallet::WalletCmd;

/// Handle keyboard events when the context menu is open.
pub fn handle_context_menu_key(app: &mut App, key: KeyEvent) {
    let Some(ref mut menu) = app.context_menu else {
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.context_menu = None;
        }
        KeyCode::Up => {
            if menu.selected > 0 {
                menu.selected -= 1;
            } else {
                menu.selected = menu.actions.len().saturating_sub(1);
            }
        }
        KeyCode::Down => {
            menu.selected = (menu.selected + 1) % menu.actions.len();
        }
        KeyCode::Enter => {
            let action = menu.actions[menu.selected];
            let section = menu.section;
            app.context_menu = None;
            execute_context_action(app, section, action);
        }
        KeyCode::Char(c) => {
            // Jump to action by shortcut character
            if let Some(idx) = menu.actions.iter().position(|a| a.shortcut() == c) {
                let action = menu.actions[idx];
                let section = menu.section;
                app.context_menu = None;
                execute_context_action(app, section, action);
            }
        }
        _ => {}
    }
}

/// Execute a context menu action.
pub fn execute_context_action(app: &mut App, section: Section, action: ContextAction) {
    // Set the legacy screen so existing handlers work
    app.screen = section.to_screen();

    match action {
        ContextAction::Send => {
            // Open quick transfer popup
            app.quick_transfer_field = 0;
            app.quick_transfer_buffers = [String::new(), String::new()];
            app.open_popup(Popup::QuickTransfer);
        }
        ContextAction::Merge => {
            app.merge_coins_for_selected();
        }
        ContextAction::Split => {
            app.open_popup(Popup::SplitCoin);
        }
        ContextAction::Transfer => {
            // Object transfer
            app.open_popup(Popup::ObjectTransfer);
        }
        ContextAction::Unstake => {
            // Pre-fill TxBuilder with Unstake command and open overlay
            if let Some(stake) = app.stakes.get(app.stakes_selected) {
                let staked_id = stake.object_id.clone();
                app.tx.reset();
                app.tx.commands.push(crate::app::PtbCommand::Unstake {
                    staked_iota_id: staked_id,
                });
                app.tx.step = crate::app::TxBuilderStep::Review;
                app.tx_builder_open = true;
                app.send_cmd(WalletCmd::DryRun {
                    sender_idx: app.tx.sender,
                    commands: app.tx.commands.clone(),
                });
            }
        }
        ContextAction::CopyId => {
            app.copy_selected();
        }
        ContextAction::CopyDigest => {
            app.copy_selected();
        }
        ContextAction::Explore => {
            // Explore the selected item
            let query = match section {
                Section::Coins => app
                    .coins
                    .get(app.coins_selected)
                    .map(|c| c.object_id.clone()),
                Section::Objects => app
                    .objects
                    .get(app.objects_selected)
                    .map(|o| o.object_id.clone()),
                _ => None,
            };
            if let Some(q) = query {
                app.search_buffer = q.clone();
                app.exploring = Some(q.clone());
                app.send_cmd(WalletCmd::LookupAddress(q));
            }
        }
        ContextAction::ExplorePackage => {
            let indices = app.package_indices();
            if let Some(&obj_idx) = indices.get(app.packages_selected) {
                let pkg_id = app.objects[obj_idx].object_id.clone();
                app.search_buffer = pkg_id.clone();
                app.exploring = Some(pkg_id.clone());
                app.send_cmd(WalletCmd::LookupAddress(pkg_id));
            }
        }
        ContextAction::ViewDetails => {
            app.popup_scroll = 0;
            app.open_popup(Popup::Detail);
        }
    }
}
