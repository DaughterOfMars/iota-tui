use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
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
        Screen::Coins => " Up/Down:navigate  Enter:details  f:faucet  r:refresh  ?:help",
        Screen::Objects => " Up/Down:navigate  Enter:details  r:refresh  ?:help",
        Screen::Transactions => " Up/Down:navigate  Enter:details  r:refresh  ?:help",
        Screen::Packages => " r:refresh  ?:help",
        Screen::AddressBook => " Up/Down:navigate  a:add  e:edit  d:delete  l:iota-name  ?:help",
        Screen::Keys => {
            " Up/Down:navigate  Enter:active  Space:visible  g:gen  i:import  e:rename  ?:help"
        }
        Screen::TxBuilder => " Left/Right:step  Up/Down:navigate  a:add  Enter:confirm  ?:help",
    }
}

pub fn draw_popup(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    match app.popup {
        Some(Popup::Help) => {
            let popup_area = centered_rect_min(70, 80, 50, 24, area);
            frame.render_widget(Clear, popup_area);
            draw_help_popup(frame, app, popup_area);
        }
        Some(Popup::Detail) => {
            let popup_area = centered_rect_min(65, 70, 50, 16, area);
            frame.render_widget(Clear, popup_area);
            draw_detail_popup(frame, app, popup_area);
        }
        Some(Popup::AddAddress) => {
            let popup_area = centered_rect_min(60, 60, 48, 14, area);
            frame.render_widget(Clear, popup_area);
            draw_address_form(frame, app, popup_area, "Add Address");
        }
        Some(Popup::EditAddress) => {
            let popup_area = centered_rect_min(60, 60, 48, 14, area);
            frame.render_widget(Clear, popup_area);
            draw_address_form(frame, app, popup_area, "Edit Address");
        }
        Some(Popup::GenerateKey) => {
            let popup_area = centered_rect_min(50, 40, 36, 11, area);
            frame.render_widget(Clear, popup_area);
            draw_generate_key_popup(frame, popup_area);
        }
        Some(Popup::ImportKey) => {
            let popup_area = centered_rect_min(60, 30, 48, 10, area);
            frame.render_widget(Clear, popup_area);
            draw_import_key_popup(frame, app, popup_area);
        }
        Some(Popup::AddCommand) => {
            let popup_area = centered_rect_min(50, 50, 40, 16, area);
            frame.render_widget(Clear, popup_area);
            draw_add_command_popup(frame, popup_area);
        }
        Some(Popup::AddCommandForm) => {
            let popup_area = centered_rect_min(65, 60, 52, 14, area);
            frame.render_widget(Clear, popup_area);
            draw_add_command_form(frame, app, popup_area);
        }
        Some(Popup::RenameKey) => {
            let popup_area = centered_rect_min(50, 30, 40, 8, area);
            frame.render_widget(Clear, popup_area);
            draw_rename_key_popup(frame, app, popup_area);
        }
        Some(Popup::SwitchNetwork) => {
            let popup_area = centered_rect_min(50, 40, 36, 12, area);
            frame.render_widget(Clear, popup_area);
            draw_switch_network_popup(frame, popup_area);
        }
        Some(Popup::ConfirmDeleteAddress) => {
            let popup_area = centered_rect_min(55, 40, 44, 10, area);
            frame.render_widget(Clear, popup_area);
            draw_confirm_delete_address(frame, app, popup_area);
        }
        Some(Popup::ConfirmDeleteKey) => {
            let popup_area = centered_rect_min(55, 40, 44, 10, area);
            frame.render_widget(Clear, popup_area);
            draw_confirm_delete_key(frame, app, popup_area);
        }
        Some(Popup::LookupIotaName) => {
            let popup_area = centered_rect_min(60, 30, 48, 10, area);
            frame.render_widget(Clear, popup_area);
            draw_iota_name_lookup(frame, app, popup_area);
        }
        Some(Popup::ConfirmQuit) => {
            let popup_area = centered_rect_min(50, 30, 40, 7, area);
            frame.render_widget(Clear, popup_area);
            draw_confirm_quit(frame, popup_area);
        }
        None => {}
    }
}

