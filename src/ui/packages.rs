//! Packages screen — browse published Move packages.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let packages = app.package_indices();

    if packages.is_empty() {
        let block = Block::default()
            .title(" Packages ")
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style());

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No published packages found for this address.",
                common::dim_style(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Package objects will appear here once you publish Move packages.",
                common::dim_style(),
            )]),
            Line::from(vec![Span::styled(
                "  Alternatively, browse all objects in the Objects tab.",
                common::dim_style(),
            )]),
        ];
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let visible_rows = area.height.saturating_sub(4) as usize;

    let header = Row::new(vec!["Object ID", "Version", ""])
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = packages
        .iter()
        .enumerate()
        .skip(app.packages_offset)
        .take(visible_rows)
        .map(|(i, &obj_idx)| {
            let obj = &app.objects[obj_idx];
            let style = if i == app.packages_selected {
                common::selected_style()
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(common::truncate_address(&obj.object_id, 40)),
                Cell::from(obj.version.clone()),
                Cell::from("⏎").style(Style::default().fg(Color::Green)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(42),
        Constraint::Length(10),
        Constraint::Length(2),
    ];

    let title = if packages.len() > visible_rows {
        format!(
            " Packages ({}) [{}-{}/{}] ",
            packages.len(),
            app.packages_offset + 1,
            (app.packages_offset + visible_rows).min(packages.len()),
            packages.len()
        )
    } else {
        format!(" Packages ({}) ", packages.len())
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
