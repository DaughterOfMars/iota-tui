//! Drawing functions for popup overlays.

mod command_form;
mod detail;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::{App, Popup, PopupFocus, Screen};

use super::common::{
    centered_rect_min, clamp_scroll, color_at, dim_at, render_popup_scrollbar, screen_hints,
    selected_style, sparkle_text,
};

/// Build a button line with focus highlighting.
fn button_line(submit_label: &str, focus: PopupFocus, prefix: &str) -> Line<'static> {
    let submit_style = if focus == PopupFocus::Submit {
        Style::default().fg(Color::Black).bg(color_at(0)).bold()
    } else {
        Style::default().fg(color_at(0)).bold()
    };
    let cancel_style = if focus == PopupFocus::Cancel {
        Style::default().fg(Color::Black).bg(Color::Red).bold()
    } else {
        Style::default().fg(dim_at(0))
    };
    Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(dim_at(0))),
        Span::styled(format!("[ {} ]", submit_label), submit_style),
        Span::raw("  "),
        Span::styled("[ Cancel ]".to_string(), cancel_style),
    ])
}

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
            detail::draw_detail_popup(frame, app, popup_area);
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
            command_form::draw_add_command_popup(frame, popup_area);
        }
        Some(Popup::AddCommandForm) => {
            let popup_area = centered_rect_min(65, 60, 52, 14, area);
            frame.render_widget(Clear, popup_area);
            command_form::draw_add_command_form(frame, app, popup_area);
        }
        Some(Popup::RenameKey) => {
            let popup_area = centered_rect_min(50, 30, 40, 8, area);
            frame.render_widget(Clear, popup_area);
            draw_rename_key_popup(frame, app, popup_area);
        }
        Some(Popup::SwitchNetwork) => {
            let popup_area = centered_rect_min(50, 50, 44, 16, area);
            frame.render_widget(Clear, popup_area);
            draw_switch_network_popup(frame, app, popup_area);
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
        Some(Popup::SplitCoin) => {
            let popup_area = centered_rect_min(50, 30, 40, 9, area);
            frame.render_widget(Clear, popup_area);
            draw_split_coin_popup(frame, app, popup_area);
        }
        Some(Popup::MergeCoin) => {
            let h = (app.merge_candidates.len() as u16 + 6).min(area.height - 2);
            let popup_area = centered_rect_min(60, 60, 50, h, area);
            frame.render_widget(Clear, popup_area);
            draw_merge_coin_popup(frame, app, popup_area);
        }
        Some(Popup::QuickTransfer) => {
            let popup_area = centered_rect_min(60, 50, 48, 13, area);
            frame.render_widget(Clear, popup_area);
            draw_quick_transfer_popup(frame, app, popup_area);
        }
        Some(Popup::ObjectTransfer) => {
            let popup_area = centered_rect_min(60, 40, 48, 10, area);
            frame.render_widget(Clear, popup_area);
            draw_object_transfer_popup(frame, app, popup_area);
        }
        Some(Popup::ActionsMenu) => {
            let popup_area = actions_menu_area(app, area);
            frame.render_widget(Clear, popup_area);
            draw_actions_menu(frame, app, popup_area);
        }
        Some(Popup::Settings) => {
            let popup_area = centered_rect_min(75, 75, 60, 20, area);
            frame.render_widget(Clear, popup_area);
            draw_settings_popup(frame, app, popup_area);
        }
        Some(Popup::Welcome) => {
            let popup_area = centered_rect_min(50, 30, 42, 9, area);
            frame.render_widget(Clear, popup_area);
            draw_welcome_popup(frame, popup_area);
        }
        None => {}
    }
}

