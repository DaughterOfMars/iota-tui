use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, InputMode, Popup, Screen};

const ACCENT: Color = Color::Cyan;
const HIGHLIGHT: Color = Color::Yellow;
const DIM: Color = Color::DarkGray;

pub fn draw_tabs(frame: &mut Frame, app: &mut App, area: Rect) {
    app.tab_areas.clear();

    let tabs: Vec<Span> = Screen::ALL
        .iter()
        .enumerate()
        .flat_map(|(i, screen)| {
            let label = format!(" {} {} ", i + 1, screen.title());
            let style = if *screen == app.screen {
                Style::default().fg(Color::Black).bg(ACCENT).bold()
            } else {
                Style::default().fg(Color::White).dim()
            };
            let sep = Span::styled(" ", Style::default());
            vec![Span::styled(label, style), sep]
        })
        .collect();

    // Calculate tab areas for mouse hit-testing
    let mut x = area.x;
    for (i, screen) in Screen::ALL.iter().enumerate() {
        let label = format!(" {} {} ", i + 1, screen.title());
        let width = label.len() as u16;
        app.tab_areas.push(Rect::new(x, area.y, width, 1));
        x += width + 1; // +1 for separator
    }

    let line = Line::from(tabs);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

pub fn draw_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new("─".repeat(area.width as usize)).style(Style::default().fg(DIM));
    frame.render_widget(sep, area);
}

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let active_addr = app
        .active_key()
        .map(|k| format!("{}..{}", &k.address[..8], &k.address[k.address.len() - 6..]))
        .unwrap_or_else(|| "No active key".into());

    let left = match &app.status_message {
        Some((msg, _)) => Span::styled(msg.as_str(), Style::default().fg(HIGHLIGHT)),
        None => Span::styled(screen_hint(app.screen), Style::default().fg(DIM)),
    };

    let mode_indicator = if app.input_mode == InputMode::Editing {
        Span::styled(
            " EDIT ",
            Style::default().fg(Color::Black).bg(Color::Green).bold(),
        )
    } else {
        Span::styled(
            " NORMAL ",
            Style::default().fg(Color::Black).bg(Color::Blue).bold(),
        )
    };

    // Network indicator
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

    let right_text = format!("  {} ", active_addr);
    let right = Span::styled(&right_text, Style::default().fg(ACCENT));

    let fixed_right =
        mode_indicator.width() as u16 + net_indicator.width() as u16 + right_text.len() as u16;
    let left_width = area.width.saturating_sub(fixed_right);

    let cols = Layout::horizontal([
        Constraint::Length(mode_indicator.width() as u16),
        Constraint::Length(left_width),
        Constraint::Length(net_indicator.width() as u16),
        Constraint::Min(0),
    ])
    .split(area);

    frame.render_widget(Paragraph::new(Line::from(vec![mode_indicator])), cols[0]);
    frame.render_widget(Paragraph::new(Line::from(vec![left])), cols[1]);
    frame.render_widget(Paragraph::new(Line::from(vec![net_indicator])), cols[2]);
    frame.render_widget(
        Paragraph::new(Line::from(vec![right])).alignment(Alignment::Right),
        cols[3],
    );
}

fn screen_hint(screen: Screen) -> &'static str {
    match screen {
        Screen::Coins => " j/k:navigate  Enter:details  f:faucet  r:refresh  ?:help",
        Screen::Objects => " j/k:navigate  Enter:details  r:refresh  ?:help",
        Screen::Packages => " r:refresh  ?:help",
        Screen::AddressBook => " j/k:navigate  a:add  e:edit  d:delete  ?:help",
        Screen::Keys => " j/k:navigate  Enter:set active  g:generate  i:import  ?:help",
        Screen::TxBuilder => " h/l:step  j/k:navigate  a:add  Enter:confirm  ?:help",
    }
}

pub fn draw_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_area = centered_rect(60, 60, area);

    frame.render_widget(Clear, popup_area);

    match app.popup {
        Some(Popup::Help) => draw_help_popup(frame, popup_area),
        Some(Popup::Confirm) => draw_confirm_popup(frame, popup_area),
        Some(Popup::AddAddress) => draw_address_form(frame, app, popup_area, "Add Address"),
        Some(Popup::EditAddress) => draw_address_form(frame, app, popup_area, "Edit Address"),
        Some(Popup::GenerateKey) => draw_generate_key_popup(frame, popup_area),
        Some(Popup::ImportKey) => draw_import_key_popup(frame, app, popup_area),
        Some(Popup::AddRecipient) => draw_add_recipient_popup(frame, app, popup_area),
        None => {}
    }
}

