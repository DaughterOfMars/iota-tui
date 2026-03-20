//! Shared UI components: tab bar, status bar, separator, and reusable helpers.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use std::sync::atomic::{AtomicU32, Ordering};

use crate::app::{App, Screen};

pub const ACCENT: Color = Color::Cyan;
pub const DIM: Color = Color::DarkGray;

static COLOR_PHASE: AtomicU32 = AtomicU32::new(0);

pub fn sync_color_phase(phase: u32) {
    COLOR_PHASE.store(phase, Ordering::Relaxed);
}

const PALETTE: [Color; 6] = [
    Color::Rgb(255, 182, 193), // light pink
    Color::Rgb(200, 170, 255), // lavender
    Color::Rgb(160, 238, 200), // mint
    Color::Rgb(255, 218, 170), // peach
    Color::Rgb(150, 200, 255), // sky blue
    Color::Rgb(255, 245, 170), // soft yellow
];

const SPARKLES: [char; 4] = ['✦', '✧', '⋆', '˚'];

pub fn color_at(offset: u32) -> Color {
    let phase = COLOR_PHASE.load(Ordering::Relaxed);
    if phase == 0 {
        return ACCENT;
    }
    PALETTE[((phase / 3 + offset) as usize) % PALETTE.len()]
}

pub fn dim_at(offset: u32) -> Color {
    let phase = COLOR_PHASE.load(Ordering::Relaxed);
    if phase == 0 {
        return DIM;
    }
    PALETTE[((phase / 4 + offset + 3) as usize) % PALETTE.len()]
}

pub fn sparkle_text(text: &str) -> String {
    let phase = COLOR_PHASE.load(Ordering::Relaxed);
    if phase == 0 {
        return text.to_string();
    }
    let idx = (phase / 5) as usize;
    let s = SPARKLES[idx % SPARKLES.len()];
    format!("{s} {text} {s}")
}

/// Width of the sidebar in expanded mode.
pub const SIDEBAR_WIDTH: u16 = 18;
/// Width of the sidebar in collapsed mode (just number + padding).
pub const SIDEBAR_COLLAPSED_WIDTH: u16 = 4;

/// Returns the current sidebar width based on collapsed state.
pub fn sidebar_width(collapsed: bool) -> u16 {
    if collapsed {
        SIDEBAR_COLLAPSED_WIDTH
    } else {
        SIDEBAR_WIDTH
    }
}

/// Draw the left sidebar showing all screens.
pub fn draw_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    app.sidebar_areas.clear();

    let phase = COLOR_PHASE.load(Ordering::Relaxed);

    // Draw a vertical separator on the right edge
    for y in area.y..area.y + area.height {
        let sep_style = if phase > 0 {
            Style::default().fg(PALETTE[((phase / 3) as usize + y as usize / 2) % PALETTE.len()])
        } else {
            Style::default().fg(DIM)
        };
        frame.render_widget(
            Paragraph::new("│").style(sep_style),
            Rect::new(area.x + area.width - 1, y, 1, 1),
        );
    }

    let inner_width = area.width.saturating_sub(1); // exclude separator column

    for (i, screen) in Screen::ALL.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }

        let row_area = Rect::new(area.x, y, inner_width, 1);
        app.sidebar_areas.push(row_area);

        let is_active = *screen == app.screen;
        let num = if i < 9 {
            format!("{}", i + 1)
        } else {
            "0".to_string()
        };

        let label = if app.sidebar_collapsed {
            format!(" {} ", num)
        } else {
            format!(
                " {} {:<width$}",
                num,
                screen.title(),
                width = (inner_width as usize).saturating_sub(4)
            )
        };

        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(color_at(i as u32))
                .bold()
        } else {
            Style::default().fg(dim_at(i as u32))
        };

        // Truncate label to fit
        let display: String = label.chars().take(inner_width as usize).collect();
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(display, style))),
            row_area,
        );
    }
}

