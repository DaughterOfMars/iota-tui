//! Packages screen — browse published Move packages, modules, and functions.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use super::common;
use crate::app::{App, PackageBrowserView};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    match app.pkg_view {
        PackageBrowserView::List => draw_list(frame, app, area),
        PackageBrowserView::Modules => draw_modules(frame, app, area),
        PackageBrowserView::Functions => draw_functions(frame, app, area),
    }
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let packages = app.package_indices();

    if packages.is_empty() {
        let block = Block::default()
            .title(common::sparkle_text(" Packages "))
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

fn draw_modules(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);

    // Breadcrumb
    let pkg_short = common::truncate_address(&app.pkg_selected_id, 20);
    let breadcrumb = Line::from(vec![
        Span::styled(" < ", Style::default().fg(common::color_at(0))),
        Span::styled(pkg_short, common::accent_style()),
        Span::styled(" > Modules", common::dim_style()),
    ]);
    frame.render_widget(Paragraph::new(breadcrumb), layout[0]);

    let modules = &app.pkg_modules;
    let visible_rows = layout[1].height.saturating_sub(4) as usize;

    let header = Row::new(vec!["Module", "Functions", "Structs"])
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = modules
        .iter()
        .enumerate()
        .skip(app.pkg_modules_offset)
        .take(visible_rows)
        .map(|(i, m)| {
            let style = if i == app.pkg_modules_selected {
                common::selected_style()
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(m.name.clone()),
                Cell::from(m.function_count.to_string()),
                Cell::from(m.struct_count.to_string()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let title = format!(" Modules ({}) ", modules.len());

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(title)
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, layout[1]);
}

fn draw_functions(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(8),
        Constraint::Length(7),
    ])
    .split(area);

    // Breadcrumb
    let pkg_short = common::truncate_address(&app.pkg_selected_id, 16);
    let breadcrumb = Line::from(vec![
        Span::styled(" < ", Style::default().fg(common::color_at(0))),
        Span::styled(pkg_short, common::accent_style()),
        Span::styled(" > ", common::dim_style()),
        Span::styled(&app.pkg_selected_module, common::accent_style()),
        Span::styled(" > Functions", common::dim_style()),
    ]);
    frame.render_widget(Paragraph::new(breadcrumb), layout[0]);

    let functions = &app.pkg_functions;
    let visible_rows = layout[1].height.saturating_sub(4) as usize;

    let header = Row::new(vec!["Function", "Visibility", "Entry", "Params", "Returns"])
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = functions
        .iter()
        .enumerate()
        .skip(app.pkg_functions_offset)
        .take(visible_rows)
        .map(|(i, f)| {
            let style = if i == app.pkg_functions_selected {
                common::selected_style()
            } else {
                Style::default()
            };
            let entry_str = if f.is_entry { "yes" } else { "" };
            let params = f.param_types.len().to_string();
            let returns = f.return_types.len().to_string();
            Row::new(vec![
                Cell::from(f.name.clone()),
                Cell::from(f.visibility.clone()),
                Cell::from(entry_str),
                Cell::from(params),
                Cell::from(returns),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(9),
    ];

    let title = format!(" Functions ({}) ", functions.len());

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(title)
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style()),
    );

    frame.render_widget(table, layout[1]);

    // Detail pane for selected function
    if let Some(func) = functions.get(app.pkg_functions_selected) {
        let params_str = if func.param_types.is_empty() {
            "none".to_string()
        } else {
            func.param_types.join(", ")
        };
        let returns_str = if func.return_types.is_empty() {
            "none".to_string()
        } else {
            func.return_types.join(", ")
        };
        let entry_label = if func.is_entry { "yes" } else { "no" };

        let lines = vec![
            common::detail_line("Name", &func.name, common::accent_style()),
            common::detail_line("Visibility", &func.visibility, Style::default()),
            common::detail_line("Entry", entry_label, Style::default()),
            common::detail_line(
                "Type Params",
                &func.type_param_count.to_string(),
                Style::default(),
            ),
            common::detail_line("Params", &params_str, Style::default()),
            common::detail_line("Returns", &returns_str, Style::default()),
        ];

        let block = Block::default()
            .title(" Detail ")
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style());

        frame.render_widget(Paragraph::new(lines).block(block), layout[2]);
    }
}
