use super::*;

use crate::tree_walk::expand_all_ids;

pub(super) fn execute(app: &mut App, parts: &[&str], cmd: &str, raw: &str) {
    let sel = app.tree.selected_path.clone();

    match parts.first().copied() {
        Some("add") | Some("a") => {
            let title = raw
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            if !title.is_empty() {
                app.add_named_task(&title, false);
            }
        }
        Some("due") => {
            if app.task_at(&sel).is_some() {
                app.push_undo();
                let today = Local::now().date_naive();
                let rest = parts[1..].join(" ");
                if let Some(task) = app.task_mut_at(&sel) {
                    task.due = parse_relative_date(&rest, today);
                }
                app.rebuild_visible();
                app.mark_dirty();
                app.message = "Due date set via command".to_string();
            } else {
                app.message = "No task selected for :due".to_string();
            }
        }
        Some("priority") | Some("p") => {
            if app.task_at(&sel).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&sel) {
                    match parts.get(1).copied() {
                        Some("high") => task.priority = Priority::High,
                        Some("med") | Some("medium") => task.priority = Priority::Medium,
                        Some("low") => task.priority = Priority::Low,
                        _ => task.priority = task.priority.cycle(),
                    }
                }
                app.rebuild_visible();
                app.mark_dirty();
            } else {
                app.message = "No task selected for :priority".to_string();
            }
        }
        Some("status") | Some("s") => {
            if app.task_at(&sel).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&sel) {
                    match parts.get(1).copied() {
                        Some("done") => task.status = Status::Done,
                        Some("doing") => task.status = Status::Doing,
                        Some("blocked") => task.status = Status::Blocked,
                        _ => task.status = task.status.cycle(),
                    }
                }
                app.rebuild_visible();
                app.mark_dirty();
            } else {
                app.message = "No task selected for :status".to_string();
            }
        }
        Some("tag") => {
            if app.task_at(&sel).is_some() {
                app.push_undo();
                if let Some(task) = app.task_mut_at(&sel) {
                    for p in &parts[1..] {
                        let t = p
                            .trim_start_matches('+')
                            .trim_start_matches('-')
                            .to_string();
                        if p.starts_with('-') {
                            task.tags.retain(|x| x != &t);
                        } else if !task.tags.contains(&t) {
                            task.tags.push(t);
                        }
                    }
                }
                app.rebuild_visible();
                app.mark_dirty();
            } else {
                app.message = "No task selected for :tag".to_string();
            }
        }
        Some("filter") => {
            if parts.get(1) == Some(&"clear") {
                app.clear_filters();
            } else if parts.get(1) == Some(&"today") || parts.get(1) == Some(&"overdue") {
                app.filter.only_overdue_or_today = true;
                app.rebuild_visible();
            }
        }
        Some("expand") => {
            if parts.get(1) == Some(&"all") {
                expand_all_ids(&app.tasks, &mut app.tree.expanded, 0);
                app.mark_dirty();
                app.rebuild_visible();
                app.message = "Expanded all".to_string();
            }
        }
        Some("sort") => {
            app.sort_mode = match parts.get(1) {
                Some(&"priority") => SortMode::Priority,
                Some(&"due") => SortMode::DueDate,
                Some(&"title") => SortMode::Title,
                _ => SortMode::Manual,
            };
            app.message = format!("Sort: {}", app.sort_mode);
            app.rebuild_visible();
            app.mark_dirty();
        }
        Some("undo") => app.undo(),
        _ => {
            app.message = format!("Unknown command: {cmd}");
        }
    }
}