fn draw_help_popup(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(vec![Span::styled(
            "IOTA Wallet TUI",
            Style::default().fg(ACCENT).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().bold().underlined(),
        )]),
        Line::from("  1-6        Switch screens"),
        Line::from("  Tab/S-Tab  Cycle screens"),
        Line::from("  j/k        Move up/down"),
        Line::from("  h/l        Move left/right (Tx Builder)"),
        Line::from("  Enter      Select / Confirm"),
        Line::from("  Esc        Cancel / Close popup"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions",
            Style::default().bold().underlined(),
        )]),
        Line::from("  a          Add entry"),
        Line::from("  e          Edit entry"),
        Line::from("  d/Del      Delete entry"),
        Line::from("  g          Generate key"),
        Line::from("  i          Import key"),
        Line::from("  p          Toggle private key visibility"),
        Line::from("  r          Refresh data from network"),
        Line::from("  f          Request faucet (testnet/devnet)"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General",
            Style::default().bold().underlined(),
        )]),
        Line::from("  ?          Show this help"),
        Line::from("  q/Ctrl-c   Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mouse: ", Style::default().bold()),
            Span::raw("Click tabs, list items. Scroll to navigate."),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press Esc to close",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Help ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_confirm_popup(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 30, area);
    frame.render_widget(Clear, popup);

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Transaction submitted!",
            Style::default().fg(Color::Green).bold(),
        )]),
        Line::from(""),
        Line::from("Check the status bar for the transaction digest."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press Enter to close",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Confirmed ")
        .title_style(Style::default().fg(Color::Green).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        popup,
    );
}

fn draw_address_form(frame: &mut Frame, app: &App, area: Rect, title: &str) {
    let fields = ["Label", "Address", "Notes"];
    let mut lines = vec![Line::from("")];

    for (i, field) in fields.iter().enumerate() {
        let is_active = i == app.address_edit_field;
        let value = if is_active {
            &app.input_buffer
        } else {
            &app.address_edit_buffers[i]
        };

        let label_style = if is_active {
            Style::default().fg(ACCENT).bold()
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {}: ", field),
            label_style,
        )]));

        let input_style = if is_active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(DIM)
        };

        let display = if value.is_empty() && !is_active {
            "(empty)".to_string()
        } else if is_active {
            format!("{}|", value)
        } else {
            value.clone()
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {}", display),
            input_style,
        )]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![Span::styled(
        "  Tab: next field  Enter: save  Esc: cancel",
        Style::default().fg(DIM),
    )]));

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_generate_key_popup(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 40, area);
    frame.render_widget(Clear, popup);

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Select key scheme:",
            Style::default().bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1/e] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Ed25519"),
        ]),
        Line::from(vec![
            Span::styled("  [2/s] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Secp256k1"),
        ]),
        Line::from(vec![
            Span::styled("  [3/r] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Secp256r1"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Esc to cancel",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Generate Key ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(text).block(block), popup);
}

fn draw_import_key_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(60, 30, area);
    frame.render_widget(Clear, popup);

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Paste private key (base64 or hex):",
            Style::default().bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}|", &app.input_buffer),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Enter: import  Esc: cancel",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Import Key ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(text).block(block), popup);
}

fn draw_add_recipient_popup(frame: &mut Frame, app: &App, area: Rect) {
    let fields = ["Address", "Amount (IOTA)"];
    let mut lines = vec![Line::from("")];

    for (i, field) in fields.iter().enumerate() {
        let is_active = i == app.tx_edit_field;
        let value = if is_active {
            &app.input_buffer
        } else {
            &app.tx_edit_buffers[i]
        };

        let label_style = if is_active {
            Style::default().fg(ACCENT).bold()
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {}: ", field),
            label_style,
        )]));

        let input_style = if is_active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(DIM)
        };

        let display = if value.is_empty() && !is_active {
            "(empty)".to_string()
        } else if is_active {
            format!("{}|", value)
        } else {
            value.clone()
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {}", display),
            input_style,
        )]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![Span::styled(
        "  Tab: next field  Enter: add  Esc: cancel",
        Style::default().fg(DIM),
    )]));

    let block = Block::default()
        .title(" Add Recipient ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

pub fn truncate_type(type_str: &str, max_width: usize) -> String {
    if type_str.len() <= max_width {
        return type_str.to_string();
    }
    if max_width < 6 {
        return type_str[..max_width].to_string();
    }
    format!("{}...", &type_str[..max_width.saturating_sub(3)])
}

pub fn truncate_address(addr: &str, max_width: usize) -> String {
    if addr.len() <= max_width {
        return addr.to_string();
    }
    if max_width < 10 {
        return addr[..max_width].to_string();
    }
    let prefix = max_width / 2 - 1;
    let suffix = max_width - prefix - 2;
    format!("{}..{}", &addr[..prefix], &addr[addr.len() - suffix..])
}

pub fn selected_style() -> Style {
    Style::default().bg(Color::DarkGray).fg(Color::White).bold()
}

pub fn header_style() -> Style {
    Style::default().fg(ACCENT).bold()
}

pub fn dim_style() -> Style {
    Style::default().fg(DIM)
}

pub fn accent_style() -> Style {
    Style::default().fg(ACCENT)
}
