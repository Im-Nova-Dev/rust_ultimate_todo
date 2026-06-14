//! Terminal-aware key chord detection shared across input handlers.
//!
//! Terminals often encode the same chord differently (e.g. Ctrl+Enter as Ctrl+M).

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Build a key event the same way the terminal loop consumes (`KeyEventKind::Press`).
pub fn press_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }
}

pub fn is_ctrl_char(key: &KeyEvent, ch: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char(c) if c.eq_ignore_ascii_case(&ch))
}

pub fn is_quit_key(key: &KeyEvent) -> bool {
    is_ctrl_char(key, 'q') || is_ctrl_char(key, 'c')
}

pub fn is_undo_key(key: &KeyEvent) -> bool {
    is_ctrl_char(key, 'z')
}

pub fn is_save_key(key: &KeyEvent) -> bool {
    is_ctrl_char(key, 's')
}

/// Keys that commit the edit modal (handled before any field-specific logic).
pub fn is_edit_save_key(key: &KeyEvent) -> bool {
    if is_save_key(key) {
        return true;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return matches!(
            key.code,
            KeyCode::Enter | KeyCode::Char('m') | KeyCode::Char('M')
        );
    }
    key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Enter
}

pub fn is_edit_cancel_key(key: &KeyEvent) -> bool {
    is_ctrl_char(key, 'c')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_matches_ctrl_c_and_ctrl_q() {
        assert!(is_quit_key(&press_key(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        )));
        assert!(is_quit_key(&press_key(
            KeyCode::Char('C'),
            KeyModifiers::CONTROL
        )));
        assert!(is_quit_key(&press_key(
            KeyCode::Char('q'),
            KeyModifiers::CONTROL
        )));
    }

    #[test]
    fn edit_save_matches_ctrl_enter_aliases() {
        assert!(is_edit_save_key(&press_key(
            KeyCode::Enter,
            KeyModifiers::CONTROL
        )));
        assert!(is_edit_save_key(&press_key(
            KeyCode::Char('m'),
            KeyModifiers::CONTROL
        )));
        assert!(is_edit_save_key(&press_key(
            KeyCode::Char('s'),
            KeyModifiers::CONTROL
        )));
        assert!(!is_edit_save_key(&press_key(
            KeyCode::Enter,
            KeyModifiers::NONE
        )));
    }
}
