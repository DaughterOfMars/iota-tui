//! Drawing functions for popup overlays.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Popup};

use super::common::{
    ACCENT, DIM, accent_style, centered_rect_min, clamp_scroll, render_popup_scrollbar,
    truncate_address,
};

/// Dispatch popup drawing to the appropriate function.
pub fn draw_popup(frame: &mut Frame, app: &mut App) {
    use ratatui::widgets::Clear;

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
        Some(Popup::GenerateKeyAlias) => {
            let popup_area = centered_rect_min(50, 30, 40, 8, area);
            frame.render_widget(Clear, popup_area);
            draw_generate_key_alias_popup(frame, app, popup_area);
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
        Some(Popup::ConfirmClearTx) => {
            let popup_area = centered_rect_min(55, 40, 44, 10, area);
            frame.render_widget(Clear, popup_area);
            draw_confirm_clear_tx(frame, popup_area);
        }
        Some(Popup::LookupIotaName) => {
            let popup_area = centered_rect_min(60, 30, 48, 10, area);
            frame.render_widget(Clear, popup_area);
            draw_iota_name_lookup(frame, app, popup_area);
        }
        Some(Popup::ErrorLog) => {
            let popup_area = centered_rect_min(80, 80, 60, 20, area);
            frame.render_widget(Clear, popup_area);
            draw_error_log_popup(frame, app, popup_area);
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
        Line::from("  E          View error log"),
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
    let inner_height = area.height.saturating_sub(2) as usize;
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
                accent_style().bold(),
            )]));
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
    let fields = ["Label", "Address (0x... or IOTA-Name)", "Notes"];
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

fn draw_generate_key_alias_popup(frame: &mut Frame, app: &App, area: Rect) {
    let scheme = app.keys_gen_scheme.as_deref().unwrap_or("unknown");
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  Alias for new {} key:", scheme),
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
            "  Enter: confirm  Esc: cancel",
            Style::default().fg(DIM),
        )]),
    ];

    let block = Block::default()
        .title(" Key Alias ")
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

fn draw_confirm_clear_tx(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Clear all transaction commands?",
            Style::default().fg(Color::Yellow).bold(),
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
        .title(" Confirm Clear ")
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

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

fn draw_error_log_popup(frame: &mut Frame, app: &mut App, area: Rect) {
    let lines: Vec<Line> = if app.error_log_lines.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No errors logged.",
                Style::default().fg(DIM),
            )]),
        ]
    } else {
        app.error_log_lines
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect()
    };

    let content_len = lines.len();
    let inner_height = area.height.saturating_sub(2) as usize;
    clamp_scroll(&mut app.popup_scroll, content_len, inner_height);

    let block = Block::default()
        .title(" Error Log (newest first) ")
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.popup_scroll as u16, 0));
    frame.render_widget(paragraph, area);

    render_popup_scrollbar(frame, area, app.popup_scroll, content_len, inner_height);
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
        AddCommandType::TransferObjects => &["Recipient (address or alias)", "Object IDs"],
        AddCommandType::MoveCall => &["Package", "Module", "Function", "Type Args", "Arguments"],
        AddCommandType::SplitCoins => &["Coin Object ID", "Amounts (comma-sep)"],
        AddCommandType::MergeCoins => &["Primary Coin ID", "Source Coin IDs"],
        AddCommandType::Stake => &["Amount (IOTA)", "Validator (address or alias)"],
        AddCommandType::Unstake => &["Staked IOTA Object ID"],
    };

    // Check which field indices are multi-value for this command type
    let is_multi = |field_idx: usize| -> bool {
        matches!(
            (ct, field_idx),
            (AddCommandType::TransferObjects, 1) | (AddCommandType::MergeCoins, 1)
        )
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

        // For multi-value fields, show the count in the label
        if is_multi(i) && !app.tx_multi_values.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                format!("  {} ({} selected): ", field, app.tx_multi_values.len()),
                label_style,
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                format!("  {}: ", field),
                label_style,
            )]));
        }

        // For multi-value fields, show the accumulated items
        if is_multi(i) && !app.tx_multi_values.is_empty() {
            for mv in &app.tx_multi_values {
                lines.push(Line::from(vec![Span::styled(
                    format!("    • {}", truncate_address(mv, 36)),
                    Style::default().fg(Color::Green),
                )]));
            }
        }

        let input_style = if is_active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(DIM)
        };

        if is_multi(i) {
            // Multi-value: show the input line for adding more
            if is_active {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}|", value),
                    input_style,
                )]));
            } else if app.tx_multi_values.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "  (none)".to_string(),
                    Style::default().fg(DIM),
                )]));
            }
        } else {
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
        }

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

    let hint = if app.is_multi_value_field() {
        "  Tab/Enter: add item  Backspace: undo  Enter(empty): submit  Esc: cancel"
    } else {
        "  Tab: next field  Enter: add  Esc: cancel"
    };
    lines.push(Line::from(vec![Span::styled(
        hint,
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
