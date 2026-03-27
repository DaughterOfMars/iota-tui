//! Grid view event handling — box navigation and search input.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, ContextMenu, InputMode, Popup, Section, actions_for};
use crate::wallet::WalletCmd;

/// Handle keyboard events when the grid view is active (no overlay open).
pub fn handle_grid_key(app: &mut App, key: KeyEvent) {
    if app.search_focused {
        handle_search_key(app, key);
        return;
    }

    match key.code {
        // Scroll within focused box
        KeyCode::Up => scroll_focused_box(app, -1),
        KeyCode::Down => scroll_focused_box(app, 1),

        // Navigate between boxes
        KeyCode::Tab => cycle_focus(app, 1),
        KeyCode::BackTab => cycle_focus(app, -1),
        KeyCode::Left => move_focus(app, -1, 0),
        KeyCode::Right => move_focus(app, 1, 0),

        // Open section overlay
        KeyCode::Enter => {
            let count = section_item_count(app, app.focused_section);
            if count > 0 {
                app.section_open = Some(app.focused_section);
            }
        }

        // Context menu
        KeyCode::Char('/') => open_context_menu(app),

        // Settings
        KeyCode::Char(',') => {
            app.open_popup(Popup::Settings);
        }

        // Tx Builder
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.tx_builder_open = true;
        }

        // Search bar focus
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.search_focused = true;
            app.input_mode = InputMode::Editing;
        }

        // Copy selected item
        KeyCode::Char('c') => {
            app.copy_selected();
        }

        // Back from exploring
        KeyCode::Esc if app.exploring.is_some() => {
            app.exploring = None;
            app.request_refresh();
        }

        _ => {}
    }
}

/// Handle keyboard events when the search bar is focused.
fn handle_search_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.search_focused = false;
            app.input_mode = InputMode::Normal;
            if app.search_buffer.is_empty() && app.exploring.is_some() {
                app.exploring = None;
                app.request_refresh();
            }
        }
        KeyCode::Enter => {
            let query = app.search_buffer.trim().to_string();
            if !query.is_empty() {
                app.exploring = Some(query.clone());
                app.search_focused = false;
                app.input_mode = InputMode::Normal;
                // Send lookup command based on query content
                app.send_cmd(WalletCmd::LookupAddress(query));
            }
        }
        KeyCode::Char(c) => {
            app.search_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.search_buffer.pop();
        }
        _ => {}
    }
}

/// Open context menu for the currently selected item in the focused section.
fn open_context_menu(app: &mut App) {
    let count = section_item_count(app, app.focused_section);
    if count == 0 {
        return;
    }

    let is_own = app.exploring.is_none();
    let actions = actions_for(app.focused_section, is_own);

    if actions.is_empty() {
        return;
    }

    // Compute anchor position from focused section's grid box area
    let (anchor_row, anchor_col) = if let Some((_, rect)) = app
        .grid_box_areas
        .iter()
        .find(|(s, _)| *s == app.focused_section)
    {
        let (selected, offset) = section_selection(app, app.focused_section);
        let row_in_box = selected.saturating_sub(offset) as u16;
        (
            rect.y + row_in_box + 1, // +1 for border
            rect.x + rect.width / 2,
        )
    } else {
        (app.frame_area.height / 2, app.frame_area.width / 2)
    };

    app.context_menu = Some(ContextMenu {
        section: app.focused_section,
        actions,
        selected: 0,
        anchor_row,
        anchor_col,
    });
}

/// Move focus between boxes in the grid using directional movement.
/// dx: -1 (left), +1 (right); dy: -1 (up), +1 (down)
fn move_focus(app: &mut App, dx: i32, dy: i32) {
    if app.grid_box_areas.is_empty() {
        return;
    }

    // Find current box position
    let current_rect = app
        .grid_box_areas
        .iter()
        .find(|(s, _)| *s == app.focused_section)
        .map(|(_, r)| *r);

    let Some(cur) = current_rect else { return };

    // For horizontal movement, find the adjacent box
    if dx != 0 {
        let sections: Vec<Section> = app.grid_box_areas.iter().map(|(s, _)| *s).collect();
        if let Some(idx) = sections.iter().position(|s| *s == app.focused_section) {
            let new_idx = if dx > 0 {
                if idx + 1 < sections.len() { idx + 1 } else { 0 }
            } else if idx > 0 {
                idx - 1
            } else {
                sections.len() - 1
            };
            app.focused_section = sections[new_idx];
        }
        return;
    }

    // For vertical movement, find the box above/below
    if dy != 0 {
        let cur_center_x = cur.x + cur.width / 2;
        let cur_center_y = cur.y + cur.height / 2;

        let mut best: Option<(Section, i32)> = None;

        for &(section, rect) in &app.grid_box_areas {
            if section == app.focused_section {
                continue;
            }

            let center_y = rect.y as i32 + rect.height as i32 / 2;
            let center_x = rect.x as i32 + rect.width as i32 / 2;
            let cur_cy = cur_center_y as i32;

            // Check direction
            let correct_direction = if dy > 0 {
                center_y > cur_cy
            } else {
                center_y < cur_cy
            };

            if !correct_direction {
                continue;
            }

            // Score: prefer small vertical distance, then small horizontal distance
            let vert_dist = (center_y - cur_cy).abs();
            let horiz_dist = (center_x - cur_center_x as i32).abs();
            let score = vert_dist * 100 + horiz_dist;

            if best.is_none() || score < best.unwrap().1 {
                best = Some((section, score));
            }
        }

        if let Some((section, _)) = best {
            app.focused_section = section;
        }
    }
}

