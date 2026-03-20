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

    let layout = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(1), // tab bar
        ratatui::layout::Constraint::Length(1), // separator
        ratatui::layout::Constraint::Min(10),   // content
        ratatui::layout::Constraint::Length(1), // status bar
    ])
    .split(area);

    common::draw_tabs(frame, app, layout[0]);
    common::draw_separator(frame, layout[1]);

    // Update layout info for scroll calculations and mouse hit-testing
    app.content_visible_rows = layout[2].height.saturating_sub(4) as usize;
    app.content_area_y = layout[2].y;
    app.content_area = layout[2];

    match app.screen {
        Screen::Coins => coins::draw(frame, app, layout[2]),
        Screen::Objects => objects::draw(frame, app, layout[2]),
        Screen::Transactions => transactions::draw(frame, app, layout[2]),
        Screen::Packages => packages::draw(frame, app, layout[2]),
        Screen::AddressBook => address_book::draw(frame, app, layout[2]),
        Screen::Keys => keys::draw(frame, app, layout[2]),
        Screen::TxBuilder => tx_builder::draw(frame, app, layout[2]),
        Screen::Explorer => explorer::draw(frame, app, layout[2]),
    }

    common::draw_status_bar(frame, app, layout[3]);

    // Draw popup overlay last
    if app.popup.is_some() {
        popups::draw_popup(frame, app);
    }
}
