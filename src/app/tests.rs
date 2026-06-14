use super::*;
use crate::app::stats::{count_all, count_done};
use crate::keys::press_key;
use crate::persist::{TDL_FORMAT, TDL_VERSION, TdlFile, write_project_to_path};
use chrono::Local;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DATA_PATH_ID: AtomicU64 = AtomicU64::new(0);

fn make_test_app(tasks: Vec<Task>) -> App {
    let id = TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed);
    let data_path = std::env::temp_dir().join(format!("rust_tui_unit_test_{id}.tdl"));
    App {
        tasks,
        tree: TreeViewState {
            selected_path: vec![],
            visible: vec![],
            visible_state: ListState::default(),
            expanded: HashSet::new(),
            pending_delete_path: None,
            focus_path: None,
            tree_rect: Rect::default(),
            last_click: None,
        },
        persist: PersistState {
            data_path,
            dirty: false,
            last_change: None,
            last_saved: None,
            last_save_failed: None,
        },
        filter: FilterState::default(),
        sort_mode: SortMode::Manual,
        mode: Mode::Normal,
        search_buf: String::new(),
        command_buf: String::new(),
        edit: None,
        message: String::new(),
        undo_stack: vec![],
        next_id: 100,
        total_tasks: 0,
        done_tasks: 0,
    }
}

fn test_key(code: KeyCode, modifiers: KeyModifiers) -> crossterm::event::KeyEvent {
    press_key(code, modifiers)
}

fn edit_desc_field(app: &mut App) {
    app.tree.selected_path = vec![0];
    app.start_edit();
    let Some(ed) = app.edit.as_mut() else {
        panic!("expected edit state");
    };
    ed.current_field = 1;
    ed.desc_lines = vec!["hello world".to_string()];
    ed.desc_row = 0;
    ed.desc_col = 6;
}

#[test]
fn test_subtask_progress_simple() {
    let mut root = Task::new(1, "root");
    root.status = Status::Todo;
    let mut child1 = Task::new(2, "child1");
    child1.status = Status::Done;
    let mut child2 = Task::new(3, "child2");
    child2.status = Status::Todo;
    root.children.push(child1);
    root.children.push(child2);

    let app = make_test_app(vec![root]);
    let (done, total) = app.subtask_progress(&app.tasks[0]);
    assert_eq!(done, 1);
    assert_eq!(total, 3);
}

#[test]
fn test_find_path_by_id() {
    let mut root = Task::new(10, "root");
    let mut child = Task::new(20, "child");
    let grandchild = Task::new(30, "grand");
    child.children.push(grandchild);
    root.children.push(child);

    let app = make_test_app(vec![root]);

    assert_eq!(app.find_path_by_id(10), Some(vec![0]));
    assert_eq!(app.find_path_by_id(20), Some(vec![0, 0]));
    assert_eq!(app.find_path_by_id(30), Some(vec![0, 0, 0]));
    assert_eq!(app.find_path_by_id(999), None);
}

#[test]
fn test_jump_by_numeric_id() {
    let mut root = Task::new(100, "root");
    let child = Task::new(200, "child");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);

    app.command_buf = "200".to_string();
    app.execute_command();
    assert_eq!(app.tree.selected_path, vec![0, 0]);
}

#[test]
fn test_id_command_done_and_delete() {
    let mut root = Task::new(100, "root");
    let child = Task::new(200, "child");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);

    app.command_buf = "done 200".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].children[0].status, Status::Done);

    app.command_buf = "delete 200".to_string();
    app.execute_command();
    assert!(app.tasks[0].children.is_empty());
    assert!(app.message.contains("Deleted"));
}

#[test]
fn test_id_command_toggle_and_priority() {
    let mut root = Task::new(300, "r2");
    let child = Task::new(400, "c2");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);

    app.command_buf = "toggle 400".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].children[0].status, Status::Doing);

    app.command_buf = "priority 400 high".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].children[0].priority, Priority::High);
}

#[test]
fn test_help_and_stats_commands() {
    let mut app = make_test_app(vec![Task::new(400, "c2")]);

    app.command_buf = "help".to_string();
    app.execute_command();
    assert!(app.message.contains("Cmds:") && app.message.contains("ID actions"));

    app.command_buf = "stats".to_string();
    app.execute_command();
    assert!(app.message.contains("Stats:") && app.message.contains("total"));
}

