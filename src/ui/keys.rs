use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use super::common;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([Constraint::Min(6), Constraint::Length(7)]).split(area);

    draw_key_table(frame, app, layout[0]);
    draw_detail(frame, app, layout[1]);
}

fn draw_key_table(frame: &mut Frame, app: &App, area: Rect) {
    if app.keys.is_empty() {
        let block = Block::default()
            .title(" Keys (0) ")
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_style(common::dim_style());

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No keys configured yet.",
                common::dim_style(),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", common::dim_style()),
                Span::styled("'g'", common::accent_style()),
                Span::styled(" to generate a new keypair, or ", common::dim_style()),
                Span::styled("'i'", common::accent_style()),
                Span::styled(" to import one.", common::dim_style()),
            ]),
        ];
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let visible_rows = area.height.saturating_sub(4) as usize;

    let header = Row::new(vec!["", "Alias", "Scheme", "Address"])
        .style(common::header_style())
        .bottom_margin(1);

    let addr_width = area.width.saturating_sub(40) as usize;

    let rows: Vec<Row> = app
        .keys
        .iter()
        .enumerate()
        .skip(app.keys_offset)
        .take(visible_rows)
        .map(|(i, key)| {
            let style = if i == app.keys_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            let active_marker = if key.is_active { "*" } else { " " };

            Row::new(vec![
                active_marker.to_string(),
                key.alias.clone(),
                key.scheme.clone(),
                common::truncate_address(&key.address, addr_width.max(20)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Min(24),
    ];

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(format!(" Keys ({}) ", app.keys.len()))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Key Details ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let content = if let Some(key) = app.keys.get(app.keys_selected) {
        let addr_width = area.width.saturating_sub(14) as usize;
        let active_str = if key.is_active { "Yes (active)" } else { "No" };
        let active_color = if key.is_active {
            Color::Green
        } else {
            Color::White
        };

        let private_line = if app.keys_show_private {
            let hex_display = if key.private_key_hex.len() > addr_width {
                format!(
                    "{}...{}",
                    &key.private_key_hex[..16],
                    &key.private_key_hex[key.private_key_hex.len().saturating_sub(8)..]
                )
            } else {
                key.private_key_hex.clone()
            };
            Line::from(vec![
                Span::styled("  Private: ", Style::default().fg(Color::White).bold()),
                Span::styled(hex_display, Style::default().fg(Color::Red)),
            ])
        } else {
            Line::from(vec![
                Span::styled("  Private: ", Style::default().fg(Color::White).bold()),
                Span::styled(
                    "********** (press 'p' to reveal)",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        };

        vec![
            Line::from(vec![
                Span::styled("  Alias:   ", Style::default().fg(Color::White).bold()),
                Span::styled(&key.alias, common::accent_style()),
                Span::styled("  |  Active: ", Style::default().fg(Color::White).bold()),
                Span::styled(active_str, Style::default().fg(active_color)),
            ]),
            Line::from(vec![
                Span::styled("  Scheme:  ", Style::default().fg(Color::White).bold()),
                Span::raw(&key.scheme),
            ]),
            Line::from(vec![
                Span::styled("  Address: ", Style::default().fg(Color::White).bold()),
                Span::raw(common::truncate_address(&key.address, addr_width)),
            ]),
            private_line,
        ]
    } else {
        vec![Line::from("  No key selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
