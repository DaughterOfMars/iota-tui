//! Transactions screen — displays transaction history.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common::{
    accent_style, dim_style, header_style, selected_style, sparkle_text, truncate_address,
};
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let filtering = app.transactions_filter.is_some();
    let filtered = app.filtered_transactions();

    let table_area = if filtering {
        let split = Layout::vertical([Constraint::Length(1), Constraint::Min(3)]).split(area);
        let query = app.transactions_filter.as_deref().unwrap_or("");
        let bar = Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(query, accent_style()),
            Span::styled("_", dim_style()),
        ]);
        frame.render_widget(Paragraph::new(bar), split[0]);
        split[1]
    } else {
        area
    };

    let header = Row::new(vec![
        Cell::from("Digest").style(header_style()),
        Cell::from("Status").style(header_style()),
        Cell::from("Gas").style(header_style()),
        Cell::from("Epoch").style(header_style()),
        Cell::from("").style(header_style()),
    ])
    .height(1);

    let visible_rows = table_area.height.saturating_sub(4) as usize;
    App::scroll_into_view(
        app.transactions_selected,
        &mut app.transactions_offset,
        visible_rows,
    );

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .skip(app.transactions_offset)
        .take(visible_rows)
        .map(|(display_idx, &real_idx)| {
            let tx = &app.transactions[real_idx];
            let style = if display_idx == app.transactions_selected {
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

    let display_len = filtered.len();
    let title = if filtering {
        format!(
            " Transactions ({}/{}) ",
            display_len,
            app.transactions.len()
        )
    } else if app.transactions.is_empty() {
        " Transactions ".to_string()
    } else {
        format!(" Transactions ({}) ", display_len)
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

    frame.render_widget(table, table_area);
}
