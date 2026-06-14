//! Keyboard input routing by application mode.
//!
//! Each mode has an isolated handler so bindings do not leak between modes.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Mode};
use crate::keys::{is_quit_key, is_save_key, is_undo_key};
use crate::sort::SortMode;

pub(crate) fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.mode {
        Mode::Editing => {
            app.edit_handle_key(key);
            false
        }
        Mode::Help => handle_help_key(app, key),
        Mode::Search => handle_search_key(app, key),
        Mode::Command => handle_command_key(app, key),
        Mode::Normal => handle_normal_key(app, key),
    }
}

fn handle_help_key(app: &mut App, key: KeyEvent) -> bool {
    if is_quit_key(&key) {
        return true;
    }
    if matches!(
        key.code,
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Char('Q')
    ) {
        app.mode = Mode::Normal;
    }
    false
}

fn handle_search_key(app: &mut App, key: KeyEvent) -> bool {
    if is_quit_key(&key) {
        app.search_buf.clear();
        app.end_search();
        return false;
    }

    match key.code {
        KeyCode::Esc => {
            app.search_buf.clear();
            app.end_search();
        }
        KeyCode::Enter => app.end_search(),
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.search_buf.push(c);
            app.update_search();
        }
        KeyCode::Backspace => {
            app.search_buf.pop();
            app.update_search();
        }
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        _ => {}
    }
    false
}

