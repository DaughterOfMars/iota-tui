//! Staking screen — displays staked IOTA objects with validator info.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common::{
    accent_style, detail_line, dim_style, header_style, selected_style, sparkle_text,
    truncate_address,
};
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = Layout::vertical([Constraint::Min(8), Constraint::Length(7)]).split(area);
    let table_area = layout[0];
    let detail_area = layout[1];

    draw_table(frame, app, table_area);
    draw_detail(frame, app, detail_area);
}

fn draw_table(frame: &mut Frame, app: &mut App, table_area: Rect) {
    let header = Row::new(vec![
        Cell::from("Object ID").style(header_style()),
        Cell::from("Principal").style(header_style()),
        Cell::from("Validator / Pool").style(header_style()),
        Cell::from("Epoch").style(header_style()),
        Cell::from("Status").style(header_style()),
    ])
    .height(1);

    let visible_rows = table_area.height.saturating_sub(4) as usize;
    App::scroll_into_view(app.stakes_selected, &mut app.stakes_offset, visible_rows);

    let rows: Vec<Row> = app
        .stakes
        .iter()
        .enumerate()
        .skip(app.stakes_offset)
        .take(visible_rows)
        .map(|(idx, stake)| {
            let style = if idx == app.stakes_selected {
                selected_style()
            } else {
                Style::default()
            };
            let status_style = if stake.status == "Active" {
                accent_style()
            } else {
                Style::default().fg(Color::Yellow)
            };
            Row::new(vec![
                Cell::from(truncate_address(&stake.object_id, 24)),
                Cell::from(stake.principal_display.clone()),
                Cell::from(truncate_address(&stake.validator_address, 20)),
                Cell::from(stake.activation_epoch.clone()),
                Cell::from(stake.status.clone()).style(status_style),
            ])
            .style(style)
        })
        .collect();

    let title = if app.stakes.is_empty() {
        " Staking ".to_string()
    } else {
        format!(" Staking ({}) ", app.stakes.len())
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
        Constraint::Length(22),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths).header(header).block(block);
    frame.render_widget(table, table_area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(sparkle_text(" Details "))
        .title_style(header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(dim_style());

    let content = if let Some(stake) = app.stakes.get(app.stakes_selected) {
        let id_width = area.width.saturating_sub(16) as usize;
        vec![
            detail_line(
                "Object",
                &truncate_address(&stake.object_id, id_width),
                accent_style(),
            ),
            detail_line(
                "Principal",
                &format!("{} (raw: {})", stake.principal_display, stake.principal),
                Style::default().fg(Color::Green).bold(),
            ),
            detail_line("Validator", &stake.validator_address, Style::default()),
            detail_line("Activation Epoch", &stake.activation_epoch, dim_style()),
            detail_line("Status", &stake.status, accent_style()),
        ]
    } else {
        vec![Line::from(
            "  No staked objects found. Stake IOTA via Tx Builder (Stake command).",
        )]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
