//! Text input field keyboard handling.

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

/// Handle keystrokes when editing a text input field.
pub fn handle_input_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => app.input_insert(c),
        KeyCode::Backspace => app.input_backspace(),
        KeyCode::Delete => app.input_delete(),
        KeyCode::Left => app.input_left(),
        KeyCode::Right => app.input_right(),
        KeyCode::Home => app.input_cursor = 0,
        KeyCode::End => app.input_cursor = app.input_buffer.len(),
        _ => {}
    }
}