#[test]
fn test_jump_select_aliases() {
    let mut root = Task::new(300, "r2");
    let child = Task::new(400, "c2");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);

    app.command_buf = "jump 400".to_string();
    app.execute_command();
    assert_eq!(app.tree.selected_path, vec![0, 0]);

    app.command_buf = "select 400".to_string();
    app.execute_command();
    assert_eq!(app.tree.selected_path, vec![0, 0]);
}

#[test]
fn test_export_command() {
    let mut app = make_test_app(vec![Task::new(400, "c2")]);
    app.command_buf = "export".to_string();
    app.execute_command();
    assert!(app.message.contains("Exported markdown") && app.message.contains("Preview"));
}

#[test]
fn test_parent_and_up_commands() {
    let mut root = Task::new(300, "r2");
    let child = Task::new(400, "c2");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);
    app.tree.selected_path = vec![0, 0];

    app.command_buf = "parent".to_string();
    app.execute_command();
    assert_eq!(app.tree.selected_path, vec![0]);

    app.command_buf = "up".to_string();
    app.execute_command();
    assert_eq!(app.tree.selected_path, vec![0]);
}

#[test]
fn test_go_to_parent_key_equivalent() {
    let mut root = Task::new(300, "r2");
    let child = Task::new(400, "c2");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);
    app.tree.selected_path = vec![0, 0];
    app.go_to_parent();
    assert_eq!(app.tree.selected_path, vec![0]);
}

#[test]
fn test_clear_command() {
    let mut app = make_test_app(vec![Task::new(1, "t")]);
    app.filter.hide_done = true;
    app.command_buf = "clear".to_string();
    app.execute_command();
    assert!(!app.filter.hide_done);
}

#[test]
fn test_top_and_bottom_commands() {
    let mut root = Task::new(300, "r2");
    let child = Task::new(400, "c2");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);
    app.tree.expanded.insert(300);
    app.rebuild_visible();

    app.command_buf = "top".to_string();
    app.execute_command();
    assert_eq!(app.tree.selected_path, vec![0]);

    app.command_buf = "bottom".to_string();
    app.execute_command();
    assert_eq!(app.tree.selected_path, vec![0, 0]);
}

#[test]
fn test_duplicate_command() {
    let mut app = make_test_app(vec![Task::new(500, "original")]);
    app.tree.selected_path = vec![0];
    app.command_buf = "duplicate 500".to_string();
    app.execute_command();
    assert_eq!(app.tasks.len(), 2);
    assert_eq!(app.tasks[1].title, "original");
    assert!(app.message.contains("Duplicated"));
}

#[test]
fn test_mark_command_status_and_priority() {
    let mut app = make_test_app(vec![Task::new(600, "task")]);
    app.tree.selected_path = vec![0];

    app.command_buf = "mark 600 doing".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].status, Status::Doing);

    app.command_buf = "mark 600 high".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].priority, Priority::High);
}

#[test]
fn test_duplicate_current_key() {
    let mut app = make_test_app(vec![Task::new(700, "dupme")]);
    app.tree.selected_path = vec![0];
    app.duplicate_current();
    assert_eq!(app.tasks.len(), 2);
    assert_eq!(app.tasks[1].title, "dupme");
    assert!(app.message.contains("Duplicated"));
}

#[test]
fn test_desc_editor_ctrl_shortcuts() {
    let mut app = make_test_app(vec![Task::new(1, "test")]);
    edit_desc_field(&mut app);

    app.edit_handle_key(test_key(KeyCode::Char('k'), KeyModifiers::CONTROL));
    assert_eq!(app.edit.as_ref().unwrap().desc_lines[0], "hello ");

    edit_desc_field(&mut app);
    app.edit_handle_key(test_key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    assert_eq!(app.edit.as_ref().unwrap().desc_lines[0], "world");
    assert_eq!(app.edit.as_ref().unwrap().desc_col, 0);

    edit_desc_field(&mut app);
    app.edit_handle_key(test_key(KeyCode::Char('a'), KeyModifiers::CONTROL));
    assert_eq!(app.edit.as_ref().unwrap().desc_col, 0);

    edit_desc_field(&mut app);
    app.edit_handle_key(test_key(KeyCode::Char('e'), KeyModifiers::CONTROL));
    assert_eq!(app.edit.as_ref().unwrap().desc_col, 11);
}

#[test]
fn test_reload_requires_confirm() {
    let mut app = make_test_app(vec![Task::new(1, "keep me")]);
    app.command_buf = "reload".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].title, "keep me");
    assert!(app.message.contains("reload confirm"));
}

