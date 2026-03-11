use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
use super::common;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([
        Constraint::Min(8),
        Constraint::Length(6),
    ])
    .split(area);

    draw_package_list(frame, app, layout[0]);
    draw_detail(frame, app, layout[1]);
}

fn draw_package_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .packages
        .iter()
        .enumerate()
        .flat_map(|(i, pkg)| {
            let is_selected = i == app.packages_selected;
            let is_expanded = app.packages_expanded == Some(i);

            let indicator = if is_expanded { "[-]" } else { "[+]" };
            let style = if is_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            let id_display = common::truncate_address(&pkg.package_id, 24);

            let mut items = vec![ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", indicator), common::accent_style()),
                Span::styled(&pkg.name, Style::default().fg(Color::White).bold()),
                Span::styled(format!("  v{}", pkg.version), common::dim_style()),
                Span::styled(format!("  {}", id_display), common::dim_style()),
            ]))
            .style(style)];

            if is_expanded {
                for module in &pkg.modules {
                    items.push(
                        ListItem::new(Line::from(vec![
                            Span::raw("     "),
                            Span::styled("module ", common::dim_style()),
                            Span::styled(module.as_str(), common::accent_style()),
                        ]))
                    );
                }
            }

            items
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Packages ({}) ", app.packages.len()))
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_style(common::dim_style()),
    );

    frame.render_widget(list, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Package Details ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let content = if let Some(pkg) = app.packages.get(app.packages_selected) {
        let id_width = area.width.saturating_sub(16) as usize;
        vec![
            Line::from(vec![
                Span::styled("  Package:  ", Style::default().fg(Color::White).bold()),
                Span::styled(&pkg.name, common::accent_style()),
                Span::styled(format!("  (v{})", pkg.version), common::dim_style()),
            ]),
            Line::from(vec![
                Span::styled("  ID:       ", Style::default().fg(Color::White).bold()),
                Span::styled(
                    common::truncate_address(&pkg.package_id, id_width),
                    common::dim_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Modules:  ", Style::default().fg(Color::White).bold()),
                Span::raw(pkg.modules.join(", ")),
            ]),
        ]
    } else {
        vec![Line::from("  No package selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
