use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

use super::common;
use crate::app::{App, InputMode, TxBuilderStep};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([
        Constraint::Length(3), // step indicator
        Constraint::Min(10),   // step content
    ])
    .split(area);

    draw_step_indicator(frame, app, layout[0]);

    match app.tx_step {
        TxBuilderStep::SelectSender => draw_select_sender(frame, app, layout[1]),
        TxBuilderStep::AddRecipients => draw_recipients(frame, app, layout[1]),
        TxBuilderStep::SetGas => draw_gas(frame, app, layout[1]),
        TxBuilderStep::Review => draw_review(frame, app, layout[1]),
    }
}

fn draw_step_indicator(frame: &mut Frame, app: &App, area: Rect) {
    let steps: Vec<Span> = TxBuilderStep::ALL
        .iter()
        .enumerate()
        .flat_map(|(i, step)| {
            let is_current = *step == app.tx_step;
            let num = format!(" {} ", i + 1);
            let title = format!(" {} ", step.title());

            let num_style = if is_current {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let title_style = if is_current {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut spans = vec![
                Span::styled(num, num_style),
                Span::styled(title, title_style),
            ];

            if i < TxBuilderStep::ALL.len() - 1 {
                spans.push(Span::styled(" > ", Style::default().fg(Color::DarkGray)));
            }

            spans
        })
        .collect();

    let block = Block::default()
        .title(" Transaction Builder ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    frame.render_widget(Paragraph::new(Line::from(steps)).block(block), area);
}

fn draw_select_sender(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Select Sender ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let items: Vec<ListItem> = app
        .keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let is_selected = i == app.tx_sender;
            let marker = if is_selected { "> " } else { "  " };
            let active = if key.is_active { " (active)" } else { "" };
            let addr_display = common::truncate_address(&key.address, 30);

            let style = if is_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::raw(marker),
                Span::styled(&key.alias, Style::default().fg(Color::White).bold()),
                Span::styled(active, Style::default().fg(Color::Green)),
                Span::styled(format!("  {}", addr_display), common::dim_style()),
                Span::styled(format!("  [{}]", key.scheme), common::dim_style()),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_recipients(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Recipients (a: add, d: delete) ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    if app.tx_recipients.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No recipients added yet.",
                common::dim_style(),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", common::dim_style()),
                Span::styled("'a'", common::accent_style()),
                Span::styled(" to add a recipient.", common::dim_style()),
            ]),
        ];
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header = Row::new(vec!["#", "Address", "Amount (IOTA)"])
        .style(common::header_style())
        .bottom_margin(1);

    let addr_width = area.width.saturating_sub(30) as usize;

    let rows: Vec<Row> = app
        .tx_recipients
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == app.tx_recipient_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            Row::new(vec![
                format!("{}", i + 1),
                common::truncate_address(&r.address, addr_width.max(20)),
                r.amount.clone(),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Min(24),
        Constraint::Length(16),
    ];

    let table = Table::new(rows, widths).header(header).block(block);
    frame.render_widget(table, area);
}

fn draw_gas(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Gas Budget ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let display = if app.input_mode == InputMode::Editing {
        format!("{}|", app.input_buffer)
    } else {
        app.tx_gas_budget.clone()
    };

    let edit_hint = if app.input_mode == InputMode::Editing {
        "Enter: confirm  Esc: cancel"
    } else {
        "Press Enter or 'e' to edit"
    };

    let budget_style = if app.input_mode == InputMode::Editing {
        Style::default().fg(Color::White).underlined()
    } else {
        Style::default().fg(Color::Cyan).bold()
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Gas Budget (MIST): ",
                Style::default().fg(Color::White).bold(),
            ),
            Span::styled(&display, budget_style),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  Approx: {} IOTA", parse_gas_iota(&app.tx_gas_budget)),
            common::dim_style(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}", edit_hint),
            common::dim_style(),
        )]),
    ];

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_review(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Review Transaction ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let sender = app
        .keys
        .get(app.tx_sender)
        .map(|k| {
            format!(
                "{} ({})",
                common::truncate_address(&k.address, 30),
                k.alias.as_str()
            )
        })
        .unwrap_or_else(|| "None".into());

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Sender:     ", Style::default().fg(Color::White).bold()),
            Span::styled(sender, common::accent_style()),
        ]),
        Line::from(vec![
            Span::styled("  Gas Budget: ", Style::default().fg(Color::White).bold()),
            Span::raw(format!(
                "{} MIST ({} IOTA)",
                app.tx_gas_budget,
                parse_gas_iota(&app.tx_gas_budget)
            )),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  Recipients ({}):", app.tx_recipients.len()),
            Style::default().fg(Color::White).bold(),
        )]),
    ];

    if app.tx_recipients.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "    (none)",
            Style::default().fg(Color::Red),
        )]));
    } else {
        for (i, r) in app.tx_recipients.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("    {}. ", i + 1), common::dim_style()),
                Span::raw(common::truncate_address(&r.address, 30)),
                Span::styled(format!("  {} IOTA", r.amount), common::accent_style()),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press Enter to sign and submit",
        Style::default().fg(Color::Green).bold(),
    )]));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn parse_gas_iota(mist: &str) -> String {
    mist.parse::<f64>()
        .map(|v| format!("{:.6}", v / 1_000_000_000.0))
        .unwrap_or_else(|_| "?".into())
}
