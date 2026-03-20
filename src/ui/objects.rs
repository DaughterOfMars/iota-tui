//! Objects screen — displays owned objects across visible addresses.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([Constraint::Min(8), Constraint::Length(7)]).split(area);

    draw_object_table(frame, app, layout[0]);
    draw_detail(frame, app, layout[1]);
}

fn draw_object_table(frame: &mut Frame, app: &App, area: Rect) {
    let filtering = app.objects_filter.is_some();
    let filtered = app.filtered_objects();

    let (filter_area, table_area) = if filtering {
        let split = Layout::vertical([Constraint::Length(1), Constraint::Min(3)]).split(area);
        let query = app.objects_filter.as_deref().unwrap_or("");
        let bar = Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(query, common::accent_style()),
            Span::styled("_", common::dim_style()),
        ]);
        frame.render_widget(Paragraph::new(bar), split[0]);
        (Some(split[0]), split[1])
    } else {
        (None, area)
    };
    let _ = filter_area;

    if app.objects.is_empty() {
        let block = Block::default()
            .title(common::sparkle_text(" Objects "))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style());

        let msg = if app.keys.is_empty() {
            "  No keys configured."
        } else if !app.connected {
            "  Connecting..."
        } else {
            "  No objects found for this address."
        };

        frame.render_widget(Paragraph::new(msg).block(block), table_area);
        return;
    }

    let visible_rows = table_area.height.saturating_sub(4) as usize;

    let show_all = app.show_multiple_owners();

    let header_cols: Vec<&str> = if show_all {
        vec!["Object ID", "Type", "Version", "Digest", "Owner", ""]
    } else {
        vec!["Object ID", "Type", "Version", "Digest", ""]
    };
    let header = Row::new(header_cols)
        .style(common::header_style())
        .bottom_margin(1);

    let max_type_width = table_area
        .width
        .saturating_sub(if show_all { 74 } else { 60 }) as usize;

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .skip(app.objects_offset)
        .take(visible_rows)
        .map(|(display_idx, &real_idx)| {
            let obj = &app.objects[real_idx];
            let style = if display_idx == app.objects_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            let mut cells: Vec<Cell> = vec![
                Cell::from(common::truncate_address(&obj.object_id, 20)),
                Cell::from(common::truncate_type(&obj.type_name, max_type_width)),
                Cell::from(obj.version.clone()),
                Cell::from(common::truncate_address(&obj.digest, 16)),
            ];
            if show_all {
                cells.push(Cell::from(obj.owner_alias.clone()));
            }
            cells.push(Cell::from("⏎").style(Style::default().fg(Color::Green)));

            Row::new(cells).style(style)
        })
        .collect();

    let widths: Vec<Constraint> = if show_all {
        vec![
            Constraint::Length(22),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(14),
            Constraint::Length(2),
        ]
    } else {
        vec![
            Constraint::Length(22),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(2),
        ]
    };

    let display_len = filtered.len();
    let title = if filtering {
        format!(" Objects ({}/{}) ", display_len, app.objects.len())
    } else if display_len > visible_rows {
        format!(
            " Objects ({}) [{}-{}/{}] ",
            display_len,
            app.objects_offset + 1,
            (app.objects_offset + visible_rows).min(display_len),
            display_len
        )
    } else {
        format!(" Objects ({}) ", display_len)
    };

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(title)
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, table_area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(common::sparkle_text(" Object Details "))
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(common::dim_style());

    let content = if let Some(obj) = app.objects.get(app.objects_selected) {
        let id_width = area.width.saturating_sub(16) as usize;
        let type_width = area.width.saturating_sub(16) as usize;
        vec![
            common::detail_line(
                "Object ID",
                &common::truncate_address(&obj.object_id, id_width),
                common::accent_style(),
            ),
            common::detail_line(
                "Type",
                &common::truncate_type(&obj.type_name, type_width),
                Style::default(),
            ),
            common::detail_line("Version", &obj.version, Style::default()),
            common::detail_line("Digest", &obj.digest, common::dim_style()),
        ]
    } else {
        vec![Line::from("  No object selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
