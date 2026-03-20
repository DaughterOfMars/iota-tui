//! Keys screen — manage cryptographic keys with visibility and active status.

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
    let layout = Layout::vertical([Constraint::Min(6), Constraint::Length(7)]).split(area);

    draw_key_table(frame, app, layout[0]);
    draw_detail(frame, app, layout[1]);
}

fn draw_key_table(frame: &mut Frame, app: &App, area: Rect) {
    if app.keys.is_empty() {
        let block = Block::default()
            .title(common::sparkle_text(" Keys (0) "))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
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

    let header = Row::new(vec!["", "Show", "Alias", "Scheme", "Address", ""])
        .style(common::header_style())
        .bottom_margin(1);

    let addr_width = area.width.saturating_sub(46) as usize;

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
            let visible_marker = if key.visible { "[x]" } else { "[ ]" };

            Row::new(vec![
                Cell::from(active_marker.to_string()),
                Cell::from(visible_marker.to_string()),
                Cell::from(key.alias.clone()),
                Cell::from(key.scheme.clone()),
                Cell::from(common::truncate_address(&key.address, addr_width.max(20))),
                Cell::from("⏎").style(Style::default().fg(Color::Green)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Min(24),
        Constraint::Length(2),
    ];

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(common::sparkle_text(&format!(
                " Keys ({}) ",
                app.keys.len()
            )))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(common::sparkle_text(" Key Details "))
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(common::dim_style());

    let content = if let Some(key) = app.keys.get(app.keys_selected) {
        let addr_width = area.width.saturating_sub(16) as usize;
        let active_str = if key.is_active { "Yes (active)" } else { "No" };
        let active_style = if key.is_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
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
            common::detail_line("Private", &hex_display, Style::default().fg(Color::Red))
        } else {
            common::detail_line(
                "Private",
                "********** (press 'p' to reveal)",
                common::dim_style(),
            )
        };

        vec![
            common::detail_line("Alias", &key.alias, common::accent_style()),
            common::detail_line("Active", active_str, active_style),
            common::detail_line("Scheme", &key.scheme, Style::default()),
            common::detail_line(
                "Address",
                &common::truncate_address(&key.address, addr_width),
                Style::default(),
            ),
            private_line,
        ]
    } else {
        vec![Line::from("  No key selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