fn handle_command_key(app: &mut App, key: KeyEvent) -> bool {
    if is_quit_key(&key) {
        app.command_buf.clear();
        app.mode = Mode::Normal;
        return false;
    }

    match key.code {
        KeyCode::Esc => {
            app.command_buf.clear();
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => app.execute_command(),
        KeyCode::Backspace => {
            app.command_buf.pop();
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.command_buf.push(c);
        }
        _ => {}
    }
    false
}

fn handle_normal_key(app: &mut App, key: KeyEvent) -> bool {
    if is_quit_key(&key) {
        return true;
    }

    if !matches!(key.code, KeyCode::Char('d') | KeyCode::Char('D')) {
        app.tree.pending_delete_path = None;
    }

    match key.code {
        KeyCode::Char('?') => app.mode = Mode::Help,
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
            app.command_buf.clear();
        }
        KeyCode::Char('/') => app.start_search(),
        KeyCode::Esc if !app.filter.search.is_empty() || !app.search_buf.is_empty() => {
            app.clear_filters();
        }
        KeyCode::Char('j') | KeyCode::Down => app.move_selection(1),
        KeyCode::Char('k') | KeyCode::Up => app.move_selection(-1),
        KeyCode::PageDown => app.move_selection(10),
        KeyCode::PageUp => app.move_selection(-10),
        KeyCode::Char('g') => {
            app.tree.visible_state.select(Some(0));
            app.sync_selection_from_visible();
        }
        KeyCode::Char('G') if !app.tree.visible.is_empty() => {
            app.tree
                .visible_state
                .select(Some(app.tree.visible.len() - 1));
            app.sync_selection_from_visible();
        }
        KeyCode::Char('h')
        | KeyCode::Left
        | KeyCode::Char('l')
        | KeyCode::Right
        | KeyCode::Char(' ') => {
            app.toggle_expanded();
        }
        KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::Char('u') | KeyCode::Char('U') => {
            app.go_to_parent();
        }
        KeyCode::Char('0') => app.go_to_top(),
        KeyCode::Char('a') => app.add_task(false),
        KeyCode::Char('A') => app.add_task(true),
        KeyCode::Char('c') | KeyCode::Char('C')
            if !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.duplicate_current();
        }
        KeyCode::Char('d') => {
            let sel_path = app.tree.selected_path.clone();
            let current_info = app.current_task().map(|t| (t.id, t.title.clone()));
            if app.tree.pending_delete_path.as_ref() == Some(&sel_path) {
                app.delete_current();
                app.tree.pending_delete_path = None;
            } else if let Some((id, title)) = current_info {
                app.tree.pending_delete_path = Some(sel_path);
                app.message = format!("Press 'd' again to confirm delete #{id}: {title}");
            }
        }
        KeyCode::Char('D') => {
            app.tree.pending_delete_path = None;
            app.delete_current();
        }
        KeyCode::Char('J') => app.move_task(1),
        KeyCode::Char('K') => app.move_task(-1),
        KeyCode::Char('>') | KeyCode::Char('L') => app.indent(),
        KeyCode::Char('<') | KeyCode::Char('H') => app.outdent(),
        KeyCode::Char('p') | KeyCode::Char('P') => app.cycle_priority(),
        KeyCode::Char('m') | KeyCode::Char('M') => app.cycle_status(),
        KeyCode::Char('x') | KeyCode::Char('X') => app.toggle_done(),
        KeyCode::Char('e') | KeyCode::Enter => app.start_edit(),
        KeyCode::Char('f') => app.cycle_quick_filter(),
        KeyCode::Char('F') => app.clear_filters(),
        _ if is_save_key(&key) => app.save_explicit(),
        KeyCode::Char('s') => {
            app.sort_mode = match app.sort_mode {
                SortMode::Manual => SortMode::Priority,
                SortMode::Priority => SortMode::DueDate,
                SortMode::DueDate => SortMode::Title,
                SortMode::Title => SortMode::Manual,
            };
            app.message = format!("Sort mode: {}", app.sort_mode);
            app.rebuild_visible();
            app.mark_dirty();
        }
        _ if is_undo_key(&key) => app.undo(),
        KeyCode::Char('t') => {
            let p = app.tree.selected_path.clone();
            if let Some(task) = app.task_at(&p) {
                let tag = "quick".to_string();
                if !task.tags.contains(&tag) {
                    app.push_undo();
                    if let Some(task) = app.task_mut_at(&p) {
                        task.tags.push(tag);
                    }
                    app.rebuild_visible();
                    app.mark_dirty();
                }
            }
        }
        _ => {}
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{FilterState, Mode, PersistState, TreeViewState};
    use crate::keys::press_key;
    use ratatui::layout::Rect;
    use ratatui::widgets::ListState;
    use std::collections::HashSet;

    fn make_event_test_app() -> App {
        App {
            tasks: vec![Task::new(1, "one"), Task::new(2, "two")],
            tree: TreeViewState {
                selected_path: vec![0],
                visible: vec![],
                visible_state: ListState::default(),
                expanded: HashSet::new(),
                pending_delete_path: None,
                focus_path: None,
                tree_rect: Rect::default(),
                last_click: None,
            },
            persist: PersistState {
                data_path: std::path::PathBuf::from("/tmp/test.tdl"),
                dirty: false,
                last_change: None,
                last_saved: None,
                last_save_failed: None,
            },
            filter: FilterState::default(),
            sort_mode: SortMode::Manual,
            total_tasks: 2,
            done_tasks: 0,
            mode: Mode::Normal,
            search_buf: String::new(),
            command_buf: String::new(),
            edit: None,
            message: String::new(),
            undo_stack: vec![],
            next_id: 3,
        }
    }

    use crate::app::App;
    use crate::model::Task;

    #[test]
    fn search_enter_does_not_start_edit() {
        let mut app = make_event_test_app();
        app.start_search();
        app.search_buf = "one".into();
        handle_key(&mut app, press_key(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.filter.search, "one");
        assert!(app.edit.is_none());
    }

    #[test]
    fn command_j_does_not_move_selection() {
        let mut app = make_event_test_app();
        app.mode = Mode::Command;
        handle_key(&mut app, press_key(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(app.command_buf, "j");
        assert_eq!(app.tree.selected_path, vec![0]);
    }

    #[test]
    fn help_j_does_not_move_selection() {
        let mut app = make_event_test_app();
        app.mode = Mode::Help;
        handle_key(&mut app, press_key(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(app.mode, Mode::Help);
        assert_eq!(app.tree.selected_path, vec![0]);
    }

    #[test]
    fn search_ctrl_c_cancels_without_quitting() {
        let mut app = make_event_test_app();
        app.start_search();
        app.search_buf = "x".into();
        let quit = handle_key(
            &mut app,
            press_key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(!quit);
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.search_buf.is_empty());
    }

    #[test]
    fn command_ctrl_c_cancels_without_quitting() {
        let mut app = make_event_test_app();
        app.mode = Mode::Command;
        app.command_buf = ":add".into();
        let quit = handle_key(
            &mut app,
            press_key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(!quit);
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.command_buf.is_empty());
    }

    #[test]
    fn normal_ctrl_z_undoes() {
        let mut app = make_event_test_app();
        app.push_undo();
        app.tasks[0].title = "changed".into();
        handle_key(
            &mut app,
            press_key(KeyCode::Char('Z'), KeyModifiers::CONTROL),
        );
        assert_eq!(app.tasks[0].title, "one");
    }
}
