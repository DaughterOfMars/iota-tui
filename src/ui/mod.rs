//! UI rendering — draws all screens, popups, and shared components.

mod address_book;
mod coins;
pub(crate) mod common;
mod context_menu;
mod explorer;
pub(crate) mod grid;
mod keys;
mod objects;
mod packages;
pub(crate) mod popups;
mod staking;
mod transactions;
mod tx_builder;

use crate::app::{App, Section};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    common::sync_color_phase(app.color_phase);
    let area = frame.area();
    app.frame_area = area;

    // Update layout info for scroll calculations
    app.content_visible_rows = area.height.saturating_sub(4) as usize;
    app.content_area_y = area.y;
    app.content_area = area;

    // Main view: grid or section overlay or explorer
    if let Some(section) = app.section_open {
        draw_section_overlay(frame, app, area, section);
    } else if app.exploring.is_some() {
        draw_explorer_overlay(frame, app, area);
    } else if app.tx_builder_open {
        // Tx Builder as full-screen overlay
        tx_builder::draw(frame, app, area);
    } else {
        // Grid view (the main view)
        grid::draw(frame, app, area);
    }

    // Draw context menu on top if active
    if app.context_menu.is_some() {
        context_menu::draw(frame, app);
    }

    // Draw popup overlay last
    if app.popup.is_some() {
        popups::draw_popup(frame, app);
    }

    // Draw toast notification overlay
    if let Some((ref msg, ref instant)) = app.clipboard_toast
        && instant.elapsed() < std::time::Duration::from_secs(2)
    {
        let msg_width = msg.len() as u16 + 4;
        let toast_area = ratatui::layout::Rect::new(
            area.width.saturating_sub(msg_width + 1),
            area.height.saturating_sub(2),
            msg_width.min(area.width),
            1,
        );
        let toast = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(
                format!("  {}  ", msg),
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(ratatui::style::Color::Green),
            ),
        ]));
        frame.render_widget(ratatui::widgets::Clear, toast_area);
        frame.render_widget(toast, toast_area);
    }
}

/// Draw a section as a near-full-screen overlay.
fn draw_section_overlay(
    frame: &mut Frame,
    app: &mut App,
    area: ratatui::layout::Rect,
    section: Section,
) {
    use ratatui::{
        style::Style,
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Clear},
    };

    // Compute overlay area (95% of terminal, centered)
    let margin_x = (area.width as f32 * 0.025).max(1.0) as u16;
    let margin_y = (area.height as f32 * 0.025).max(1.0) as u16;
    let overlay = ratatui::layout::Rect::new(
        area.x + margin_x,
        area.y + margin_y,
        area.width.saturating_sub(margin_x * 2),
        area.height.saturating_sub(margin_y * 2),
    );

    // Clear background
    frame.render_widget(Clear, overlay);

    // Title with close hint
    let title = format!(" {} ", section.title());
    let close_hint = " Esc to close ";

    let block = Block::default()
        .title(common::sparkle_text(&title))
        .title_style(common::header_style())
        .title_bottom(
            Line::from(Span::styled(close_hint, common::dim_style()))
                .alignment(ratatui::layout::Alignment::Right),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(common::color_at(0)));

    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    // Update content area for the overlay
    app.content_visible_rows = inner.height.saturating_sub(4) as usize;
    app.content_area = inner;

    // Set the legacy screen so existing draw functions work
    app.screen = section.to_screen();

    // Render the existing screen content inside the overlay
    match section {
        Section::Coins => coins::draw(frame, app, inner),
        Section::Objects => objects::draw(frame, app, inner),
        Section::Staking => staking::draw(frame, app, inner),
        Section::Transactions => transactions::draw(frame, app, inner),
        Section::Packages => packages::draw(frame, app, inner),
    }
}

/// Draw the explorer as a near-full-screen overlay when exploring an external entity.
fn draw_explorer_overlay(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    use ratatui::{
        style::Style,
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Clear},
    };

    let margin_x = (area.width as f32 * 0.025).max(1.0) as u16;
    let margin_y = (area.height as f32 * 0.025).max(1.0) as u16;
    let overlay = ratatui::layout::Rect::new(
        area.x + margin_x,
        area.y + margin_y,
        area.width.saturating_sub(margin_x * 2),
        area.height.saturating_sub(margin_y * 2),
    );

    frame.render_widget(Clear, overlay);

    let query_label = app
        .exploring
        .as_deref()
        .map(|q| {
            if q.len() > 30 {
                format!(" Exploring: {}…{} ", &q[..14], &q[q.len() - 10..])
            } else {
                format!(" Exploring: {} ", q)
            }
        })
        .unwrap_or_else(|| " Explorer ".to_string());

    let close_hint = " Esc to close ";

    let block = Block::default()
        .title(common::sparkle_text(&query_label))
        .title_style(common::header_style())
        .title_bottom(
            Line::from(Span::styled(close_hint, common::dim_style()))
                .alignment(ratatui::layout::Alignment::Right),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(common::color_at(0)));

    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    app.content_visible_rows = inner.height.saturating_sub(4) as usize;
    app.content_area = inner;
    app.screen = crate::app::Screen::Explorer;

    explorer::draw(frame, app, inner);
}