#[test]
fn test_edit_save_shortcuts() {
    let mut app = make_test_app(vec![Task::new(1, "before")]);
    app.tree.selected_path = vec![0];
    app.start_edit();
    app.edit.as_mut().unwrap().title_buf = "via ctrl-enter".to_string();
    app.edit_handle_key(test_key(KeyCode::Enter, KeyModifiers::CONTROL));
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.tasks[0].title, "via ctrl-enter");

    app.start_edit();
    app.edit.as_mut().unwrap().title_buf = "via ctrl-m".to_string();
    app.edit_handle_key(test_key(KeyCode::Char('m'), KeyModifiers::CONTROL));
    assert_eq!(app.tasks[0].title, "via ctrl-m");

    app.start_edit();
    app.edit.as_mut().unwrap().title_buf = "via ctrl-s".to_string();
    app.edit_handle_key(test_key(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(app.tasks[0].title, "via ctrl-s");

    // In the description field, save shortcuts must still win over newline insertion.
    app.start_edit();
    let ed = app.edit.as_mut().unwrap();
    ed.current_field = 1;
    ed.title_buf = "desc save".to_string();
    app.edit_handle_key(test_key(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.tasks[0].title, "desc save");
}

#[test]
fn test_apply_edit_empty_title_prevention() {
    let mut app = make_test_app(vec![Task::new(1, "test")]);
    app.tree.selected_path = vec![0];
    app.start_edit();
    if let Some(mut ed) = app.edit.take() {
        ed.title_buf = "".to_string();
        app.edit = Some(ed);
    }
    app.apply_edit();
    assert!(app.message.contains("empty title"));
    assert!(app.edit.is_some());
}

#[test]
fn test_mark_command_blocked() {
    let mut app = make_test_app(vec![Task::new(700, "task")]);
    app.tree.selected_path = vec![0];
    app.command_buf = "mark 700 blocked".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].status, Status::Blocked);
    assert!(app.message.contains("Marked #700 blocked"));
}

#[test]
fn test_pending_delete_cleared_on_invalid_after_rebuild() {
    let mut root = Task::new(1, "root");
    let child = Task::new(2, "child");
    root.children.push(child);
    let mut app = make_test_app(vec![root]);

    app.tree.pending_delete_path = Some(vec![0, 0]);
    app.tasks[0].children.clear();
    app.rebuild_visible();
    assert!(app.tree.pending_delete_path.is_none());
}

#[test]
fn test_breadcrumb_and_parent_nav() {
    let mut root = Task::new(1, "root");
    let mut child = Task::new(2, "child");
    let grand = Task::new(3, "grand");
    child.children.push(grand);
    root.children.push(child);
    let mut app = make_test_app(vec![root]);

    app.tree.selected_path = vec![0, 0, 0];
    assert_eq!(app.get_breadcrumb(), "#1 > #2 > #3");

    app.go_to_parent();
    assert_eq!(app.tree.selected_path, vec![0, 0]);
    assert_eq!(app.get_breadcrumb(), "#1 > #2");

    app.go_to_parent();
    assert_eq!(app.tree.selected_path, vec![0]);
    assert_eq!(app.get_breadcrumb(), "#1");
}

#[test]
fn test_sanitize_duplicate_ids() {
    let mut root = Task::new(1, "root");
    let child1 = Task::new(2, "c1");
    let child2 = Task::new(2, "c2");
    root.children.push(child1);
    root.children.push(child2);

    let mut app = make_test_app(vec![root]);
    let had_dups = app.sanitize_duplicate_ids();

    assert!(had_dups);
    assert_eq!(app.tasks[0].children[0].id, 2);
    assert_eq!(app.tasks[0].children[1].id, 3);
    assert_eq!(app.next_id, 4);
    assert!(app.persist.dirty);
}

