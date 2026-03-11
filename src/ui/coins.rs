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
        Constraint::Length(3), // portfolio summary
        Constraint::Min(5),   // coin table
        Constraint::Length(5), // selected coin detail
    ])
    .split(area);

    draw_summary(frame, app, layout[0]);
    draw_coin_table(frame, app, layout[1]);
    draw_detail(frame, app, layout[2]);
}

fn draw_summary(frame: &mut Frame, app: &App, area: Rect) {
    let total = app.total_usd_value();
    let block = Block::default()
        .title(" Portfolio ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let text = Line::from(vec![
        Span::styled("  Total Value: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("${:.2}", total),
            Style::default().fg(Color::Green).bold(),
        ),
        Span::styled(
            format!("    {} coins", app.coins.len()),
            common::dim_style(),
        ),
    ]);

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_coin_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Coin", "Symbol", "Balance", "USD Value", "24h"])
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .coins
        .iter()
        .enumerate()
        .map(|(i, coin)| {
            let change_str = if coin.change_24h >= 0.0 {
                format!("+{:.2}%", coin.change_24h)
            } else {
                format!("{:.2}%", coin.change_24h)
            };

            let style = if i == app.coins_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            Row::new(vec![
                coin.name.clone(),
                coin.symbol.clone(),
                format!("{:.4}", coin.balance),
                format!("${:.2}", coin.usd_value),
                change_str,
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(" Coins ")
                .title_style(common::header_style())
                .borders(Borders::ALL)
                .border_style(common::dim_style()),
        );

    frame.render_widget(table, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_style(common::dim_style());

    let content = if let Some(coin) = app.coins.get(app.coins_selected) {
        let addr_display = common::truncate_address(&coin.object_id, area.width.saturating_sub(16) as usize);
        vec![
            Line::from(vec![
                Span::styled("  Name: ", Style::default().fg(Color::White).bold()),
                Span::raw(&coin.name),
                Span::styled("  |  Symbol: ", Style::default().fg(Color::White).bold()),
                Span::styled(&coin.symbol, common::accent_style()),
            ]),
            Line::from(vec![
                Span::styled("  Balance: ", Style::default().fg(Color::White).bold()),
                Span::raw(format!("{:.4} {}", coin.balance, coin.symbol)),
                Span::styled("  |  Value: ", Style::default().fg(Color::White).bold()),
                Span::styled(format!("${:.2}", coin.usd_value), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("  Object: ", Style::default().fg(Color::White).bold()),
                Span::styled(addr_display, common::dim_style()),
            ]),
        ]
    } else {
        vec![Line::from("  No coin selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}
