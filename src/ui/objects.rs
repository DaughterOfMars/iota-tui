use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::App;
use super::common;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([
        Constraint::Min(8),   // object table
        Constraint::Length(7), // selected object detail
    ])
    .split(area);

    draw_object_table(frame, app, layout[0]);
    draw_detail(frame, app, layout[1]);
}

fn draw_object_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Object ID", "Type", "Version", "Digest"])
        .style(common::header_style())
        .bottom_margin(1);

    let max_type_width = area.width.saturating_sub(60) as usize;

    let rows: Vec<Row> = app
        .objects
        .iter()
        .enumerate()
        .map(|(i, obj)| {
            let style = if i == app.objects_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            let id_display = common::truncate_address(&obj.object_id, 20);
            let type_display = if obj.type_name.len() > max_type_width {
                format!("{}...", &obj.type_name[..max_type_width.saturating_sub(3)])
            } else {
                obj.type_name.clone()
            };
            let digest_display = common::truncate_address(&obj.digest, 16);

            Row::new(vec![
                id_display,
                type_display,
                format!("v{}", obj.version),
                digest_display,
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(22),
        Constraint::Min(20),
        Constraint::Length(8),
        Constraint::Length(18),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" Objects ({}) ", app.objects.len()))
                .title_style(common::header_style())
                .borders(Borders::ALL)
                .border_style(common::dim_style()),
        );

    frame.render_widget(table, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Object Details ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let content = if let Some(obj) = app.objects.get(app.objects_selected) {
        let id_width = area.width.saturating_sub(14) as usize;
        let type_width = area.width.saturating_sub(14) as usize;
        vec![
            Line::from(vec![
                Span::styled("  Object ID: ", Style::default().fg(Color::White).bold()),
                Span::styled(
                    common::truncate_address(&obj.object_id, id_width),
                    common::accent_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Type:      ", Style::default().fg(Color::White).bold()),
                Span::raw(if obj.type_name.len() > type_width {
                    format!("{}...", &obj.type_name[..type_width.saturating_sub(3)])
                } else {
                    obj.type_name.clone()
                }),
            ]),
            Line::from(vec![
                Span::styled("  Version:   ", Style::default().fg(Color::White).bold()),
                Span::raw(format!("v{}", obj.version)),
            ]),
            Line::from(vec![
                Span::styled("  Digest:    ", Style::default().fg(Color::White).bold()),
                Span::styled(&obj.digest, common::dim_style()),
            ]),
        ]
    } else {
        vec![Line::from("  No object selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