fn draw_help_popup(frame: &mut Frame, app: &mut App, area: Rect) {
    let hints = screen_hints(app.screen);
    let screen_title = app.screen.title();

    let mut text = vec![
        Line::from(vec![Span::styled(
            format!("{} — Help", screen_title),
            Style::default().fg(color_at(0)).bold(),
        )]),
        Line::from(""),
    ];

    // Screen-specific actions
    if !hints.is_empty() {
        text.push(Line::from(vec![Span::styled(
            "Actions",
            Style::default().bold().underlined(),
        )]));
        for &(key, short_label, _) in &hints {
            let desc = hint_description(key, short_label, app.screen);
            text.push(Line::from(format!("  {:<12}{}", key, desc)));
        }
        text.push(Line::from(""));
    }

    // Navigation
    text.push(Line::from(vec![Span::styled(
        "Navigation",
        Style::default().bold().underlined(),
    )]));
    text.push(Line::from("  Up/Down    Navigate list"));
    match app.screen {
        Screen::TxBuilder => {
            text.push(Line::from("  Left/Right Move between steps"));
        }
        Screen::Packages => {
            text.push(Line::from("  Left/Esc   Back to parent level"));
        }
        _ => {}
    }
    text.push(Line::from("  1-0        Switch screens"));
    text.push(Line::from("  Tab        Toggle sidebar"));
    text.push(Line::from(""));

    // Global
    text.push(Line::from(vec![Span::styled(
        "Global",
        Style::default().bold().underlined(),
    )]));
    text.push(Line::from("  n          Switch network"));
    text.push(Line::from("  .          Actions menu"));
    text.push(Line::from("  E          Error log"));
    text.push(Line::from("  q/Ctrl-c   Quit"));
    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        "Press Esc to close",
        Style::default().fg(dim_at(0)),
    )]));

    let content_len = text.len();
    let inner_height = area.height.saturating_sub(2) as usize;
    clamp_scroll(&mut app.popup_scroll, content_len, inner_height);

    let block = Block::default()
        .title(sparkle_text(" Help "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    let paragraph = Paragraph::new(text)
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
            Style::default().fg(color_at(0)).bold()
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
            Style::default().fg(dim_at(0))
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

    lines.push(button_line("Save", app.popup_focus, "  Tab: next  "));

    let block = Block::default()
        .title(sparkle_text(&format!(" {} ", title)))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

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
            Span::styled("  [1/e] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Ed25519"),
        ]),
        Line::from(vec![
            Span::styled("  [2/s] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Secp256k1"),
        ]),
        Line::from(vec![
            Span::styled("  [3/r] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Secp256r1"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Esc to cancel",
            Style::default().fg(dim_at(0)),
        )]),
    ];

    let block = Block::default()
        .title(sparkle_text(" Generate Key "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

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
        button_line("Confirm", app.popup_focus, "  "),
    ];

    let block = Block::default()
        .title(sparkle_text(" Key Alias "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

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
            Style::default().fg(dim_at(0)),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}|", &app.input_buffer),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from(""),
        button_line("Import", app.popup_focus, "  "),
    ];

    let block = Block::default()
        .title(sparkle_text(" Import Key "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

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
        button_line("Save", app.popup_focus, "  "),
    ];

    let block = Block::default()
        .title(sparkle_text(" Rename Key "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_switch_network_popup(frame: &mut Frame, app: &App, area: Rect) {
    let mut text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Connected: ", Style::default().fg(dim_at(0))),
            Span::styled(&app.network_name, Style::default().fg(Color::White).bold()),
        ]),
    ];

    // Show network overview stats if available
    if let Some(ref ov) = app.explorer.overview {
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::styled("  Chain ID    ", Style::default().fg(dim_at(0))),
            Span::styled(&ov.chain_id, Style::default().fg(Color::White)),
        ]));
        text.push(Line::from(vec![
            Span::styled("  Epoch       ", Style::default().fg(dim_at(0))),
            Span::styled(&ov.epoch, Style::default().fg(Color::White)),
        ]));
        text.push(Line::from(vec![
            Span::styled("  Gas Price   ", Style::default().fg(dim_at(0))),
            Span::styled(&ov.gas_price, Style::default().fg(Color::White)),
        ]));
        text.push(Line::from(vec![
            Span::styled("  Checkpoint  ", Style::default().fg(dim_at(0))),
            Span::styled(&ov.latest_checkpoint, Style::default().fg(Color::White)),
        ]));
        text.push(Line::from(vec![
            Span::styled("  Total Txs   ", Style::default().fg(dim_at(0))),
            Span::styled(&ov.total_txs, Style::default().fg(Color::White)),
        ]));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        "  Switch to:",
        Style::default().bold(),
    )]));
    text.push(Line::from(vec![
        Span::styled("  [1/m] ", Style::default().fg(color_at(0)).bold()),
        Span::raw("Mainnet"),
    ]));
    text.push(Line::from(vec![
        Span::styled("  [2/t] ", Style::default().fg(color_at(0)).bold()),
        Span::raw("Testnet"),
    ]));
    text.push(Line::from(vec![
        Span::styled("  [3/d] ", Style::default().fg(color_at(0)).bold()),
        Span::raw("Devnet"),
    ]));
    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        "  Esc to close",
        Style::default().fg(dim_at(0)),
    )]));

    let block = Block::default()
        .title(sparkle_text(" Network "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

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
            Span::styled("  Enter/y", Style::default().fg(color_at(0)).bold()),
            Span::raw(" confirm   "),
            Span::styled("Esc/n", Style::default().fg(color_at(0)).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(sparkle_text(" Confirm Delete "))
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
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
            Span::styled("  Enter/y", Style::default().fg(color_at(0)).bold()),
            Span::raw(" confirm   "),
            Span::styled("Esc/n", Style::default().fg(color_at(0)).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(sparkle_text(" Confirm Delete "))
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
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
            Span::styled("  Enter/y", Style::default().fg(color_at(0)).bold()),
            Span::raw(" confirm   "),
            Span::styled("Esc/n", Style::default().fg(color_at(0)).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(sparkle_text(" Confirm Clear "))
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
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
        button_line("Lookup", app.popup_focus, "  "),
    ];

    let block = Block::default()
        .title(sparkle_text(" IOTA-Name Lookup "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_error_log_popup(frame: &mut Frame, app: &mut App, area: Rect) {
    let lines: Vec<Line> = if app.error_log_lines.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No errors logged.",
                Style::default().fg(dim_at(0)),
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
        .title(sparkle_text(" Error Log (newest first) "))
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.popup_scroll as u16, 0));
    frame.render_widget(paragraph, area);

    render_popup_scrollbar(frame, area, app.popup_scroll, content_len, inner_height);
}

/// Compute the area for the actions menu popup, anchored above the status bar.
pub fn actions_menu_area(app: &App, frame_area: Rect) -> Rect {
    let hints = screen_hints(app.screen);
    let clickable: Vec<_> = hints.iter().filter(|(_, _, id)| !id.is_empty()).collect();
    let row_count = clickable.len() as u16;
    let width: u16 = 28;
    let height = row_count + 2; // +2 for borders

    // Anchor x to the Actions button position, or fallback to left edge
    let button_x = app
        .hint_areas
        .iter()
        .find(|(_, id)| *id == "open_menu")
        .map(|(r, _)| r.x)
        .unwrap_or(0);
    let x = button_x.min(frame_area.width.saturating_sub(width));
    // Place just above the status bar (last row of frame)
    let status_bar_y = frame_area.y + frame_area.height.saturating_sub(1);
    let y = status_bar_y.saturating_sub(height);

    Rect::new(
        x,
        y,
        width.min(frame_area.width),
        height.min(frame_area.height),
    )
}

fn draw_actions_menu(frame: &mut Frame, app: &App, area: Rect) {
    let hints = screen_hints(app.screen);
    let clickable: Vec<_> = hints.iter().filter(|(_, _, id)| !id.is_empty()).collect();

    let mut lines: Vec<Line> = Vec::new();
    for (i, (key_label, description, _)) in clickable.iter().enumerate() {
        let style = if i == app.action_menu_selected {
            selected_style()
        } else {
            Style::default()
        };
        // Capitalize description for display
        let desc = capitalize(description);
        let line = Line::from(vec![
            Span::styled(
                format!(" [{}]", key_label),
                style.fg(color_at(i as u32)).bold(),
            ),
            Span::styled(format!(" {} ", desc), style),
        ]);
        lines.push(line);
    }

    let block = Block::default()
        .title(sparkle_text(" Actions "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Map a hint's short label to a full description, context-sensitive per screen.
fn hint_description(key: &'static str, short: &'static str, screen: Screen) -> &'static str {
    // Screen-specific overrides first
    match (screen, key) {
        (Screen::Coins, "m") => "Merge selected coins",
        (Screen::Coins, "s") => "Split selected coin",
        (Screen::Coins, "x") => "Quick transfer coin",
        (Screen::Coins, "p") => "Portfolio view (multi-account)",
        (Screen::Coins, "t") => "Search by coin type",
        (Screen::Coins, "f") => "Request faucet tokens",
        (Screen::Objects, "t") => "Search by object type",
        (Screen::Staking, "u") => "Unstake selected stake",
        (Screen::Packages, "Enter") => "Browse package modules",
        (Screen::Packages, "e") => "Explore package in explorer",
        (Screen::Packages, "Esc") => "Back to parent level",
        (Screen::AddressBook, "a") => "Add address",
        (Screen::AddressBook, "e") => "Edit selected entry",
        (Screen::AddressBook, "d") => "Delete selected entry",
        (Screen::AddressBook, "l") => "Lookup IOTA-Name",
        (Screen::Keys, "a") => "Activate selected key",
        (Screen::Keys, "Sp") => "Toggle key visibility",
        (Screen::Keys, "g") => "Generate new key",
        (Screen::Keys, "i") => "Import key from private key",
        (Screen::Keys, "e") => "Rename selected key",
        (Screen::Keys, "d") => "Delete selected key",
        (Screen::TxBuilder, "a") => "Add command",
        (Screen::TxBuilder, "d") => "Delete selected command",
        (Screen::TxBuilder, "c") => "Clear / reset transaction",
        (Screen::Explorer, "Enter") => "Search / lookup",
        _ => match short {
            "explore" => "Explore in explorer",
            "filter" | "search" => "Filter list",
            "copy" => "Copy selected to clipboard",
            "export" => "Export list as CSV",
            "refresh" => "Refresh data",
            "help" => "Show this help",
            other => other,
        },
    }
}

fn draw_split_coin_popup(frame: &mut Frame, app: &App, area: Rect) {
    let coin_label = app
        .coins
        .get(app.coins_selected)
        .map(|c| format!("{} ({})", c.symbol, c.balance_display))
        .unwrap_or_default();

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  Split: {}", coin_label),
            Style::default().bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Number of parts: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}|", &app.input_buffer),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        button_line("Split", app.popup_focus, "  "),
    ];

    let block = Block::default()
        .title(sparkle_text(" Split Coin "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_merge_coin_popup(frame: &mut Frame, app: &App, area: Rect) {
    let selected_count = app.merge_candidates.iter().filter(|c| c.3).count();
    let total = app.merge_candidates.len();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Space", Style::default().fg(color_at(0)).bold()),
            Span::styled(" toggle  ", Style::default().fg(Color::DarkGray)),
            Span::styled("a", Style::default().fg(color_at(0)).bold()),
            Span::styled(" all  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("({}/{})", selected_count, total),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
    ];

    for (i, (id, symbol, balance, checked)) in app.merge_candidates.iter().enumerate() {
        let is_cursor = i == app.merge_cursor;
        let check = if *checked { "[x]" } else { "[ ]" };
        let short_id = if id.len() > 10 {
            format!("{}..{}", &id[..6], &id[id.len() - 4..])
        } else {
            id.clone()
        };

        let style = if is_cursor {
            selected_style()
        } else if *checked {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", check), style),
            Span::styled(format!("{:<6} ", symbol), style.bold()),
            Span::styled(format!("{:<14} ", balance), style),
            Span::styled(short_id, style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(button_line("Merge", app.popup_focus, "  "));

    let block = Block::default()
        .title(sparkle_text(" Merge Coins "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_quick_transfer_popup(frame: &mut Frame, app: &App, area: Rect) {
    let fields = ["Recipient (address or alias)", "Amount (IOTA)"];
    let mut lines = vec![Line::from("")];

    for (i, field) in fields.iter().enumerate() {
        let is_active = app.popup_focus == PopupFocus::Fields && i == app.quick_transfer_field;
        let value = if is_active {
            &app.input_buffer
        } else {
            &app.quick_transfer_buffers[i]
        };

        let label_style = if is_active {
            Style::default().fg(color_at(0)).bold()
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
            Style::default().fg(dim_at(0))
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

        // Show autocomplete suggestions for the recipient field
        if is_active && i == 0 && !app.autocomplete.is_empty() {
            for (j, (alias, addr)) in app.autocomplete.iter().enumerate() {
                let is_sel = app.autocomplete_idx == Some(j);
                let trunc = if addr.len() > 24 {
                    format!("{}…{}", &addr[..10], &addr[addr.len() - 6..])
                } else {
                    addr.clone()
                };
                let style = if is_sel {
                    Style::default().fg(color_at(0)).bold()
                } else {
                    Style::default().fg(dim_at(0))
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

    lines.push(button_line("Send", app.popup_focus, "  Tab: next  "));

    let block = Block::default()
        .title(sparkle_text(" Quick Transfer "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_object_transfer_popup(frame: &mut Frame, app: &App, area: Rect) {
    let obj_label = app
        .filtered_objects()
        .get(app.objects_selected)
        .and_then(|&i| app.objects.get(i))
        .map(|o| {
            format!(
                "Object: {}",
                super::common::truncate_address(&o.object_id, 24)
            )
        })
        .unwrap_or_default();

    let is_active = app.popup_focus == PopupFocus::Fields;
    let value = &app.input_buffer;
    let label_style = if is_active {
        Style::default().fg(color_at(0)).bold()
    } else {
        Style::default().fg(Color::White)
    };
    let input_style = if is_active {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::UNDERLINED)
    } else {
        Style::default().fg(dim_at(0))
    };

    let display = if value.is_empty() && !is_active {
        "(empty)".to_string()
    } else if is_active {
        format!("{}|", value)
    } else {
        value.clone()
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}", obj_label),
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Recipient (address or alias): ",
            label_style,
        )]),
        Line::from(vec![Span::styled(format!("  {}", display), input_style)]),
    ];

    // Autocomplete suggestions
    if is_active && !app.autocomplete.is_empty() {
        for (j, (alias, addr)) in app.autocomplete.iter().enumerate() {
            let is_sel = app.autocomplete_idx == Some(j);
            let trunc = if addr.len() > 24 {
                format!("{}…{}", &addr[..10], &addr[addr.len() - 6..])
            } else {
                addr.clone()
            };
            let style = if is_sel {
                Style::default().fg(color_at(0)).bold()
            } else {
                Style::default().fg(dim_at(0))
            };
            let prefix = if is_sel { "▸ " } else { "  " };
            lines.push(Line::from(vec![Span::styled(
                format!("    {}{} → {}", prefix, alias, trunc),
                style,
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(button_line("Send", app.popup_focus, "  Tab: next  "));

    let block = Block::default()
        .title(sparkle_text(" Transfer Object "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(lines).block(block), area);
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
            Span::styled("  Enter/y", Style::default().fg(color_at(0)).bold()),
            Span::raw(" quit   "),
            Span::styled("Esc/n", Style::default().fg(color_at(0)).bold()),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(sparkle_text(" Confirm Quit "))
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_settings_popup(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::app::SettingsTab;
    use ratatui::layout::{Constraint, Layout};

    let block = Block::default()
        .title(sparkle_text(" Settings "))
        .title_style(Style::default().fg(color_at(0)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(0)));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 4 || inner.width < 20 {
        return;
    }

    let tab_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(4)]).split(inner);

    let mut tab_spans: Vec<Span> = vec![Span::raw("  ")];
    for tab in SettingsTab::ALL {
        let is_active = tab == app.settings_tab;
        let style = if is_active {
            Style::default().fg(Color::Black).bg(color_at(0)).bold()
        } else {
            Style::default().fg(dim_at(0))
        };
        tab_spans.push(Span::styled(format!(" {} ", tab.title()), style));
        tab_spans.push(Span::raw("  "));
    }
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), tab_layout[0]);

    let content_area = tab_layout[1];

    match app.settings_tab {
        SettingsTab::Keys => {
            app.screen = crate::app::Screen::Keys;
            super::keys::draw(frame, app, content_area);
        }
        SettingsTab::AddressBook => {
            app.screen = crate::app::Screen::AddressBook;
            super::address_book::draw(frame, app, content_area);
        }
        SettingsTab::Network => {
            draw_network_settings(frame, app, content_area);
        }
    }
}

fn draw_network_settings(frame: &mut Frame, app: &App, area: Rect) {
    let net_status = if app.connected {
        format!("Connected to {}", app.network_name)
    } else if app.loading {
        format!("Connecting to {} ...", app.network_name)
    } else {
        "Disconnected".to_string()
    };

    let status_color = if app.connected {
        Color::Green
    } else if app.loading {
        Color::Yellow
    } else {
        Color::Red
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Network: ", Style::default().fg(Color::White).bold()),
            Span::styled(&net_status, Style::default().fg(status_color)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Press [n] to switch network",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    frame.render_widget(Paragraph::new(text), area);
}

fn draw_welcome_popup(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  To get started, set up a keypair:",
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    [", Style::default().fg(Color::DarkGray)),
            Span::styled("G", Style::default().fg(color_at(0)).bold()),
            Span::styled(
                "]  Generate a new keypair",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("    [", Style::default().fg(Color::DarkGray)),
            Span::styled("I", Style::default().fg(color_at(0)).bold()),
            Span::styled(
                "]  Import an existing private key",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
    ];

    let block = Block::default()
        .title(sparkle_text(" Welcome to iota-tui! "))
        .title_style(Style::default().fg(color_at(0)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(0)));

    frame.render_widget(Paragraph::new(text).block(block), area);
}