/// Open context menu for a specific section at a given screen position.
pub fn open_context_menu_at(app: &mut App, section: Section, anchor_row: u16, anchor_col: u16) {
    let is_own = app.exploring.is_none();
    let actions = actions_for(section, is_own);
    if actions.is_empty() {
        return;
    }

    app.focused_section = section;
    app.context_menu = Some(ContextMenu {
        section,
        actions,
        selected: 0,
        anchor_row,
        anchor_col,
    });
}

pub fn section_item_count(app: &App, section: Section) -> usize {
    match section {
        Section::Coins => app.coins.len(),
        Section::Objects => app.objects.len(),
        Section::Staking => app.stakes.len(),
        Section::Transactions => app.transactions.len(),
        Section::Packages => app.package_indices().len(),
    }
}

fn section_selection(app: &App, section: Section) -> (usize, usize) {
    match section {
        Section::Coins => (app.coins_selected, app.coins_offset),
        Section::Objects => (app.objects_selected, app.objects_offset),
        Section::Staking => (app.stakes_selected, app.stakes_offset),
        Section::Transactions => (app.transactions_selected, app.transactions_offset),
        Section::Packages => (app.packages_selected, app.packages_offset),
    }
}

/// Scroll the selected item within the focused grid box by `delta` (+1 down, -1 up).
fn scroll_focused_box(app: &mut App, delta: i32) {
    let section = app.focused_section;
    let count = section_item_count(app, section);
    if count == 0 {
        return;
    }

    // Compute visible rows from the grid box area (minus 2 for borders)
    let visible = app
        .grid_box_areas
        .iter()
        .find(|(s, _)| *s == section)
        .map(|(_, r)| r.height.saturating_sub(2) as usize)
        .unwrap_or(10);

    let (sel, off) = section_selection(app, section);
    let new_sel = if delta > 0 {
        (sel + 1).min(count - 1)
    } else {
        sel.saturating_sub(1)
    };
    let new_off = if new_sel < off {
        new_sel
    } else if new_sel >= off + visible {
        new_sel + 1 - visible
    } else {
        off
    };

    set_section_selection(app, section, new_sel, new_off);
}

/// Set the selected index and scroll offset for a section.
fn set_section_selection(app: &mut App, section: Section, sel: usize, off: usize) {
    match section {
        Section::Coins => {
            app.coins_selected = sel;
            app.coins_offset = off;
        }
        Section::Objects => {
            app.objects_selected = sel;
            app.objects_offset = off;
        }
        Section::Staking => {
            app.stakes_selected = sel;
            app.stakes_offset = off;
        }
        Section::Transactions => {
            app.transactions_selected = sel;
            app.transactions_offset = off;
        }
        Section::Packages => {
            app.packages_selected = sel;
            app.packages_offset = off;
        }
    }
}

/// Scroll within a grid box under the mouse cursor.
pub fn scroll_grid_box(app: &mut App, col: u16, row: u16, delta: i32) {
    for &(section, rect) in &app.grid_box_areas {
        if col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
        {
            app.focused_section = section;
            scroll_focused_box(app, delta);
            return;
        }
    }
}

/// Cycle focus linearly through the grid boxes.
fn cycle_focus(app: &mut App, direction: i32) {
    if app.grid_box_areas.is_empty() {
        return;
    }
    let sections: Vec<Section> = app.grid_box_areas.iter().map(|(s, _)| *s).collect();
    let current = sections
        .iter()
        .position(|s| *s == app.focused_section)
        .unwrap_or(0);
    let len = sections.len();
    let next = if direction > 0 {
        (current + 1) % len
    } else {
        (current + len - 1) % len
    };
    app.focused_section = sections[next];
}