#[test]
fn test_export_markdown() {
    let today = Local::now().date_naive();
    let mut root = Task::new(1, "Root task");
    root.status = Status::Todo;
    root.priority = Priority::High;
    root.due = Some(today);
    root.tags = vec!["work".to_string()];

    let mut child = Task::new(2, "Child done");
    child.status = Status::Done;
    root.children.push(child);

    let app = make_test_app(vec![root]);
    let md = app.export_markdown();

    assert!(md.contains("- [ ] Root task **HIGH** **TODAY** #work"));
    assert!(md.contains("  - [x] Child done"));
    assert!(md.contains("Root task"));
}

#[test]
fn test_duplicate_assigns_fresh_ids_to_subtree() {
    let mut root = Task::new(1, "root");
    root.children.push(Task::new(2, "child"));
    root.children.push(Task::new(3, "child2"));
    let mut app = make_test_app(vec![root]);
    app.tree.selected_path = vec![0];
    app.next_id = 4;

    app.duplicate_current();

    assert_eq!(app.tasks.len(), 2);
    let dup = &app.tasks[1];
    assert_eq!(dup.id, 4);
    assert_eq!(dup.children[0].id, 5);
    assert_eq!(dup.children[1].id, 6);
    assert_eq!(app.next_id, 7);
}

#[test]
fn test_move_task_at_root_level() {
    let mut app = make_test_app(vec![
        Task::new(1, "first"),
        Task::new(2, "second"),
        Task::new(3, "third"),
    ]);
    app.tree.selected_path = vec![0];
    app.rebuild_visible();

    app.move_task(1);
    assert_eq!(app.tasks[0].title, "second");
    assert_eq!(app.tasks[1].title, "first");
    assert_eq!(app.tree.selected_path, vec![1]);
}

#[test]
fn test_outdent_from_depth_one_promotes_to_root() {
    let mut root = Task::new(1, "parent");
    root.children.push(Task::new(2, "child"));
    let mut app = make_test_app(vec![root]);
    app.tree.selected_path = vec![0, 0];

    app.outdent();

    assert_eq!(app.tasks.len(), 2);
    assert_eq!(app.tasks[0].title, "parent");
    assert_eq!(app.tasks[1].title, "child");
    assert_eq!(app.tree.selected_path, vec![1]);
    assert!(app.tasks[0].children.is_empty());
}

#[test]
fn test_id_command_mark_is_undoable() {
    let mut app = make_test_app(vec![Task::new(100, "task")]);
    app.tree.selected_path = vec![0];
    app.command_buf = "mark 100 doing".to_string();
    app.execute_command();
    assert_eq!(app.tasks[0].status, Status::Doing);
    assert_eq!(app.undo_stack.len(), 1);
    app.undo();
    assert_eq!(app.tasks[0].status, Status::Todo);
}

#[test]
fn test_undo_marks_dirty() {
    let mut app = make_test_app(vec![Task::new(100, "task")]);
    app.tree.selected_path = vec![0];
    app.push_undo();
    app.tasks[0].title = "changed".to_string();
    app.persist.dirty = false;

    app.undo();

    assert_eq!(app.tasks[0].title, "task");
    assert!(app.persist.dirty);
}

#[test]
fn test_stress_tree_operations_no_panic() {
    let mut root = Task::new(1, "root");
    for i in 2..=8 {
        root.children.push(Task::new(i, &format!("child {i}")));
    }
    let mut app = make_test_app(vec![root]);
    app.tree.selected_path = vec![0, 2];
    app.rebuild_visible();

    app.outdent();
    app.indent();
    app.move_task(-1);
    app.move_task(1);
    app.sort_mode = SortMode::Priority;
    app.rebuild_visible();
    app.sort_mode = SortMode::Manual;
    app.rebuild_visible();
    app.duplicate_current();
    app.undo();

    assert!(!app.tasks.is_empty());
}

#[test]
fn test_save_keeps_dirty_on_failure() {
    let mut app = make_test_app(vec![Task::new(1, "t")]);
    let blocker = std::env::temp_dir().join("rust_tui_save_blocker");
    let _ = std::fs::write(&blocker, "not a directory");
    app.persist.data_path = blocker.join("tasks.tdl");
    app.mark_dirty();
    app.save();
    assert!(app.persist.dirty);
    let _ = std::fs::remove_file(&blocker);
}

