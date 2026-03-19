//! Coins screen — displays coin balances across visible addresses.

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
    let layout = Layout::vertical([
        Constraint::Length(3), // balance summary
        Constraint::Min(5),    // coin table
        Constraint::Length(5), // selected coin detail
    ])
    .split(area);

    draw_summary(frame, app, layout[0]);
    draw_coin_table(frame, app, layout[1]);
    draw_detail(frame, app, layout[2]);
}

fn draw_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Portfolio ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(common::dim_style());

    let balance_display = format_nanos(app.total_balance_iota);

    let text = Line::from(vec![
        Span::styled("  Total IOTA: ", Style::default().fg(Color::White)),
        Span::styled(&balance_display, Style::default().fg(Color::Green).bold()),
        Span::styled(
            format!("    {} coin objects", app.coins.len()),
            common::dim_style(),
        ),
    ]);

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_coin_table(frame: &mut Frame, app: &App, area: Rect) {
    if app.coins.is_empty() {
        let block = Block::default()
            .title(" Coins ")
            .title_style(common::header_style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(common::dim_style());

        let msg = if app.keys.is_empty() {
            "  No keys configured. Press 5 to go to Keys, then 'g' to generate one."
        } else if !app.connected {
            "  Connecting..."
        } else {
            "  No coins found. Press 'f' to request from faucet (testnet/devnet)."
        };

        frame.render_widget(Paragraph::new(msg).block(block), area);
        return;
    }

    // borders (2) + header (1) + header margin (1) = 4 rows overhead
    let visible_rows = area.height.saturating_sub(4) as usize;

    let show_all = app.show_multiple_owners();

    let header_cols: Vec<&str> = if show_all {
        vec!["Symbol", "Type", "Balance", "Object ID", "Owner", ""]
    } else {
        vec!["Symbol", "Type", "Balance", "Object ID", ""]
    };
    let header = Row::new(header_cols)
        .style(common::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .coins
        .iter()
        .enumerate()
        .skip(app.coins_offset)
        .take(visible_rows)
        .map(|(i, coin)| {
            let style = if i == app.coins_selected {
                common::selected_style()
            } else {
                Style::default()
            };

            let id_display = common::truncate_address(&coin.object_id, 24);

            let mut cells: Vec<Cell> = vec![
                Cell::from(coin.symbol.clone()),
                Cell::from(common::truncate_type(&coin.coin_type, 30)),
                Cell::from(coin.balance_display.clone()),
                Cell::from(id_display),
            ];
            if show_all {
                cells.push(Cell::from(coin.owner_alias.clone()));
            }
            cells.push(Cell::from("⏎").style(Style::default().fg(Color::Green)));

            Row::new(cells).style(style)
        })
        .collect();

    let widths: Vec<Constraint> = if show_all {
        vec![
            Constraint::Length(10),
            Constraint::Min(20),
            Constraint::Length(20),
            Constraint::Length(26),
            Constraint::Length(14),
            Constraint::Length(2),
        ]
    } else {
        vec![
            Constraint::Length(10),
            Constraint::Min(20),
            Constraint::Length(20),
            Constraint::Length(26),
            Constraint::Length(2),
        ]
    };

    let title = if app.coins.len() > visible_rows {
        format!(
            " Coins ({}) [{}-{}/{}] ",
            app.coins.len(),
            app.coins_offset + 1,
            (app.coins_offset + visible_rows).min(app.coins.len()),
            app.coins.len()
        )
    } else {
        format!(" Coins ({}) ", app.coins.len())
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

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .title_style(common::header_style())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(common::dim_style());

    let content = if let Some(coin) = app.coins.get(app.coins_selected) {
        let id_width = area.width.saturating_sub(16) as usize;
        vec![
            common::detail_line("Symbol", &coin.symbol, common::accent_style()),
            common::detail_line(
                "Balance",
                &coin.balance_display,
                Style::default().fg(Color::Green),
            ),
            common::detail_line("Type", &coin.coin_type, Style::default()),
            common::detail_line(
                "Object",
                &common::truncate_address(&coin.object_id, id_width),
                common::dim_style(),
            ),
        ]
    } else {
        vec![Line::from("  No coin selected")]
    };

    frame.render_widget(Paragraph::new(content).block(block), area);
}

fn format_nanos(nanos: u128) -> String {
    let whole = nanos / 1_000_000_000;
    let frac = nanos % 1_000_000_000;
    let frac_str = format!("{:09}", frac);
    let trimmed = frac_str.trim_end_matches('0');
    if trimmed.is_empty() {
        format!("{} IOTA", whole)
    } else {
        format!("{}.{} IOTA", whole, trimmed)
    }
}
