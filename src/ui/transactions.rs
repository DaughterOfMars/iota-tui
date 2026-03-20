//! Transactions screen — displays transaction history.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use super::common::{
    accent_style, dim_style, header_style, selected_style, sparkle_text, truncate_address,
};
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Digest").style(header_style()),
        Cell::from("Status").style(header_style()),
        Cell::from("Gas").style(header_style()),
        Cell::from("Epoch").style(header_style()),
        Cell::from("").style(header_style()),
    ])
    .height(1);

    let visible_rows = area.height.saturating_sub(4) as usize; // border + header + border-bottom
    App::scroll_into_view(
        app.transactions_selected,
        &mut app.transactions_offset,
        visible_rows,
    );

    let rows: Vec<Row> = app
        .transactions
        .iter()
        .enumerate()
        .skip(app.transactions_offset)
        .take(visible_rows)
        .map(|(i, tx)| {
            let style = if i == app.transactions_selected {
                selected_style()
            } else {
                Style::default()
            };
            let status_style = if tx.status == "Success" {
                accent_style()
            } else {
                Style::default().fg(ratatui::style::Color::Red)
            };
            Row::new(vec![
                Cell::from(truncate_address(&tx.digest, 24)),
                Cell::from(tx.status.clone()).style(status_style),
                Cell::from(tx.gas_used.clone()),
                Cell::from(tx.epoch.clone()),
                Cell::from("⏎").style(Style::default().fg(Color::Green)),
            ])
            .style(style)
        })
        .collect();

    let title = if app.transactions.is_empty() {
        " Transactions ".to_string()
    } else {
        format!(" Transactions ({}) ", app.transactions.len())
    };

    let block = Block::default()
        .title(sparkle_text(&title))
        .title_style(header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(dim_style());

    let widths = [
        Constraint::Min(26),
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Length(8),
        Constraint::Length(2),
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    frame.render_widget(table, area);
}