#[test]
fn test_sort_by_priority_reorders_visible_not_data() {
    let mut low = Task::new(1, "low");
    low.priority = Priority::Low;
    let mut high = Task::new(2, "high");
    high.priority = Priority::High;
    let mut app = make_test_app(vec![low, high]);
    app.sort_mode = SortMode::Priority;
    app.rebuild_visible();

    assert_eq!(app.tasks[0].title, "low");
    assert_eq!(app.tasks[1].title, "high");
    assert_eq!(app.tree.visible.len(), 2);
    assert_eq!(
        app.task_at(&app.tree.visible[0].path).unwrap().title,
        "high"
    );
    assert_eq!(app.task_at(&app.tree.visible[1].path).unwrap().title, "low");
}

#[test]
fn test_apply_edit_rejects_invalid_due_date() {
    let mut app = make_test_app(vec![Task::new(1, "test")]);
    app.tree.selected_path = vec![0];
    app.start_edit();
    if let Some(mut ed) = app.edit.take() {
        ed.due_buf = "not-a-date".to_string();
        app.edit = Some(ed);
    }
    app.apply_edit();
    assert!(app.message.contains("Invalid due date"));
    assert!(app.edit.is_some());
    assert_eq!(app.undo_stack.len(), 0);
}

#[test]
fn test_hide_done_filter_without_active_search() {
    let mut done = Task::new(1, "done task");
    done.status = Status::Done;
    let mut todo = Task::new(2, "open task");
    todo.status = Status::Todo;
    let mut app = make_test_app(vec![done, todo]);
    app.filter.hide_done = true;
    app.rebuild_visible();
    assert_eq!(app.tree.visible.len(), 1);
    assert_eq!(
        app.task_at(&app.tree.visible[0].path).unwrap().title,
        "open task"
    );
}

#[test]
fn test_push_undo_does_not_mark_dirty() {
    let mut app = make_test_app(vec![Task::new(1, "t")]);
    app.push_undo();
    assert!(!app.persist.dirty);
}

#[test]
fn test_push_undo_skips_duplicate_snapshot() {
    let mut app = make_test_app(vec![Task::new(1, "t")]);
    app.push_undo();
    app.push_undo();
    assert_eq!(app.undo_stack.len(), 1);
}

#[test]
fn test_undo_stack_trims_to_entry_cap() {
    let mut app = make_test_app(vec![Task::new(1, "t")]);
    for i in 2..=20u64 {
        app.push_undo();
        app.tasks.push(Task::new(i, "x"));
    }
    assert!(app.undo_stack.len() <= 12);
}

#[test]
fn test_undo_clears_edit_mode() {
    let mut app = make_test_app(vec![Task::new(1, "test")]);
    app.tree.selected_path = vec![0];
    app.start_edit();
    app.push_undo();
    app.undo();
    assert!(app.edit.is_none());
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_add_sibling_at_root_level() {
    let mut app = make_test_app(vec![Task::new(1, "first"), Task::new(2, "second")]);
    app.tree.selected_path = vec![0];
    app.rebuild_visible();

    app.add_task(false);
    app.cancel_edit();

    assert_eq!(app.tasks.len(), 3);
    assert_eq!(app.tasks[1].title, "New task");
    assert_eq!(app.tree.selected_path, vec![1]);
}

#[test]
fn test_selection_preserved_when_filtered_out() {
    let mut root = Task::new(1, "visible");
    let mut done_child = Task::new(2, "hidden done");
    done_child.status = Status::Done;
    root.children.push(done_child);
    let mut app = make_test_app(vec![root]);
    app.tree.selected_path = vec![0, 0];
    app.filter.hide_done = true;
    app.rebuild_visible();

    assert_eq!(app.tree.selected_path, vec![0, 0]);
    assert_eq!(app.current_task().unwrap().title, "hidden done");
    assert_eq!(app.find_exact_visible_index(&[0, 0]), None);
    assert!(app.tree.visible_state.selected().is_none());
    assert_eq!(app.find_visible_index_for_path(&[0, 0]), Some(0));
}

#[test]
fn test_load_recovers_from_backup() {
    let id = TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed);
    let data_path = std::env::temp_dir().join(format!("rust_tui_backup_test_{id}.tdl"));
    let backup_path = crate::persist::sidecar_path(&data_path, "bak");

    let _ = write_project_to_path(
        &backup_path,
        &[Task::new(42, "from backup")],
        &[],
        43,
        SortMode::Manual,
    );
    let _ = fs::write(&data_path, "{ this is not valid json");

    let (tasks, _, _, _, msg) = App::load_project(&data_path, MissingFilePolicy::EmptyProject);
    assert_eq!(tasks[0].title, "from backup");
    assert!(msg.unwrap().contains("backup"));

    let _ = fs::remove_file(&data_path);
    let _ = fs::remove_file(&backup_path);
}

