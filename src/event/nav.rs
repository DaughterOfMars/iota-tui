//! Shared list navigation helper to reduce duplication across screen handlers.

use crossterm::event::KeyCode;

use crate::app::App;

/// Mutable references to a list's navigation state.
pub struct ListNav<'a> {
    pub selected: &'a mut usize,
    pub offset: &'a mut usize,
    pub len: usize,
    pub visible_rows: usize,
}

impl ListNav<'_> {
    /// Handle standard navigation keys. Returns `true` if the key was consumed.
    pub fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Up => {
                if *self.selected > 0 {
                    *self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if *self.selected + 1 < self.len {
                    *self.selected += 1;
                }
            }
            KeyCode::Home => {
                *self.selected = 0;
            }
            KeyCode::End => {
                if self.len > 0 {
                    *self.selected = self.len - 1;
                }
            }
            KeyCode::PageUp => {
                *self.selected = self.selected.saturating_sub(10);
            }
            KeyCode::PageDown => {
                *self.selected = (*self.selected + 10).min(self.len.saturating_sub(1));
            }
            _ => return false,
        }
        App::scroll_into_view(*self.selected, self.offset, self.visible_rows);
        true
    }
}
