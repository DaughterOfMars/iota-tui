//! Activity Feed screen — live feed of network transactions and on-chain events.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common::{
    accent_style, detail_line, dim_style, header_style, selected_style, sparkle_text,
    truncate_address,
};
use crate::app::{ActivityKind, App};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let filtered = app.filtered_feed();
    let filtering = app.feed_filter.is_some();

    // Filter bar
    let content_area = if filtering {
        let split = Layout::vertical([Constraint::Length(1), Constraint::Min(3)]).split(area);
        let query = app.feed_filter.as_deref().unwrap_or("");
        let bar = Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(query, accent_style()),
            Span::styled("_", dim_style()),
        ]);
        frame.render_widget(Paragraph::new(bar), split[0]);
        split[1]
    } else {
        area
    };

    if app.activity_feed.is_empty() {
        let block = Block::default()
            .title(sparkle_text(" Activity Feed "))
            .title_style(header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(dim_style());

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No activity yet. Recent transactions and events will load automatically.",
                dim_style(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Refreshes while on this page.  /=filter  m=mode (Txns/Events)  Enter=explore",
                dim_style(),
            )]),
        ];
        frame.render_widget(Paragraph::new(text).block(block), content_area);
        return;
    }

    // Split into table + detail pane
    let layout = Layout::vertical([Constraint::Min(8), Constraint::Length(7)]).split(content_area);
    let table_area = layout[0];
    let detail_area = layout[1];

    draw_table(frame, app, table_area, &filtered, filtering);
    draw_detail(frame, app, detail_area, &filtered);
}

fn draw_table(
    frame: &mut Frame,
    app: &mut App,
    table_area: Rect,
    filtered: &[usize],
    filtering: bool,
) {
    let is_events = app.feed_mode == crate::app::FeedMode::Events;

    let header = if is_events {
        Row::new(vec![
            Cell::from("Time").style(header_style()),
            Cell::from("Event").style(header_style()),
            Cell::from("Sender").style(header_style()),
        ])
    } else {
        Row::new(vec![
            Cell::from("Time").style(header_style()),
            Cell::from("Digest").style(header_style()),
            Cell::from("Status").style(header_style()),
        ])
    }
    .height(1);

    let visible_rows = table_area.height.saturating_sub(4) as usize;
    App::scroll_into_view(app.feed_selected, &mut app.feed_offset, visible_rows);

    // Available width inside borders, minus the time column and padding
    let inner_w = table_area.width.saturating_sub(2) as usize;
    let time_w = 10;

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .skip(app.feed_offset)
        .take(visible_rows)
        .map(|(vi, &real_idx)| {
            let event = &app.activity_feed[real_idx];
            let style = if vi == app.feed_selected {
                selected_style()
            } else {
                Style::default()
            };
            let cells = if is_events {
                // Sender gets whatever space remains after time + event name
                let sender_budget = inner_w
                    .saturating_sub(time_w)
                    .saturating_sub(event.summary.len())
                    .saturating_sub(6); // column gaps
                let sender_display =
                    if event.sender.len() <= sender_budget || sender_budget >= event.sender.len() {
                        event.sender.clone()
                    } else {
                        truncate_address(&event.sender, sender_budget.max(12))
                    };
                vec![
                    Cell::from(event.timestamp.clone()),
                    Cell::from(event.summary.clone()),
                    Cell::from(sender_display),
                ]
            } else {
                // Digest gets whatever space remains after time + status
                let digest_budget = inner_w
                    .saturating_sub(time_w)
                    .saturating_sub(event.summary.len())
                    .saturating_sub(6);
                let digest_display = if event.digest.len() <= digest_budget {
                    event.digest.clone()
                } else {
                    truncate_address(&event.digest, digest_budget.max(12))
                };
                vec![
                    Cell::from(event.timestamp.clone()),
                    Cell::from(digest_display),
                    Cell::from(event.summary.clone()),
                ]
            };
            Row::new(cells).style(style)
        })
        .collect();

    // Compute max width of the middle column from visible data
    let mid_max: u16 = filtered
        .iter()
        .skip(app.feed_offset)
        .take(visible_rows)
        .map(|&i| {
            let e = &app.activity_feed[i];
            if is_events {
                e.summary.len()
            } else {
                e.digest.len()
            }
        })
        .max()
        .unwrap_or(10) as u16;

    let widths = if is_events {
        [
            Constraint::Length(time_w as u16),
            Constraint::Length(mid_max.max(6)),
            Constraint::Min(12),
        ]
    } else {
        [
            Constraint::Length(time_w as u16),
            Constraint::Min(mid_max.max(12)),
            Constraint::Min(10),
        ]
    };

    let mode_label = app.feed_mode.label();
    let count = filtered.len();
    let title = if filtering {
        format!(
            " Activity [{mode_label}] ({count}/{}) ",
            app.activity_feed.len()
        )
    } else if count > visible_rows {
        format!(
            " Activity [{mode_label}] ({count}) [{}-{}/{}] ",
            app.feed_offset + 1,
            (app.feed_offset + visible_rows).min(count),
            count,
        )
    } else {
        format!(" Activity [{mode_label}] ({count}) ")
    };

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(sparkle_text(&title))
            .title_style(header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(dim_style()),
    );

    frame.render_widget(table, table_area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect, filtered: &[usize]) {
    let block = Block::default()
        .title(sparkle_text(" Details "))
        .title_style(header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(dim_style());

    let real_idx = filtered.get(app.feed_selected).copied();
    let content = if let Some(event) = real_idx.and_then(|i| app.activity_feed.get(i)) {
        let id_width = area.width.saturating_sub(16) as usize;
        let mut lines = vec![
            detail_line("Time", &event.timestamp, Style::default()),
            detail_line(
                if event.kind == ActivityKind::Event {
                    "Type"
                } else {
                    "Digest"
                },
                &truncate_address(&event.digest, id_width),
                accent_style(),
            ),
            detail_line("Summary", &event.summary, Style::default()),
        ];
        if !event.gas_used.is_empty() {
            lines.push(detail_line("Gas", &event.gas_used, dim_style()));
        }
        if !event.sender.is_empty() {
            lines.push(detail_line(
                "Sender",
                &truncate_address(&event.sender, id_width),
                accent_style(),
            ));
        }
        lines
    } else {
        vec![Line::from("  No item selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
