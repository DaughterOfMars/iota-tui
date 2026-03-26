//! Explorer view — renders lookup results (object/address/transaction details).

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common;
use crate::app::{App, LookupResult, LookupSection};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    if !app.explorer.search_results.is_empty() {
        draw_search_results(frame, app, area);
    } else if app.explorer.lookup_result.is_some() {
        draw_lookup_result(frame, app, area);
    } else {
        let block = Block::default()
            .title(common::sparkle_text(" Result "))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style());
        frame.render_widget(Paragraph::new("  Loading...").block(block), area);
    }
}

fn draw_search_results(frame: &mut Frame, app: &mut App, area: Rect) {
    let visible_rows = area.height.saturating_sub(4) as usize;
    app.explorer.visible_rows = visible_rows;
    App::scroll_into_view(
        app.explorer.search_selected,
        &mut app.explorer.search_offset,
        visible_rows,
    );
    let header = Row::new(vec!["Object ID", "Type", "Version", "Owner", ""])
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .explorer
        .search_results
        .iter()
        .enumerate()
        .skip(app.explorer.search_offset)
        .take(visible_rows)
        .map(|(i, obj)| {
            let style = if i == app.explorer.search_selected {
                common::selected_style()
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(common::truncate_address(&obj.object_id, 24)),
                Cell::from(common::truncate_type(&obj.type_name, 30)),
                Cell::from(obj.version.clone()),
                Cell::from(common::truncate_address(&obj.owner, 20)),
                Cell::from("⏎").style(Style::default().fg(Color::Green)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(26),
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(22),
        Constraint::Length(2),
    ];

    let page_num = app.explorer.search_cursors.len() + 1;
    let has_prev = !app.explorer.search_cursors.is_empty();
    let has_next = app.explorer.search_has_next;
    let page_hint = match (has_prev, has_next) {
        (true, true) => format!(" | pg {} [:prev ]:next", page_num),
        (true, false) => format!(" | pg {} [:prev", page_num),
        (false, true) => " | ]:next".to_string(),
        (false, false) => String::new(),
    };
    let title = format!(
        " Search Results ({}){} ",
        app.explorer.search_results.len(),
        page_hint,
    );

    let (table_area, pagination_area) = if has_prev || has_next {
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
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

    if let Some(pa) = pagination_area {
        app.explorer.pagination_row_y = pa.y;
        render_pagination_row(frame, pa, has_prev, has_next);
    } else {
        app.explorer.pagination_row_y = 0;
    }
}

fn draw_lookup_result(frame: &mut Frame, app: &mut App, area: Rect) {
    let addr_has_prev = !app.explorer.lookup_obj_cursors.is_empty();
    let addr_has_next = app.explorer.lookup_obj_has_next || app.explorer.lookup_tx_has_next;
    let addr_paginated = matches!(
        app.explorer.lookup_result,
        Some(LookupResult::Address { .. })
    ) && (addr_has_prev || addr_has_next);

    let (result_area, addr_pagination_area) = if addr_paginated {
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    let (sections, title_override) = match app.explorer.lookup_result.as_ref().unwrap() {
        LookupResult::NotFound(msg) => {
            let block = Block::default()
                .title(common::sparkle_text(" Result "))
                .title_style(common::header_style())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(common::dim_style());
            let content = vec![Line::from(Span::styled(
                format!("  {}", msg),
                Style::default().fg(Color::Yellow),
            ))];
            frame.render_widget(Paragraph::new(content).block(block), result_area);
            return;
        }
        LookupResult::Object { sections } | LookupResult::Transaction { sections } => {
            (sections.clone(), None)
        }
        LookupResult::Address { sections } => {
            let page_num = app.explorer.lookup_obj_page + 1;
            let page_hint = match (addr_has_prev, addr_has_next) {
                (true, true) => format!(" | pg {} [:prev ]:next", page_num),
                (true, false) => format!(" | pg {} [:prev", page_num),
                (false, true) => " | ]:next".to_string(),
                (false, false) => String::new(),
            };
            let title = if page_hint.is_empty() {
                None
            } else {
                Some(format!(" Result{} ", page_hint))
            };
            (sections.clone(), title)
        }
    };

    let visible = draw_lookup_sections(
        frame,
        app,
        &sections,
        title_override.as_deref(),
        result_area,
    );
    app.explorer.visible_rows = visible;

    if let Some(ref result) = app.explorer.lookup_result {
        result.scroll_cursor_into_view(
            app.explorer.lookup_section,
            app.explorer.lookup_depth,
            app.explorer.lookup_field_idx,
            &mut app.explorer.lookup_offset,
            visible,
        );
    }

    if let Some(pa) = addr_pagination_area {
        app.explorer.pagination_row_y = pa.y;
        render_pagination_row(frame, pa, addr_has_prev, addr_has_next);
    } else if app.explorer.search_results.is_empty() {
        app.explorer.pagination_row_y = 0;
    }
}

fn draw_lookup_sections(
    frame: &mut Frame,
    app: &App,
    sections: &[LookupSection],
    title: Option<&str>,
    area: Rect,
) -> usize {
    let block = Block::default()
        .title(common::sparkle_text(title.unwrap_or(" Result ")))
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(common::dim_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_rows = inner.height as usize;
    let mut lines: Vec<Line> = Vec::new();

    let cur_sec = app.explorer.lookup_section;
    let cur_depth = app.explorer.lookup_depth;
    let cur_field = app.explorer.lookup_field_idx;

    for (si, section) in sections.iter().enumerate() {
        let heading_selected = si == cur_sec && cur_depth == 0;
        let collapse_indicator = if section.collapsed { "▸" } else { "▾" };

        let heading_style = if heading_selected {
            common::selected_style()
        } else {
            Style::default().fg(Color::Cyan).bold()
        };
        lines.push(Line::from(Span::styled(
            format!(
                "{} {} ({})",
                collapse_indicator,
                section.title,
                section.fields.len()
            ),
            heading_style,
        )));

        if !section.collapsed {
            for (fi, field) in section.fields.iter().enumerate() {
                let is_selected = si == cur_sec && cur_depth == 1 && fi == cur_field;
                let has_action = field.action.is_some();

                let key_style = if is_selected {
                    common::selected_style()
                } else {
                    Style::default().fg(Color::White).bold()
                };
                let val_style = if is_selected {
                    common::selected_style()
                } else if has_action {
                    common::accent_style()
                } else {
                    Style::default().fg(Color::Gray)
                };

                let nav_hint = if is_selected && has_action {
                    " ⏎"
                } else {
                    ""
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<20}", field.key), key_style),
                    Span::styled(
                        common::truncate_address(
                            &field.value,
                            inner.width.saturating_sub(24) as usize,
                        ),
                        val_style,
                    ),
                    Span::styled(nav_hint.to_string(), Style::default().fg(Color::Green)),
                ]));
            }
        }
    }

    let line_offset = app.explorer.lookup_offset;

    let display_lines: Vec<Line> = lines
        .into_iter()
        .skip(line_offset)
        .take(visible_rows)
        .collect();

    frame.render_widget(Paragraph::new(display_lines), inner);

    visible_rows
}

fn render_pagination_row(frame: &mut Frame, area: Rect, has_prev: bool, has_next: bool) {
    let mut spans = vec![Span::raw("  ")];
    if has_prev {
        spans.push(Span::styled(
            "[ \u{25C0} Prev ]",
            Style::default().fg(common::color_at(0)).bold(),
        ));
        spans.push(Span::raw("  "));
    }
    if has_next {
        spans.push(Span::styled(
            "[ Next \u{25B6} ]",
            Style::default().fg(common::color_at(0)).bold(),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
