//! Explorer screen — browse network state: overview, checkpoints, validators, and lookup.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use super::common;
use crate::app::{App, ExplorerView, InputMode, LookupResult, LookupSection};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([
        Constraint::Length(3), // sub-view tabs
        Constraint::Min(5),    // content
    ])
    .split(area);

    draw_sub_tabs(frame, app, layout[0]);

    match app.explorer_view {
        ExplorerView::Overview => draw_overview(frame, app, layout[1]),
        ExplorerView::Checkpoints => draw_checkpoints(frame, app, layout[1]),
        ExplorerView::Validators => draw_validators(frame, app, layout[1]),
        ExplorerView::Lookup => draw_lookup(frame, app, layout[1]),
    }
}

fn draw_sub_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let tabs: Vec<Span> = ExplorerView::ALL
        .iter()
        .flat_map(|view| {
            let label = format!(" {} ", view.title());
            let style = if *view == app.explorer_view {
                Style::default().fg(Color::Black).bg(common::ACCENT).bold()
            } else {
                Style::default().fg(Color::White).dim()
            };
            vec![Span::styled(label, style), Span::raw(" ")]
        })
        .collect();

    let block = Block::default()
        .title(" Explorer ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let line = Line::from(tabs);
    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Network Overview ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let content = if let Some(ref overview) = app.explorer_overview {
        vec![
            kv_line("  Chain ID", &overview.chain_id),
            kv_line("  Current Epoch", &overview.epoch),
            kv_line("  Gas Price", &overview.gas_price),
            kv_line("  Latest Checkpoint", &overview.latest_checkpoint),
            kv_line("  Total Transactions", &overview.total_txs),
        ]
    } else {
        vec![Line::from("  Loading...")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}

fn draw_checkpoints(frame: &mut Frame, app: &App, area: Rect) {
    if app.explorer_checkpoints.is_empty() {
        let block = Block::default()
            .title(" Checkpoints ")
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_style(common::dim_style());
        frame.render_widget(
            Paragraph::new("  Loading checkpoints...").block(block),
            area,
        );
        return;
    }

    let filtered = app.filtered_checkpoints();

    // If filtering, show a search bar row at the top
    let (table_area, filter_area) = if app.explorer_checkpoints_filter.is_some() {
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);
        (chunks[1], Some(chunks[0]))
    } else {
        (area, None)
    };

    if let Some(fa) = filter_area {
        let q = app.explorer_checkpoints_filter.as_deref().unwrap_or("");
        let line = Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(common::ACCENT).bold()),
            Span::styled(q, Style::default().fg(Color::White)),
            Span::styled("_", Style::default().fg(common::DIM)),
        ]);
        frame.render_widget(Paragraph::new(line), fa);
    }

    let visible_rows = table_area.height.saturating_sub(4) as usize;

    let sort_indicator = if app.explorer_checkpoints_sort_asc {
        " ^"
    } else {
        " v"
    };
    let header = Row::new(vec![
        format!("Sequence{}", sort_indicator),
        "Digest".to_string(),
        "Timestamp".to_string(),
        "TXs".to_string(),
    ])
    .style(common::header_style())
    .bottom_margin(1);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .skip(app.explorer_checkpoints_offset)
        .take(visible_rows)
        .map(|(vi, &ci)| {
            let cp = &app.explorer_checkpoints[ci];
            let style = if vi == app.explorer_checkpoints_selected {
                common::selected_style()
            } else {
                Style::default()
            };
            Row::new(vec![
                cp.sequence.to_string(),
                common::truncate_address(&cp.digest, 20),
                cp.timestamp.clone(),
                cp.tx_count.to_string(),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(14),
        Constraint::Length(22),
        Constraint::Min(20),
        Constraint::Length(10),
    ];

    let page_num = app.explorer_checkpoints_page + 1;
    let has_prev = !app.explorer_checkpoints_cursors.is_empty();
    let has_next = app.explorer_checkpoints_has_next;
    let page_hint = match (has_prev, has_next) {
        (true, true) => format!(" | pg {} [:prev ]:next", page_num),
        (true, false) => format!(" | pg {} [:prev", page_num),
        (false, true) => " | ]:next".to_string(),
        (false, false) => String::new(),
    };
    let sort_hint = " s:sort";
    let search_hint = if app.explorer_checkpoints_filter.is_some() {
        " Esc:clear"
    } else {
        " /:search"
    };
    let title = if app.explorer_checkpoints_filter.is_some() {
        format!(
            " Checkpoints ({}/{}){}{}{} ",
            filtered.len(),
            app.explorer_checkpoints.len(),
            page_hint,
            sort_hint,
            search_hint,
        )
    } else {
        format!(
            " Checkpoints ({}){}{}{} ",
            app.explorer_checkpoints.len(),
            page_hint,
            sort_hint,
            search_hint,
        )
    };
    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(title)
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, table_area);
}

fn draw_validators(frame: &mut Frame, app: &App, area: Rect) {
    if app.explorer_validators.is_empty() {
        let block = Block::default()
            .title(" Validators ")
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_style(common::dim_style());
        frame.render_widget(Paragraph::new("  Loading validators...").block(block), area);
        return;
    }

    let visible_rows = area.height.saturating_sub(4) as usize;

    let header = Row::new(vec!["Name", "Address", "Voting Power"])
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .explorer_validators
        .iter()
        .enumerate()
        .skip(app.explorer_validators_offset)
        .take(visible_rows)
        .map(|(i, v)| {
            let style = if i == app.explorer_validators_selected {
                common::selected_style()
            } else {
                Style::default()
            };
            Row::new(vec![
                v.name.clone(),
                common::truncate_address(&v.address, 24),
                v.stake.clone(),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Length(26),
        Constraint::Length(14),
    ];

    let title = format!(" Validators ({}) ", app.explorer_validators.len());
    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(title)
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, area);
}

fn draw_lookup(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([
        Constraint::Length(3), // search input
        Constraint::Min(5),    // result
    ])
    .split(area);

    // Search input
    let editing = app.input_mode == InputMode::Editing;

    let input_title = if editing {
        if app.explorer_search_mode {
            " Type Search (Enter:submit  Esc:cancel) "
        } else {
            " Lookup (Enter:submit  Esc:cancel) "
        }
    } else {
        " Search "
    };

    let input_block = Block::default()
        .title(input_title)
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(if editing {
            Style::default().fg(Color::Green)
        } else {
            common::dim_style()
        });

    let input_line: Line = if editing {
        let (before, after) = app.input_buffer.split_at(app.input_cursor);
        Line::from(vec![
            Span::raw("  "),
            Span::raw(before.to_string()),
            Span::styled("|", Style::default().fg(Color::Green).bold()),
            Span::raw(after.to_string()),
        ])
    } else {
        Line::from(Span::styled(
            "  Enter:lookup  s:type-search",
            common::dim_style(),
        ))
    };

    frame.render_widget(Paragraph::new(input_line).block(input_block), layout[0]);

    // Result area
    let result_block = Block::default()
        .title(" Result ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    if !app.explorer_search_results.is_empty() {
        // Show type-search results as a table
        let visible_rows = layout[1].height.saturating_sub(4) as usize;
        let header = Row::new(vec!["Object ID", "Type", "Version", "Owner"])
            .style(common::header_style())
            .bottom_margin(1);

        let rows: Vec<Row> = app
            .explorer_search_results
            .iter()
            .enumerate()
            .skip(app.explorer_search_offset)
            .take(visible_rows)
            .map(|(i, obj)| {
                let style = if i == app.explorer_search_selected {
                    common::selected_style()
                } else {
                    Style::default()
                };
                Row::new(vec![
                    common::truncate_address(&obj.object_id, 24),
                    common::truncate_type(&obj.type_name, 30),
                    obj.version.clone(),
                    common::truncate_address(&obj.owner, 20),
                ])
                .style(style)
            })
            .collect();

        let widths = [
            Constraint::Length(26),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(22),
        ];

        let page_num = app.explorer_search_cursors.len() + 1;
        let has_prev = !app.explorer_search_cursors.is_empty();
        let has_next = app.explorer_search_has_next;
        let page_hint = match (has_prev, has_next) {
            (true, true) => format!(" | pg {} [:prev ]:next", page_num),
            (true, false) => format!(" | pg {} [:prev", page_num),
            (false, true) => " | ]:next".to_string(),
            (false, false) => String::new(),
        };
        let title = format!(
            " Search Results ({}){}",
            app.explorer_search_results.len(),
            page_hint,
        );
        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .title(title)
                .title_style(common::header_style())
                .borders(Borders::ALL)
                .border_style(common::dim_style()),
        );

        frame.render_widget(table, layout[1]);
    } else if let Some(ref result) = app.explorer_lookup_result {
        match result {
            LookupResult::NotFound(msg) => {
                let content = vec![Line::from(Span::styled(
                    format!("  {}", msg),
                    Style::default().fg(Color::Yellow),
                ))];
                frame.render_widget(Paragraph::new(content).block(result_block), layout[1]);
            }
            LookupResult::Object { sections } | LookupResult::Transaction { sections } => {
                draw_lookup_sections(frame, app, sections, None, layout[1]);
            }
            LookupResult::Address { sections } => {
                let page_num = app.explorer_lookup_obj_page + 1;
                let has_prev = !app.explorer_lookup_obj_cursors.is_empty();
                let has_next = app.explorer_lookup_obj_has_next || app.explorer_lookup_tx_has_next;
                let page_hint = match (has_prev, has_next) {
                    (true, true) => format!(" | pg {} [:prev ]:next", page_num),
                    (true, false) => format!(" | pg {} [:prev", page_num),
                    (false, true) => " | ]:next".to_string(),
                    (false, false) => String::new(),
                };
                let title_override = if page_hint.is_empty() {
                    None
                } else {
                    Some(format!(" Result{} ", page_hint))
                };
                draw_lookup_sections(frame, app, sections, title_override.as_deref(), layout[1]);
            }
        }
    } else {
        frame.render_widget(
            Paragraph::new("  Enter a hex address, object ID, or transaction digest")
                .block(result_block),
            layout[1],
        );
    }
}

fn draw_lookup_sections(
    frame: &mut Frame,
    app: &App,
    sections: &[LookupSection],
    title: Option<&str>,
    area: Rect,
) {
    let block = Block::default()
        .title(title.unwrap_or(" Result "))
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_rows = inner.height as usize;
    let mut flat_idx: usize = 0;
    let mut lines: Vec<Line> = Vec::new();

    for section in sections {
        // Section header
        lines.push(Line::from(Span::styled(
            format!("── {} ──", section.title),
            Style::default().fg(Color::Cyan).bold(),
        )));

        for field in &section.fields {
            let is_selected = flat_idx == app.explorer_lookup_selected;
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
                    common::truncate_address(&field.value, inner.width.saturating_sub(24) as usize),
                    val_style,
                ),
                Span::styled(nav_hint.to_string(), Style::default().fg(Color::Green)),
            ]));

            flat_idx += 1;
        }
    }

    // Apply scroll offset
    let display_lines: Vec<Line> = lines
        .into_iter()
        .skip(app.explorer_lookup_offset)
        .take(visible_rows)
        .collect();

    frame.render_widget(Paragraph::new(display_lines), inner);
}

fn kv_line(key: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:<24}", key),
            Style::default().fg(Color::White).bold(),
        ),
        Span::styled(value.to_string(), common::accent_style()),
    ])
}
