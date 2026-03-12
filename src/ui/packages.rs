use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::common;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Packages ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    // Filter objects that look like packages
    let package_objects: Vec<_> = app
        .objects
        .iter()
        .filter(|o| {
            o.type_name.contains("package") || o.type_name == "Package" || o.type_name.is_empty()
        })
        .collect();

    if package_objects.is_empty() {
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
    } else {
        let lines: Vec<Line> = package_objects
            .iter()
            .enumerate()
            .map(|(i, obj)| {
                Line::from(vec![
                    Span::styled(format!("  {}. ", i + 1), common::dim_style()),
                    Span::styled(
                        common::truncate_address(&obj.object_id, 40),
                        common::accent_style(),
                    ),
                    Span::styled(format!("  {}", obj.version), common::dim_style()),
                ])
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }
}