/// Draw the bottom status bar: network (left), actions button + active address (right).
pub fn draw_status_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    app.hint_areas.clear();

    let active_addr = app
        .active_key()
        .map(|k| format!("{}..{}", &k.address[..8], &k.address[k.address.len() - 6..]))
        .unwrap_or_else(|| "No active key".into());

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

    let button_text = " [. Actions] ";
    let addr_tag = " addr ";
    let addr_text = format!(" {} ", active_addr);
    let right_width = button_text.len() as u16 + addr_tag.len() as u16 + addr_text.len() as u16 + 1;

    let cols = Layout::horizontal([
        Constraint::Length(net_indicator.width() as u16),
        Constraint::Min(0),
        Constraint::Length(right_width),
    ])
    .split(area);

    // Network indicator (left) — clickable to switch network
    app.hint_areas.push((cols[0], "network"));
    frame.render_widget(Paragraph::new(Line::from(vec![net_indicator])), cols[0]);

    // Actions button + active address (right)
    let right_area = cols[2];
    let button_x = right_area.x;
    let button_width = button_text.len() as u16;
    app.hint_areas.push((
        Rect::new(button_x, right_area.y, button_width, 1),
        "open_menu",
    ));

    let right_line = Line::from(vec![
        Span::styled(button_text, Style::default().fg(color_at(0)).bold()),
        Span::raw(" "),
        Span::styled(
            " addr ",
            Style::default().fg(Color::Black).bg(color_at(1)).bold(),
        ),
        Span::styled(addr_text, Style::default().fg(color_at(2))),
    ]);
    frame.render_widget(
        Paragraph::new(right_line).alignment(Alignment::Right),
        right_area,
    );
}

/// Return structured hints for each screen.
/// Each entry: (key_label, description, action_id).
/// Empty action_id means not clickable (navigation hints).
pub fn screen_hints(screen: Screen) -> Vec<(&'static str, &'static str, &'static str)> {
    match screen {
        Screen::Coins => vec![
            ("Enter", "explore", "explore"),
            ("t", "type-search", "type_search"),
            ("f", "faucet", "faucet"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::Objects => vec![
            ("Enter", "explore", "explore"),
            ("t", "type-search", "type_search"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::Transactions => vec![
            ("Enter", "explore", "explore"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::Packages => vec![
            ("Enter", "explore", "explore"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::AddressBook => vec![
            ("Enter", "explore", "explore"),
            ("a", "add", "addr_add"),
            ("e", "edit", "addr_edit"),
            ("d", "delete", "addr_delete"),
            ("l", "iota-name", "iota_name"),
            ("?", "help", "help"),
        ],
        Screen::Keys => vec![
            ("Enter", "explore", "explore"),
            ("a", "activate", "key_activate"),
            ("Sp", "visible", "key_visible"),
            ("g", "gen", "key_gen"),
            ("i", "import", "key_import"),
            ("e", "rename", "key_rename"),
            ("d", "delete", "key_delete"),
            ("?", "help", "help"),
        ],
        Screen::TxBuilder => vec![
            ("a", "add", "tx_add"),
            ("d", "delete", "tx_delete"),
            ("c", "clear", "tx_clear"),
            ("?", "help", "help"),
        ],
        Screen::Explorer => vec![
            ("Enter", "search", "explore"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
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
    Style::default()
        .bg(Color::Indexed(236))
        .fg(color_at(0))
        .bold()
}

pub fn header_style() -> Style {
    Style::default().fg(color_at(1)).bold()
}

pub fn dim_style() -> Style {
    Style::default().fg(dim_at(0))
}

pub fn accent_style() -> Style {
    Style::default().fg(color_at(2))
}

/// Create a detail line with a fixed-width label and styled value.
pub fn detail_line(label: &str, value: &str, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<12}", label),
            Style::default().fg(Color::White).bold(),
        ),
        Span::styled(value.to_string(), value_style),
    ])
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
        .thumb_style(Style::default().fg(color_at(0)))
        .track_style(Style::default().fg(dim_at(0)));
    let mut state = ScrollbarState::new(content_len.saturating_sub(visible)).position(scroll);
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    frame.render_stateful_widget(scrollbar, inner, &mut state);
}
