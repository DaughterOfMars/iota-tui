//! Shared UI components: tab bar, status bar, separator, and reusable helpers.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use std::sync::atomic::{AtomicU32, Ordering};

use crate::app::Screen;

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

/// Return structured hints for each screen.
/// Each entry: (key_label, description, action_id).
/// Empty action_id means not clickable (navigation hints).
pub fn screen_hints(screen: Screen) -> Vec<(&'static str, &'static str, &'static str)> {
    match screen {
        Screen::Coins => vec![
            ("Enter", "explore", "explore"),
            ("t", "type-search", "type_search"),
            ("m", "merge", "merge"),
            ("s", "split", "split"),
            ("x", "transfer", "quick_transfer"),
            ("p", "portfolio", "portfolio"),
            ("/", "search", "filter"),
            ("c", "copy", "copy"),
            ("C", "export", "export"),
            ("f", "faucet", "faucet"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::Objects => vec![
            ("Enter", "explore", "explore"),
            ("t", "type-search", "type_search"),
            ("x", "transfer", "transfer"),
            ("/", "search", "filter"),
            ("c", "copy", "copy"),
            ("C", "export", "export"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::Transactions => vec![
            ("Enter", "explore", "explore"),
            ("/", "search", "filter"),
            ("c", "copy", "copy"),
            ("C", "export", "export"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::Staking => vec![
            ("Enter", "explore", "explore"),
            ("u", "unstake", "unstake"),
            ("c", "copy", "copy"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::Packages => vec![
            ("Enter", "browse", "pkg_browse"),
            ("e", "explore", "explore"),
            ("Esc", "back", "pkg_back"),
            ("c", "copy", "copy"),
            ("r", "refresh", "refresh"),
            ("?", "help", "help"),
        ],
        Screen::AddressBook => vec![
            ("Enter", "explore", "explore"),
            ("a", "add", "addr_add"),
            ("e", "edit", "addr_edit"),
            ("d", "delete", "addr_delete"),
            ("l", "iota-name", "iota_name"),
            ("c", "copy", "copy"),
            ("C", "export", "export"),
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
            ("c", "copy", "copy"),
            ("C", "export", "export"),
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

/// Extract the short type name from a fully-qualified Move type string.
/// e.g. `0x2::iota::IOTA` → `IOTA`, `0xabc::module::MyStruct<0x2::coin::Coin>` → `MyStruct<Coin>`.
pub fn short_type_name(full_type: &str) -> String {
    // Handle generics: extract the outer name and recursively shorten type params
    if let Some(open) = full_type.find('<') {
        let outer = short_segment(&full_type[..open]);
        let inner = &full_type[open + 1..full_type.len().saturating_sub(1)];
        // Split inner type params by comma, shorten each
        let params: Vec<String> = split_type_params(inner)
            .iter()
            .map(|p| short_type_name(p.trim()))
            .collect();
        format!("{}<{}>", outer, params.join(", "))
    } else {
        short_segment(full_type)
    }
}

/// Extract the last `::` segment from a type path (e.g. `0x2::iota::IOTA` → `IOTA`).
fn short_segment(path: &str) -> String {
    path.rsplit("::").next().unwrap_or(path).to_string()
}

/// Split generic type parameters by top-level commas (respecting nested `<>`).
fn split_type_params(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
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
