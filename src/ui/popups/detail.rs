//! Detail popup renderer.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::App;

use super::super::common::{
    accent_style, clamp_scroll, color_at, dim_at, render_popup_scrollbar, sparkle_text,
};

pub(super) fn draw_detail_popup(frame: &mut Frame, app: &mut App, area: Rect) {
    let (title, fields) = app.detail_info();

    let mut lines: Vec<Line> = vec![Line::from("")];
    if fields.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No item selected",
            Style::default().fg(dim_at(0)),
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
                    Style::default().fg(dim_at(0)),
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
        Style::default().fg(dim_at(0)),
    )]));

    let content_len = lines.len();
    let inner_height = area.height.saturating_sub(2) as usize;
    clamp_scroll(&mut app.popup_scroll, content_len, inner_height);

    let block = Block::default()
        .title(sparkle_text(&format!(" {} ", title)))
        .title_style(Style::default().fg(color_at(1)).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_at(2)));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.popup_scroll as u16, 0));
    frame.render_widget(paragraph, area);

    render_popup_scrollbar(frame, area, app.popup_scroll, content_len, inner_height);
}
