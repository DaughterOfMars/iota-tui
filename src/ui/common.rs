//! Shared UI components: tab bar, status bar, separator, and reusable helpers.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::{App, InputMode, Screen};

pub const ACCENT: Color = Color::Cyan;
pub const HIGHLIGHT: Color = Color::Yellow;
pub const DIM: Color = Color::DarkGray;

/// Draw the top tab bar showing all screens.
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

/// Draw a horizontal separator line.
pub fn draw_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new("─".repeat(area.width as usize)).style(Style::default().fg(DIM));
    frame.render_widget(sep, area);
}

/// Draw the bottom status bar with mode, hints, network, and active address.
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
        Screen::TxBuilder => {
            " Left/Right:step  Up/Down:navigate  a:add  c:clear  Enter:confirm  ?:help"
        }
    }
}

// ── Reusable helpers for popup and screen drawing ──────────────────

/// Create a centered rectangle with minimum dimensions.
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

/// Truncate a type string to fit within `max_width`, adding "..." if needed.
pub fn truncate_type(type_str: &str, max_width: usize) -> String {
    if type_str.len() <= max_width {
        return type_str.to_string();
    }
    if max_width < 6 {
        return type_str[..max_width].to_string();
    }
    format!("{}...", &type_str[..max_width.saturating_sub(3)])
}

/// Truncate an address for display, keeping prefix and suffix visible.
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
pub fn clamp_scroll(scroll: &mut usize, content_len: usize, visible: usize) {
    let max = content_len.saturating_sub(visible);
    if *scroll > max {
        *scroll = max;
    }
}

/// Render a scrollbar on the right edge of a popup area.
/// Only draws if content overflows the visible area.
pub fn render_popup_scrollbar(
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
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    frame.render_stateful_widget(scrollbar, inner, &mut state);
}
