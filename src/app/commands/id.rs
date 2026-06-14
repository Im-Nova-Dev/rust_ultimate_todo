use super::*;

pub(super) fn try_execute(app: &mut App, parts: &[&str]) -> bool {
    if parts.len() < 2 {
        return false;
    }
    let Ok(id) = parts[1].parse::<u64>() else {
        return false;
    };

    let Some(path) = app.find_path_by_id(id) else {
        app.message = format!("No task #{id}");
        return true;
    };

    app.ensure_path_visible(&path);
    app.tree.selected_path = path;

    match parts[0] {
        "delete" | "del" | "rm" => {
            app.delete_current();
            app.message = format!("Deleted #{id}");
        }
        "done" => {
            let path_for_action = app.tree.selected_path.clone();
            if app.task_at(&path_for_action).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&path_for_action) {
                    task.status = Status::Done;
                }
                app.rebuild_visible();
                app.mark_dirty();
                app.clear_pending_delete();
                app.message = format!("Marked #{id} done");
            }
        }
        "edit" => {
            app.clear_pending_delete();
            app.start_edit();
        }
        "priority" if parts.len() >= 3 => {
            let prio_str = parts[2];
            let path_for_action = app.tree.selected_path.clone();
            if app.task_at(&path_for_action).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&path_for_action) {
                    task.priority = match prio_str {
                        "high" => Priority::High,
                        "med" | "medium" => Priority::Medium,
                        "low" => Priority::Low,
                        _ => task.priority.clone(),
                    };
                }
                app.rebuild_visible();
                app.mark_dirty();
                app.clear_pending_delete();
                app.message = format!("Set priority on #{id} to {prio_str}");
            }
        }
        "due" if parts.len() >= 3 => {
            let due_str = parts[2..].join(" ");
            let today = Local::now().date_naive();
            let path_for_action = app.tree.selected_path.clone();
            if app.task_at(&path_for_action).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&path_for_action) {
                    task.due = parse_relative_date(&due_str, today);
                }
                app.rebuild_visible();
                app.mark_dirty();
                app.clear_pending_delete();
                app.message = format!("Set due on #{id}");
            }
        }
        "toggle" => {
            let path_for_action = app.tree.selected_path.clone();
            if app.task_at(&path_for_action).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&path_for_action) {
                    task.status = task.status.cycle();
                }
                app.rebuild_visible();
                app.mark_dirty();
                app.clear_pending_delete();
                app.message = format!("Toggled status on #{id}");
            }
        }
        "mark" if parts.len() >= 3 => {
            let arg = parts[2];
            let path_for_action = app.tree.selected_path.clone();
            let known = matches!(
                arg,
                "done" | "todo" | "doing" | "blocked" | "high" | "med" | "medium" | "low"
            );
            if !known {
                app.message = format!("Unknown mark value: {arg}");
            } else if app.task_at(&path_for_action).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&path_for_action) {
                    match arg {
                        "done" => task.status = Status::Done,
                        "todo" => task.status = Status::Todo,
                        "doing" => task.status = Status::Doing,
                        "blocked" => task.status = Status::Blocked,
                        "high" => task.priority = Priority::High,
                        "med" | "medium" => task.priority = Priority::Medium,
                        "low" => task.priority = Priority::Low,
                        _ => {}
                    }
                }
                app.rebuild_visible();
                app.mark_dirty();
                app.clear_pending_delete();
                app.message = format!("Marked #{id} {arg}");
            }
        }
        "duplicate" | "copy" | "dup" => {
            app.duplicate_current();
        }
        _ => return false,
    }
    true
}
