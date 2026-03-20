//! UI rendering — draws all screens, popups, and shared components.

mod address_book;
mod coins;
pub(crate) mod common;
mod explorer;
mod keys;
mod objects;
mod packages;
pub(crate) mod popups;
mod transactions;
mod tx_builder;

use crate::app::{App, Screen};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    common::sync_color_phase(app.color_phase);
    let area = frame.area();
    app.frame_area = area;

    let sw = common::sidebar_width(app.sidebar_collapsed);

    // Horizontal split: sidebar | main content
    let h_layout = ratatui::layout::Layout::horizontal([
        ratatui::layout::Constraint::Length(sw),
        ratatui::layout::Constraint::Min(10),
    ])
    .split(area);

    let sidebar_area = h_layout[0];
    let main_area = h_layout[1];

    // Vertical split of main area: content + status bar
    let v_layout = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Min(10),   // content
        ratatui::layout::Constraint::Length(1), // status bar
    ])
    .split(main_area);

    common::draw_sidebar(frame, app, sidebar_area);

    // Update layout info for scroll calculations and mouse hit-testing
    app.content_visible_rows = v_layout[0].height.saturating_sub(4) as usize;
    app.content_area_y = v_layout[0].y;
    app.content_area = v_layout[0];
    let layout = [v_layout[0], v_layout[1]]; // content, status

    match app.screen {
        Screen::Coins => coins::draw(frame, app, layout[0]),
        Screen::Objects => objects::draw(frame, app, layout[0]),
        Screen::Transactions => transactions::draw(frame, app, layout[0]),
        Screen::Packages => packages::draw(frame, app, layout[0]),
        Screen::AddressBook => address_book::draw(frame, app, layout[0]),
        Screen::Keys => keys::draw(frame, app, layout[0]),
        Screen::TxBuilder => tx_builder::draw(frame, app, layout[0]),
        Screen::Explorer => explorer::draw(frame, app, layout[0]),
    }

    common::draw_status_bar(frame, app, layout[1]);

    // Draw popup overlay last
    if app.popup.is_some() {
        popups::draw_popup(frame, app);
    }
}