fn draw_help_popup(frame: &mut Frame, app: &mut App, area: Rect) {
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
        Line::from("  Up/Down    Move up/down"),
        Line::from("  Left/Right Move left/right (Tx Builder)"),
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
        Line::from("  n          Switch network"),
        Line::from("  Space      Toggle key visibility (Keys screen)"),
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

    let content_len = text.len();
    let inner_height = area.height.saturating_sub(2) as usize; // border top + bottom
    clamp_scroll(&mut app.popup_scroll, content_len, inner_height);

    let block = Block::default()
        .title(" Help ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.popup_scroll as u16, 0));
    frame.render_widget(paragraph, area);

    render_popup_scrollbar(frame, area, app.popup_scroll, content_len, inner_height);
}

fn draw_detail_popup(frame: &mut Frame, app: &mut App, area: Rect) {
    let (title, fields) = app.detail_info();

    let mut lines: Vec<Line> = vec![Line::from("")];
    if fields.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No item selected",
            Style::default().fg(DIM),
        )]));
    } else {
        for (label, value) in &fields {
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", label),
                Style::default().fg(ACCENT).bold(),
            )]));
            // Wrap long values across multiple lines
            let max_w = area.width.saturating_sub(6) as usize;
            if value.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "  (empty)",
                    Style::default().fg(DIM),
                )]));
            } else if value.len() <= max_w {
                lines.push(Line::from(format!("  {}", value)));
            } else {
                for chunk in value.as_bytes().chunks(max_w) {
                    let s = String::from_utf8_lossy(chunk);
                    lines.push(Line::from(format!("  {}", s)));
                }
            }
            lines.push(Line::from(""));
        }
    }
    lines.push(Line::from(vec![Span::styled(
        "  Esc to close",
        Style::default().fg(DIM),
    )]));

    let content_len = lines.len();
    let inner_height = area.height.saturating_sub(2) as usize;
    clamp_scroll(&mut app.popup_scroll, content_len, inner_height);

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.popup_scroll as u16, 0));
    frame.render_widget(paragraph, area);

    render_popup_scrollbar(frame, area, app.popup_scroll, content_len, inner_height);
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

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_import_key_popup(frame: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Paste private key:",
            Style::default().bold(),
        )]),
        Line::from(vec![Span::styled(
            "  (hex, base64, or bech32 iotaprivkey1...)",
            Style::default().fg(DIM),
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

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_rename_key_popup(frame: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled("  New alias:", Style::default().bold())]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}|", &app.input_buffer),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Enter: save  Esc: cancel",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Rename Key ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_switch_network_popup(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Select network:",
            Style::default().bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1/m] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Mainnet"),
        ]),
        Line::from(vec![
            Span::styled("  [2/t] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Testnet"),
        ]),
        Line::from(vec![
            Span::styled("  [3/d] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Devnet"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Esc to cancel",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Switch Network ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_confirm_delete_address(frame: &mut Frame, app: &App, area: Rect) {
    let label = app
        .user_address_index(app.address_selected)
        .and_then(|i| app.address_book.get(i))
        .map(|e| e.label.as_str())
        .unwrap_or("?");

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Delete this address?",
            Style::default().fg(Color::Red).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  \"{}\"", label),
            Style::default().fg(Color::White).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter/y", Style::default().fg(ACCENT).bold()),
            Span::raw(" confirm   "),
            Span::styled("Esc/n", Style::default().fg(ACCENT).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(" Confirm Delete ")
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_confirm_delete_key(frame: &mut Frame, app: &App, area: Rect) {
    let alias = app
        .keys
        .get(app.keys_selected)
        .map(|k| k.alias.as_str())
        .unwrap_or("?");

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Delete this key?",
            Style::default().fg(Color::Red).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  \"{}\"", alias),
            Style::default().fg(Color::White).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter/y", Style::default().fg(ACCENT).bold()),
            Span::raw(" confirm   "),
            Span::styled("Esc/n", Style::default().fg(ACCENT).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(" Confirm Delete ")
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_iota_name_lookup(frame: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Enter an IOTA name (e.g. alice@iota):",
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
            "  Enter: lookup  Esc: cancel",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" IOTA-Name Lookup ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_confirm_quit(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Quit IOTA Wallet TUI?",
            Style::default().fg(Color::Yellow).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter/y", Style::default().fg(ACCENT).bold()),
            Span::raw(" quit   "),
            Span::styled("Esc/n", Style::default().fg(ACCENT).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(" Confirm Quit ")
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_add_command_popup(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Select command type:",
            Style::default().bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1/t] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Transfer IOTA"),
        ]),
        Line::from(vec![
            Span::styled("  [2/o] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Transfer Objects"),
        ]),
        Line::from(vec![
            Span::styled("  [3/m] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Move Call"),
        ]),
        Line::from(vec![
            Span::styled("  [4/s] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Split Coins"),
        ]),
        Line::from(vec![
            Span::styled("  [5/r] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Merge Coins"),
        ]),
        Line::from(vec![
            Span::styled("  [6/k] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Stake"),
        ]),
        Line::from(vec![
            Span::styled("  [7/u] ", Style::default().fg(ACCENT).bold()),
            Span::raw("Unstake"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Esc to cancel",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Add Command ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_add_command_form(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::app::AddCommandType;
    let Some(ct) = app.tx_adding_cmd else {
        return;
    };

    let fields: &[&str] = match ct {
        AddCommandType::TransferIota => &["Recipient (address or alias)", "Amount (IOTA)"],
        AddCommandType::TransferObjects => {
            &["Recipient (address or alias)", "Object IDs (comma-sep)"]
        }
        AddCommandType::MoveCall => &["Package", "Module", "Function", "Type Args", "Arguments"],
        AddCommandType::SplitCoins => &["Coin Object ID", "Amounts (comma-sep)"],
        AddCommandType::MergeCoins => &["Primary Coin ID", "Source Coin IDs (comma-sep)"],
        AddCommandType::Stake => &["Amount (IOTA)", "Validator (address or alias)"],
        AddCommandType::Unstake => &["Staked IOTA Object ID"],
    };

    let mut lines = vec![Line::from("")];
    for (i, field) in fields.iter().enumerate() {
        let is_active = i == app.tx_edit_field;
        let value = if is_active {
            &app.input_buffer
        } else {
            app.tx_edit_buffers.get(i).map(|s| s.as_str()).unwrap_or("")
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
            value.to_string()
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {}", display),
            input_style,
        )]));

        if is_active && !app.autocomplete.is_empty() {
            for (j, (alias, addr)) in app.autocomplete.iter().enumerate() {
                let is_sel = app.autocomplete_idx == Some(j);
                let trunc = truncate_address(addr, 24);
                let style = if is_sel {
                    Style::default().fg(ACCENT).bold()
                } else {
                    Style::default().fg(DIM)
                };
                let prefix = if is_sel { "▸ " } else { "  " };
                lines.push(Line::from(vec![Span::styled(
                    format!("    {}{} → {}", prefix, alias, trunc),
                    style,
                )]));
            }
        }

        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![Span::styled(
        "  Tab: next field  Enter: add  Esc: cancel",
        Style::default().fg(DIM),
    )]));

    let content_len = lines.len();
    let inner_height = area.height.saturating_sub(2) as usize;
    clamp_scroll(&mut app.popup_scroll, content_len, inner_height);

    let block = Block::default()
        .title(format!(" {} ", ct.label()))
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.popup_scroll as u16, 0)),
        area,
    );

    render_popup_scrollbar(frame, area, app.popup_scroll, content_len, inner_height);
}

pub fn centered_rect_min(
    percent_x: u16,
    percent_y: u16,
    min_w: u16,
    min_h: u16,
    area: Rect,
) -> Rect {
    let w = (area.width * percent_x / 100).max(min_w).min(area.width);
    let h = (area.height * percent_y / 100).max(min_h).min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
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

/// Clamp scroll offset so content doesn't scroll past the end.
fn clamp_scroll(scroll: &mut usize, content_len: usize, visible: usize) {
    let max = content_len.saturating_sub(visible);
    if *scroll > max {
        *scroll = max;
    }
}

/// Render a scrollbar on the right edge of a popup area.
/// Only draws if content overflows the visible area.
fn render_popup_scrollbar(
    frame: &mut Frame,
    area: Rect,
    scroll: usize,
    content_len: usize,
    visible: usize,
) {
    if content_len <= visible {
        return;
    }
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(ACCENT))
        .track_style(Style::default().fg(DIM));
    let mut state = ScrollbarState::new(content_len.saturating_sub(visible)).position(scroll);
    // Render inside the border (inset by 1 on each side)
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    frame.render_stateful_widget(scrollbar, inner, &mut state);
}
