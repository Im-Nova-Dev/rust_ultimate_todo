use super::*;

pub(super) fn try_execute(app: &mut App, parts: &[&str], raw: &str) -> bool {
    if parts.is_empty() {
        return false;
    }

    match parts[0] {
        "help" => {
            app.message = "Cmds: add/due/priority/status/tag/filter/expand/sort/undo ; ID actions: delete/done/edit/priority/due/toggle <id> ; :42 or :jump/:select/:go 42 jumps ; :open/:save-as <path.tdl> [confirm] ; :parent/:up/:back/:b :top :bottom :clear :export :stats :help :duplicate/:copy <id> :mark <id> <status/priority> :focus <id> :unfocus :reload confirm. See ? for keys.".to_string();
            true
        }
        "stats" => {
            let overdue = app.count_overdue();
            app.message = format!(
                "Stats: {} total, {} done, {} overdue",
                app.total_tasks, app.done_tasks, overdue
            );
            true
        }
        "export" => {
            let md = app.export_markdown();
            app.write_export_markdown(&md);
            let preview: String = md.lines().take(5).collect::<Vec<_>>().join("\n");
            app.message = format!(
                "Exported markdown ({} chars). Preview:\n{preview}",
                md.len()
            );
            true
        }
        "open" => {
            if let Some((path, confirm)) = App::parse_open_path_raw(raw) {
                app.open_project_file(path, confirm);
            } else {
                app.message =
                    "Usage: :open <path.tdl> [confirm]  — e.g. :open ~/projects/foo.tdl confirm"
                        .into();
            }
            true
        }
        "save-as" | "saveas" => {
            if let Some(path) = App::parse_save_as_path_raw(raw) {
                app.save_project_as(path);
            } else {
                app.message =
                    "Usage: :save-as <path.tdl>  — e.g. :save-as ~/projects/foo.tdl".into();
            }
            true
        }
        "parent" | "up" | "back" | "b" => {
            app.go_to_parent();
            app.message = "Went to parent".to_string();
            true
        }
        "clear" => {
            app.clear_filters();
            app.message = "Filters cleared".to_string();
            true
        }
        "top" => {
            app.go_to_top();
            app.message = "Went to top".to_string();
            true
        }
        "bottom" => {
            app.go_to_bottom();
            app.message = "Went to bottom".to_string();
            true
        }
        "focus" if parts.len() >= 2 => {
            if let Ok(id) = parts[1].parse::<u64>() {
                app.focus_on(id);
            }
            true
        }
        "unfocus" => {
            app.unfocus();
            true
        }
        "reload" if parts.len() >= 2 && parts[1] == "confirm" => {
            app.cancel_edit();
            let path = app.persist.data_path.clone();
            let (tasks, next_id, expanded, sort_mode, load_msg) =
                App::load_project(&path, MissingFilePolicy::EmptyProject);
            app.apply_loaded_project(LoadedProject {
                tasks,
                next_id,
                expanded: expanded.into_iter().collect(),
                sort_mode,
            });
            if app.sanitize_duplicate_ids() {
                app.message = "Reloaded from disk; duplicate IDs were renumbered.".to_string();
            } else if let Some(msg) = load_msg {
                app.message = msg;
                app.mark_dirty();
            } else {
                app.message = "Reloaded from disk".to_string();
                app.persist.dirty = false;
                app.persist.last_saved = Some(std::time::Instant::now());
            }
            true
        }
        "reload" => {
            app.message =
                "Type :reload confirm to reload from disk (discards unsaved changes)".into();
            true
        }
        _ => false,
    }
}