#[test]
fn test_tdl_roundtrip_preserves_sort_and_next_id() {
    let path = std::env::temp_dir().join(format!(
        "rust_tui_tdl_roundtrip_{}.tdl",
        TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut root = Task::new(5, "root");
    root.children.push(Task::new(6, "child"));
    let _ = write_project_to_path(&path, &[root], &[5], 7, SortMode::Priority);

    let parsed = crate::persist::parse_project_data(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(parsed.tasks[0].title, "root");
    assert_eq!(parsed.next_id, 7);
    assert_eq!(parsed.sort_mode, SortMode::Priority);
    assert_eq!(parsed.expanded, vec![5]);

    let _ = fs::remove_file(&path);
}

#[test]
fn test_legacy_json_still_loads() {
    let legacy = r#"{"tasks":[{"id":1,"title":"legacy","desc":"","priority":"medium","status":"todo","children":[]}],"expanded":[1]}"#;
    let parsed = crate::persist::parse_project_data(legacy).unwrap();
    assert_eq!(parsed.tasks[0].title, "legacy");
    assert_eq!(parsed.next_id, 2);
    assert_eq!(parsed.sort_mode, SortMode::Manual);
}

#[test]
fn test_portable_missing_file_starts_empty() {
    let path = std::env::temp_dir().join(format!(
        "rust_tui_new_portable_{}.tdl",
        TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_file(&path);
    let app = App::new_portable(path.clone());
    assert!(app.tasks.is_empty());
    assert_eq!(app.next_id, 1);
    assert!(app.persist.dirty);
    let _ = fs::remove_file(&path);
}

#[test]
fn test_open_and_save_as_commands() {
    let id = TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir();
    let first = base.join(format!("rust_tui_open_a_{id}.tdl"));
    let second = base.join(format!("rust_tui_open_b_{id}.tdl"));
    let _ = fs::remove_file(&first);
    let _ = fs::remove_file(&second);

    let mut app = make_test_app(vec![Task::new(1, "portable")]);
    app.persist.data_path = first.clone();
    app.mark_dirty();
    app.save();
    app.tasks[0].title = "changed".to_string();
    app.mark_dirty();
    app.command_buf = format!("save-as {}", second.display());
    app.execute_command();
    assert!(second.exists());
    assert_eq!(app.persist.data_path, second);

    app.tasks[0].title = "changed".to_string();
    app.mark_dirty();
    app.command_buf = format!("open {}", first.display());
    app.execute_command();
    assert!(app.message.contains("Unsaved changes"));

    app.command_buf = format!("open {} confirm", first.display());
    app.execute_command();
    assert_eq!(app.tasks[0].title, "portable");

    let _ = fs::remove_file(&first);
    let _ = fs::remove_file(&second);
}

#[test]
fn test_tdl_file_has_format_tag() {
    let json = serde_json::to_string(&TdlFile {
        format: TDL_FORMAT.to_string(),
        version: TDL_VERSION,
        tasks: vec![Task::new(1, "x")],
        expanded: vec![],
        next_id: Some(2),
        sort_mode: Some(SortMode::Title),
    })
    .unwrap();
    assert!(json.contains(TDL_FORMAT));
    assert!(json.contains("\"version\":1"));
}

#[test]
fn test_add_named_task_at_root() {
    let mut app = make_test_app(vec![Task::new(1, "existing")]);
    app.command_buf = "add Buy milk".to_string();
    app.execute_command();
    assert_eq!(app.tasks.len(), 2);
    assert_eq!(app.tasks[1].title, "Buy milk");
    assert_eq!(app.tree.selected_path, vec![1]);
    assert!(app.edit.is_none());
}

#[test]
fn test_move_task_at_boundary_is_noop() {
    let mut app = make_test_app(vec![Task::new(1, "only")]);
    app.tree.selected_path = vec![0];
    app.rebuild_visible();
    app.persist.dirty = false;
    app.move_task(-1);
    assert!(!app.persist.dirty);
    assert_eq!(app.undo_stack.len(), 0);
}

#[test]
fn test_hide_done_shows_incomplete_child_of_done_parent() {
    let mut done_parent = Task::new(1, "done parent");
    done_parent.status = Status::Done;
    let mut open_child = Task::new(2, "still open");
    open_child.status = Status::Todo;
    done_parent.children.push(open_child);
    let mut app = make_test_app(vec![done_parent]);
    app.tree.expanded.insert(1);
    app.filter.hide_done = true;
    app.rebuild_visible();
    assert_eq!(app.tree.visible.len(), 1);
    assert_eq!(
        app.task_at(&app.tree.visible[0].path).unwrap().title,
        "still open"
    );
}

#[test]
fn test_corrupt_existing_file_does_not_load_sample_data() {
    let id = TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed);
    let data_path = std::env::temp_dir().join(format!("rust_tui_corrupt_{id}.tdl"));
    let _ = fs::write(&data_path, "not valid json {{{");
    let (tasks, _, _, _, msg) = App::load_project(&data_path, MissingFilePolicy::SampleData);
    assert!(tasks.is_empty());
    assert!(msg.unwrap().contains("Could not load"));
    let _ = fs::remove_file(&data_path);
    let corrupt = crate::persist::sidecar_path(&data_path, "corrupt");
    let _ = fs::remove_file(&corrupt);
}

#[test]
fn test_delete_with_stale_selection() {
    let mut app = make_test_app(vec![Task::new(1, "gone")]);
    app.tree.selected_path = vec![99];
    app.delete_current();
    assert!(app.message.contains("Nothing to delete"));
    assert_eq!(app.undo_stack.len(), 0);
}

#[test]
fn test_undo_restores_sort_mode() {
    let mut app = make_test_app(vec![Task::new(1, "t")]);
    app.tree.selected_path = vec![0];
    app.sort_mode = SortMode::Manual;
    app.push_undo();
    app.sort_mode = SortMode::Priority;
    app.rebuild_visible();
    app.undo();
    assert_eq!(app.sort_mode, SortMode::Manual);
}

#[test]
fn test_save_on_exit_persists_dirty_project() {
    let id = TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("rust_tui_exit_save_{id}.tdl"));
    let _ = fs::remove_file(&path);
    let mut app = make_test_app(vec![Task::new(1, "persist me")]);
    app.persist.data_path = path.clone();
    app.mark_dirty();
    assert!(app.save_on_exit());
    assert!(!app.persist.dirty);
    let loaded = crate::persist::parse_project_data(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(loaded.tasks[0].title, "persist me");
    let _ = fs::remove_file(&path);
}

#[test]
fn test_save_as_preserves_path_casing() {
    let id = TEST_DATA_PATH_ID.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir();
    let dest = base.join(format!("RustTUI_CaseTest_{id}.tdl"));
    let _ = fs::remove_file(&dest);
    let mut app = make_test_app(vec![Task::new(1, "cased")]);
    app.mark_dirty();
    app.command_buf = format!("save-as {}", dest.display());
    app.execute_command();
    assert!(dest.exists());
    assert_eq!(app.persist.data_path, dest);
    let _ = fs::remove_file(&dest);
}

#[test]
fn test_count_all_and_done() {
    let mut tasks = vec![Task::new(1, "t1")];
    tasks[0].status = Status::Done;
    let mut child = Task::new(2, "c1");
    child.status = Status::Todo;
    tasks[0].children.push(child);
    assert_eq!(count_all(&tasks), 2);
    assert_eq!(count_done(&tasks), 1);
}
