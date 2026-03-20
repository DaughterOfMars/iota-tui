//! Activity Feed screen — live feed of new transaction events.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    if app.activity_feed.is_empty() {
        let block = Block::default()
            .title(common::sparkle_text(" Activity Feed "))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style());

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No activity yet. New transactions will appear here automatically.",
                common::dim_style(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  The feed polls for new transactions every 30 seconds.",
                common::dim_style(),
            )]),
        ];
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let visible_rows = area.height.saturating_sub(4) as usize;

    let header = Row::new(vec!["Time", "", "Summary", "Digest"])
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .activity_feed
        .iter()
        .enumerate()
        .skip(app.feed_offset)
        .take(visible_rows)
        .map(|(i, event)| {
            let style = if i == app.feed_selected {
                common::selected_style()
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(event.timestamp.clone()),
                Cell::from(event.kind.icon()),
                Cell::from(event.summary.clone()),
                Cell::from(common::truncate_address(&event.digest, 24)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(3),
        Constraint::Min(20),
        Constraint::Length(26),
    ];

    let title = if app.activity_feed.len() > visible_rows {
        format!(
            " Activity Feed ({}) [{}-{}/{}] ",
            app.activity_feed.len(),
            app.feed_offset + 1,
            (app.feed_offset + visible_rows).min(app.activity_feed.len()),
            app.activity_feed.len()
        )
    } else {
        format!(" Activity Feed ({}) ", app.activity_feed.len())
    };

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(title)
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, area);
}
