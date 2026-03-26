//! Grid view — adaptive box grid showing all address data at a glance.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
};

use super::common::{self, accent_style, dim_style, header_style};
use crate::app::{App, Section};

/// Draw the main grid view with search bar, boxes, and hint bar.
pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    // Layout: search bar (1) | grid content | hint bar (1)
    let layout = Layout::vertical([
        Constraint::Length(1), // search bar
        Constraint::Min(6),    // grid
        Constraint::Length(1), // hint bar
    ])
    .split(area);

    draw_search_bar(frame, app, layout[0]);
    draw_box_grid(frame, app, layout[1]);
    draw_hint_bar(frame, app, layout[2]);
}

/// Draw the search bar at the top.
fn draw_search_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let exploring_label = if let Some(ref query) = app.exploring {
        let truncated = if query.len() > 20 {
            format!("{}..{}", &query[..10], &query[query.len() - 8..])
        } else {
            query.clone()
        };
        format!("Exploring: {}  ", truncated)
    } else {
        String::new()
    };

    let mut spans = vec![];

    if !exploring_label.is_empty() {
        spans.push(Span::styled(
            &exploring_label,
            Style::default().fg(Color::Yellow).bold(),
        ));
        spans.push(Span::styled("[Esc] Back  ", dim_style()));
    }

    if app.search_focused {
        spans.push(Span::styled(
            " Search: ",
            Style::default().fg(Color::Green).bold(),
        ));
        spans.push(Span::styled(&app.search_buffer, accent_style()));
        spans.push(Span::styled("_", dim_style()));
    } else if exploring_label.is_empty() {
        spans.push(Span::styled(" Search ", dim_style()));
        spans.push(Span::styled(
            "(Ctrl+L or click)",
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Network + address on the right side
    let net_indicator = if app.loading {
        Span::styled(
            format!(" {} ... ", app.network_name),
            Style::default().fg(Color::Black).bg(Color::Yellow).bold(),
        )
    } else if app.connected {
        Span::styled(
            format!(" {} ", app.network_name),
            Style::default().fg(Color::Black).bg(Color::Green).bold(),
        )
    } else {
        Span::styled(
            " offline ",
            Style::default().fg(Color::Black).bg(Color::Red).bold(),
        )
    };

    let active_addr = app
        .active_key()
        .map(|k| {
            if k.address.len() > 14 {
                format!("{}..{}", &k.address[..8], &k.address[k.address.len() - 6..])
            } else {
                k.address.clone()
            }
        })
        .unwrap_or_default();

    let net_width = net_indicator.width() as u16;
    let right_width = net_width + active_addr.len() as u16 + 3;
    let cols =
        Layout::horizontal([Constraint::Min(10), Constraint::Length(right_width)]).split(area);

    // Store the search bar area as only the left column (excludes network tag)
    app.search_bar_area = cols[0];

    // Store the network tag area for click detection
    // Right-aligned: net_indicator starts at cols[1].x, spans net_width
    app.net_tag_area = Rect::new(cols[1].x, cols[1].y, net_width, 1);

    frame.render_widget(Paragraph::new(Line::from(spans)), cols[0]);

    let right_line = Line::from(vec![
        net_indicator,
        Span::raw(" "),
        Span::styled(active_addr, Style::default().fg(common::color_at(2))),
    ]);
    frame.render_widget(
        Paragraph::new(right_line).alignment(ratatui::layout::Alignment::Right),
        cols[1],
    );
}

/// Determine which sections are visible (have data).
fn visible_sections(app: &App) -> Vec<Section> {
    let mut sections = vec![];
    if !app.coins.is_empty() {
        sections.push(Section::Coins);
    }
    if !app.objects.is_empty() {
        sections.push(Section::Objects);
    }
    if !app.stakes.is_empty() {
        sections.push(Section::Staking);
    }
    if !app.transactions.is_empty() {
        sections.push(Section::Transactions);
    }
    if !app.package_indices().is_empty() {
        sections.push(Section::Packages);
    }
    sections
}

/// Draw the adaptive box grid.
fn draw_box_grid(frame: &mut Frame, app: &mut App, area: Rect) {
    app.grid_box_areas.clear();

    let sections = visible_sections(app);

    if sections.is_empty() {
        // No data yet — show a placeholder
        let msg = if app.keys.is_empty() {
            "No keys configured. Press , to open Settings and generate a key."
        } else if app.loading {
            "Loading..."
        } else {
            "No data available. Press r to refresh."
        };
        let text = Paragraph::new(Line::from(Span::styled(msg, dim_style())))
            .alignment(ratatui::layout::Alignment::Center);
        let centered_y = area.y + area.height / 2;
        frame.render_widget(text, Rect::new(area.x, centered_y, area.width, 1));
        return;
    }

    // Ensure focused section is valid
    if !sections.contains(&app.focused_section) {
        app.focused_section = sections[0];
    }

    // Adaptive grid layout based on terminal width and section count
    let cols = if area.width >= 120 { 3 } else { 2 };
    let rows_needed = sections.len().div_ceil(cols);

    // Compute row heights (equal distribution)
    let row_height = (area.height as usize / rows_needed).max(4) as u16;
    let row_constraints: Vec<Constraint> = (0..rows_needed)
        .map(|i| {
            if i == rows_needed - 1 {
                Constraint::Min(4) // last row gets remaining space
            } else {
                Constraint::Length(row_height)
            }
        })
        .collect();

    let row_areas = Layout::vertical(row_constraints).split(area);

    let mut section_idx = 0;
    for &row_area in row_areas.iter() {
        let sections_in_row = if section_idx + cols <= sections.len() {
            cols
        } else {
            sections.len() - section_idx
        };

        if sections_in_row == 0 {
            break;
        }

        let col_constraints: Vec<Constraint> = (0..sections_in_row)
            .map(|_| Constraint::Ratio(1, sections_in_row as u32))
            .collect();

        let col_areas = Layout::horizontal(col_constraints).split(row_area);

        for &col_area in col_areas.iter() {
            if section_idx >= sections.len() {
                break;
            }
            let section = sections[section_idx];
            app.grid_box_areas.push((section, col_area));
            draw_box(frame, app, section, col_area);
            section_idx += 1;
        }
    }
}

/// Draw a single box for a section.
fn draw_box(frame: &mut Frame, app: &App, section: Section, area: Rect) {
    let is_focused = section == app.focused_section;
    let count = section_item_count(app, section);

    let title = format!(" {} ({}) ", section.title(), count);

    let border_color = if is_focused {
        common::color_at(0)
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(title)
        .title_style(if is_focused {
            header_style()
        } else {
            dim_style()
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Render compact item list inside the box
    let visible_rows = inner.height as usize;
    let (selected, offset) = section_selection(app, section);

    match section {
        Section::Coins => draw_coins_compact(frame, app, inner, visible_rows, selected, offset),
        Section::Objects => draw_objects_compact(frame, app, inner, visible_rows, selected, offset),
        Section::Staking => draw_staking_compact(frame, app, inner, visible_rows, selected, offset),
        Section::Transactions => {
            draw_transactions_compact(frame, app, inner, visible_rows, selected, offset)
        }
        Section::Packages => {
            draw_packages_compact(frame, app, inner, visible_rows, selected, offset)
        }
    }

    // Draw scrollbar if content overflows
    if count > visible_rows {
        let mut scrollbar_state =
            ScrollbarState::new(count.saturating_sub(visible_rows)).position(offset);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_symbol("▐")
                .track_symbol(Some("│"))
                .thumb_style(Style::default().fg(if section == app.focused_section {
                    common::color_at(0)
                } else {
                    Color::DarkGray
                })),
            area,
            &mut scrollbar_state,
        );
    }
}

/// Get item count for a section.
fn section_item_count(app: &App, section: Section) -> usize {
    match section {
        Section::Coins => app.coins.len(),
        Section::Objects => app.objects.len(),
        Section::Staking => app.stakes.len(),
        Section::Transactions => app.transactions.len(),
        Section::Packages => app.package_indices().len(),
    }
}

/// Get the (selected, offset) pair for a section.
fn section_selection(app: &App, section: Section) -> (usize, usize) {
    match section {
        Section::Coins => (app.coins_selected, app.coins_offset),
        Section::Objects => (app.objects_selected, app.objects_offset),
        Section::Staking => (app.stakes_selected, app.stakes_offset),
        Section::Transactions => (app.transactions_selected, app.transactions_offset),
        Section::Packages => (app.packages_selected, app.packages_offset),
    }
}

// ── Compact renderers for each section ────────────────────────────

fn draw_coins_compact(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    visible_rows: usize,
    selected: usize,
    offset: usize,
) {
    let is_focused = app.focused_section == Section::Coins;
    for (i, idx) in (offset..app.coins.len().min(offset + visible_rows)).enumerate() {
        let coin = &app.coins[idx];
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let row_area = Rect::new(area.x, y, area.width, 1);
        let is_selected = is_focused && idx == selected;

        let symbol = format!("{:<6}", coin.symbol);
        let balance = format!("{:>12}", coin.balance_display);

        let style = if is_selected {
            common::selected_style()
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(format!(" {}", symbol), style.fg(Color::White)),
            Span::styled(balance, style.fg(Color::Green)),
        ]);
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

fn draw_objects_compact(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    visible_rows: usize,
    selected: usize,
    offset: usize,
) {
    let is_focused = app.focused_section == Section::Objects;
    for (i, idx) in (offset..app.objects.len().min(offset + visible_rows)).enumerate() {
        let obj = &app.objects[idx];
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let row_area = Rect::new(area.x, y, area.width, 1);
        let is_selected = is_focused && idx == selected;

        let type_short = common::short_type_name(&obj.type_name);
        let type_display =
            common::truncate_type(&type_short, (area.width as usize).saturating_sub(14));
        let id_short = if obj.object_id.len() > 10 {
            format!(
                "{}..{}",
                &obj.object_id[..6],
                &obj.object_id[obj.object_id.len() - 4..]
            )
        } else {
            obj.object_id.clone()
        };

        let style = if is_selected {
            common::selected_style()
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(
                format!(
                    " {:<width$}",
                    type_display,
                    width = (area.width as usize).saturating_sub(14)
                ),
                style.fg(Color::White),
            ),
            Span::styled(format!(" {}", id_short), style.fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

fn draw_staking_compact(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    visible_rows: usize,
    selected: usize,
    offset: usize,
) {
    let is_focused = app.focused_section == Section::Staking;
    for (i, idx) in (offset..app.stakes.len().min(offset + visible_rows)).enumerate() {
        let stake = &app.stakes[idx];
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let row_area = Rect::new(area.x, y, area.width, 1);
        let is_selected = is_focused && idx == selected;

        let status_color = if stake.status == "Active" {
            Color::Green
        } else {
            Color::Yellow
        };

        let style = if is_selected {
            common::selected_style()
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(
                format!(" {:>10}", stake.principal_display),
                style.fg(Color::Green),
            ),
            Span::styled(format!(" {:<8}", stake.status), style.fg(status_color)),
        ]);
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

fn draw_transactions_compact(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    visible_rows: usize,
    selected: usize,
    offset: usize,
) {
    let is_focused = app.focused_section == Section::Transactions;
    for (i, idx) in (offset..app.transactions.len().min(offset + visible_rows)).enumerate() {
        let tx = &app.transactions[idx];
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let row_area = Rect::new(area.x, y, area.width, 1);
        let is_selected = is_focused && idx == selected;

        let digest_short = if tx.digest.len() > 10 {
            format!("{}..{}", &tx.digest[..6], &tx.digest[tx.digest.len() - 4..])
        } else {
            tx.digest.clone()
        };

        let status_color = if tx.status.contains("Success") {
            Color::Green
        } else {
            Color::Red
        };

        let style = if is_selected {
            common::selected_style()
        } else {
            Style::default()
        };

        let kind_width = (area.width as usize).saturating_sub(22);
        let kind_display: String = tx.tx_kind.chars().take(kind_width).collect();

        let line = Line::from(vec![
            Span::styled(format!(" {}", digest_short), style.fg(Color::DarkGray)),
            Span::styled(
                format!(" {:<width$}", kind_display, width = kind_width),
                style.fg(Color::White),
            ),
            Span::styled(
                format!(
                    " {:>7}",
                    if tx.status.contains("Success") {
                        "OK"
                    } else {
                        "FAIL"
                    }
                ),
                style.fg(status_color),
            ),
        ]);
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

fn draw_packages_compact(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    visible_rows: usize,
    selected: usize,
    offset: usize,
) {
    let indices = app.package_indices();
    let is_focused = app.focused_section == Section::Packages;
    for (i, pkg_pos) in (offset..indices.len().min(offset + visible_rows)).enumerate() {
        let obj_idx = indices[pkg_pos];
        let obj = &app.objects[obj_idx];
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let row_area = Rect::new(area.x, y, area.width, 1);
        let is_selected = is_focused && pkg_pos == selected;

        let id_short = if obj.object_id.len() > 12 {
            format!(
                "{}..{}",
                &obj.object_id[..8],
                &obj.object_id[obj.object_id.len() - 4..]
            )
        } else {
            obj.object_id.clone()
        };

        let style = if is_selected {
            common::selected_style()
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(format!(" {}", id_short), style.fg(Color::White)),
            Span::styled(format!(" {}", obj.version), style.fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

/// Draw the bottom hint bar.
fn draw_hint_bar(frame: &mut Frame, _app: &App, area: Rect) {
    let hints = vec![
        ("←→", "navigate"),
        ("↑↓", "scroll"),
        ("Enter", "open"),
        ("/", "actions"),
        (",", "settings"),
        ("Ctrl+T", "build tx"),
        ("?", "help"),
    ];

    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    for (key, desc) in hints {
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default()
                .fg(Color::Black)
                .bg(common::color_at(0))
                .bold(),
        ));
        spans.push(Span::styled(format!(" {}  ", desc), dim_style()));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
