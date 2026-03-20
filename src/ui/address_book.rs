//! Address Book screen — manage saved addresses and IOTA-Names.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([Constraint::Min(6), Constraint::Length(6)]).split(area);

    draw_address_table(frame, app, layout[0]);
    draw_detail(frame, app, layout[1]);
}

fn draw_address_table(frame: &mut Frame, app: &App, area: Rect) {
    let visible_rows = area.height.saturating_sub(4) as usize;
    let combined = app.combined_address_book();
    let key_count = app.key_entry_count();

    let header = Row::new(vec!["Label", "Address", "Notes", ""])
        .style(common::header_style())
        .bottom_margin(1);

    let addr_width = area.width.saturating_sub(50) as usize;

    let rows: Vec<Row> = combined
        .iter()
        .enumerate()
        .skip(app.address_offset)
        .take(visible_rows)
        .map(|(i, entry)| {
            let is_key = i < key_count;
            let style = if i == app.address_selected {
                common::selected_style()
            } else if is_key {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(entry.label.clone()),
                Cell::from(common::truncate_address(&entry.address, addr_width.max(20))),
                Cell::from(entry.notes.clone()),
                Cell::from("⏎").style(Style::default().fg(Color::Green)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(22),
        Constraint::Min(24),
        Constraint::Length(28),
        Constraint::Length(2),
    ];

    let title = format!(
        " Address Book ({} keys + {}) ",
        key_count,
        app.address_book.len()
    );

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(common::sparkle_text(&title))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(common::sparkle_text(" Address Details "))
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(common::dim_style());

    let combined = app.combined_address_book();
    let content = if let Some(entry) = combined.get(app.address_selected) {
        let addr_width = area.width.saturating_sub(16) as usize;
        let is_key = app.address_selected < app.key_entry_count();
        let mut lines = vec![
            common::detail_line("Label", &entry.label, common::accent_style()),
            common::detail_line(
                "Address",
                &common::truncate_address(&entry.address, addr_width),
                Style::default(),
            ),
            common::detail_line("Notes", &entry.notes, common::dim_style()),
        ];
        if is_key {
            lines.push(Line::from(vec![Span::styled(
                "  (read-only key entry)",
                common::dim_style(),
            )]));
        }
        lines
    } else {
        vec![Line::from("  No address selected. Press 'a' to add one.")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
