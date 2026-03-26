//! Context menu — renders an action dropdown anchored at the cursor position.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common;
use crate::app::App;

/// Draw the context menu if active.
pub fn draw(frame: &mut Frame, app: &App) {
    let Some(ref menu) = app.context_menu else {
        return;
    };

    if menu.actions.is_empty() {
        return;
    }

    let menu_width = menu
        .actions
        .iter()
        .map(|a| a.label().len() + 6) // " [x] Label "
        .max()
        .unwrap_or(12) as u16
        + 2; // borders

    let menu_height = menu.actions.len() as u16 + 2; // borders

    let frame_area = frame.area();

    // Position: try below-right of anchor, clamp to screen
    let x = menu
        .anchor_col
        .min(frame_area.width.saturating_sub(menu_width));
    let y = if menu.anchor_row + 1 + menu_height <= frame_area.height {
        menu.anchor_row + 1
    } else {
        menu.anchor_row.saturating_sub(menu_height)
    };

    let area = Rect::new(
        x,
        y,
        menu_width.min(frame_area.width),
        menu_height.min(frame_area.height),
    );

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(common::color_at(0)));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    for (i, action) in menu.actions.iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.y + inner.height {
            break;
        }

        let is_selected = i == menu.selected;
        let shortcut = action.shortcut();
        let label = action.label();

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(common::color_at(0))
                .bold()
        } else {
            Style::default().fg(Color::White)
        };

        let line = Line::from(vec![
            Span::styled(
                format!(" [{}] ", shortcut),
                if is_selected {
                    style
                } else {
                    Style::default().fg(common::color_at(0))
                },
            ),
            Span::styled(
                format!(
                    "{:<width$}",
                    label,
                    width = (inner.width as usize).saturating_sub(6)
                ),
                style,
            ),
        ]);
        frame.render_widget(
            Paragraph::new(line),
            Rect::new(inner.x, row_y, inner.width, 1),
        );
    }
}
