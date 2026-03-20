//! Command-form popup renderers (add command type picker and field form).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::{AddCommandType, App};

use super::super::common::{
    clamp_scroll, color_at, dim_at, render_popup_scrollbar, sparkle_text, truncate_address,
};

pub(super) fn draw_add_command_popup(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Select command type:",
            Style::default().bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1/t] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Transfer IOTA"),
        ]),
        Line::from(vec![
            Span::styled("  [2/o] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Transfer Objects"),
        ]),
        Line::from(vec![
            Span::styled("  [3/m] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Move Call"),
        ]),
        Line::from(vec![
            Span::styled("  [4/s] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Split Coins"),
        ]),
        Line::from(vec![
            Span::styled("  [5/r] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Merge Coins"),
        ]),
        Line::from(vec![
            Span::styled("  [6/k] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Stake"),
        ]),
        Line::from(vec![
            Span::styled("  [7/u] ", Style::default().fg(color_at(0)).bold()),
            Span::raw("Unstake"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Esc to cancel",
            Style::default().fg(dim_at(0)),
        )]),
    ];

    let block = Block::default()
        .title(sparkle_text(" Add Command "))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

pub(super) fn draw_add_command_form(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(ct) = app.tx.adding_cmd else {
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
        let is_active = i == app.tx.edit_field;
        let value = if is_active {
            &app.input_buffer
        } else {
            app.tx.edit_buffers.get(i).map(|s| s.as_str()).unwrap_or("")
        };

        let label_style = if is_active {
            Style::default().fg(color_at(0)).bold()
        } else {
            Style::default().fg(Color::White)
        };

        // For multi-value fields, show the count in the label
        if is_multi(i) && !app.tx.multi_values.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                format!("  {} ({} selected): ", field, app.tx.multi_values.len()),
                label_style,
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                format!("  {}: ", field),
                label_style,
            )]));
        }

        // For multi-value fields, show the accumulated items
        if is_multi(i) && !app.tx.multi_values.is_empty() {
            for mv in &app.tx.multi_values {
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
            Style::default().fg(dim_at(0))
        };

        if is_multi(i) {
            // Multi-value: show the input line for adding more
            if is_active {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}|", value),
                    input_style,
                )]));
            } else if app.tx.multi_values.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "  (none)".to_string(),
                    Style::default().fg(dim_at(0)),
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

    if app.tx.is_multi_value_field() {
        lines.push(Line::from(vec![Span::styled(
            "  Tab/Enter: add  Backspace: undo",
            Style::default().fg(dim_at(0)),
        )]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            "  Tab: next field",
            Style::default().fg(dim_at(0)),
        )]));
    }
    lines.push(super::button_line("Add", app.popup_focus, "  "));

    let content_len = lines.len();
    let inner_height = area.height.saturating_sub(2) as usize;
    clamp_scroll(&mut app.popup_scroll, content_len, inner_height);

    let block = Block::default()
        .title(sparkle_text(&format!(" {} ", ct.label())))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.popup_scroll as u16, 0)),
        area,
    );

    render_popup_scrollbar(frame, area, app.popup_scroll, content_len, inner_height);
}
