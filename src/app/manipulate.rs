//! Task tree structural edits: indent, move, add, delete, and quick field cycles.

use super::*;

impl App {
    pub(crate) fn indent(&mut self) {
        let last = match self.tree.selected_path.last().copied() {
            Some(l) if l > 0 => l,
            _ => {
                self.message = "Cannot indent (no previous sibling or no selection)".to_string();
                return;
            }
        };

        let mut parent_path = self.tree.selected_path.clone();
        parent_path.pop();

        let can_indent = if parent_path.is_empty() {
            last < self.tasks.len()
        } else if let Some(parent) = self.task_at(&parent_path) {
            last < parent.children.len() && last > 0
        } else {
            false
        };

        if !can_indent {
            self.message = "Cannot indent (invalid selection)".to_string();
            return;
        }

        self.push_undo();
        self.clear_pending_delete();

        if parent_path.is_empty() {
            let task = self.tasks.remove(last);
            let prev_idx = last - 1;
            let parent_id = self.tasks[prev_idx].id;
            self.tasks[prev_idx].children.push(task);
            let child_idx = self.tasks[prev_idx].children.len() - 1;
            self.tree.selected_path = vec![prev_idx, child_idx];
            self.tree.expanded.insert(parent_id);
        } else if let Some(parent) = self.task_mut_at(&parent_path) {
            let task = parent.children.remove(last);
            let prev_idx = last - 1;
            parent.children[prev_idx].children.push(task);
            let mut new_path = parent_path.clone();
            new_path.push(prev_idx);
            new_path.push(parent.children[prev_idx].children.len() - 1);
            self.tree.selected_path = new_path;
        }

        self.rebuild_visible();
        self.mark_dirty();
        self.message = "Indented task".to_string();
    }

    pub(crate) fn outdent(&mut self) {
        if self.tree.selected_path.len() <= 1 {
            self.message = "Cannot outdent (already at top level)".to_string();
            return;
        }

        let child_idx = match self.tree.selected_path.last().copied() {
            Some(i) => i,
            None => return,
        };
        let mut parent_path = self.tree.selected_path.clone();
        parent_path.pop();
        let parent_idx = match parent_path.last().copied() {
            Some(i) => i,
            None => return,
        };
        parent_path.pop();

        let can_outdent = if parent_path.is_empty() {
            parent_idx < self.tasks.len() && child_idx < self.tasks[parent_idx].children.len()
        } else if let Some(p) = self.task_at(&parent_path) {
            parent_idx < p.children.len() && child_idx < p.children[parent_idx].children.len()
        } else {
            false
        };

        if !can_outdent {
            self.message = "Cannot outdent (invalid selection)".to_string();
            return;
        }

        self.push_undo();
        self.clear_pending_delete();

        let task = if parent_path.is_empty() {
            self.tasks[parent_idx].children.remove(child_idx)
        } else if let Some(p) = self.task_mut_at(&parent_path) {
            p.children[parent_idx].children.remove(child_idx)
        } else {
            return;
        };

        if parent_path.is_empty() {
            let insert_pos = (parent_idx + 1).min(self.tasks.len());
            self.tasks.insert(insert_pos, task);
            self.tree.selected_path = vec![insert_pos];
        } else if let Some(p) = self.task_mut_at(&parent_path) {
            let insert_pos = parent_idx + 1;
            p.children.insert(insert_pos, task);
            let mut new_path = parent_path;
            new_path.push(insert_pos);
            self.tree.selected_path = new_path;
        }

        self.rebuild_visible();
        self.mark_dirty();
        self.message = "Outdented task".to_string();
    }

    pub(crate) fn move_task(&mut self, delta: isize) {
        let idx = match self.tree.selected_path.last().copied() {
            Some(i) => i,
            None => return,
        };
        let mut parent_path = self.tree.selected_path.clone();
        parent_path.pop();

        let new_idx = if parent_path.is_empty() {
            let len = self.tasks.len();
            if len == 0 {
                return;
            }
            (idx as isize + delta).clamp(0, (len - 1) as isize) as usize
        } else if let Some(parent) = self.task_at(&parent_path) {
            let len = parent.children.len();
            if len == 0 {
                return;
            }
            (idx as isize + delta).clamp(0, (len - 1) as isize) as usize
        } else {
            return;
        };

        if new_idx == idx {
            return;
        }

        self.push_undo();
        self.clear_pending_delete();

        if parent_path.is_empty() {
            let task = self.tasks.remove(idx);
            self.tasks.insert(new_idx, task);
            self.tree.selected_path = vec![new_idx];
        } else if let Some(parent) = self.task_mut_at(&parent_path) {
            let task = parent.children.remove(idx);
            parent.children.insert(new_idx, task);
            let mut new_path = parent_path;
            new_path.push(new_idx);
            self.tree.selected_path = new_path;
        }

        self.rebuild_visible();
        self.mark_dirty();
    }

    pub(crate) fn add_task(&mut self, as_child: bool) {
        self.clear_pending_delete();

        let sel_path = self.tree.selected_path.clone();

        if as_child && !sel_path.is_empty() && self.task_at(&sel_path).is_none() {
            self.message = "Cannot add child (invalid selection)".to_string();
            return;
        }

        self.push_undo();

        let new_task = Task::new(self.next_id, "New task");
        self.next_id += 1;

        if sel_path.is_empty() || self.tasks.is_empty() {
            self.tasks.push(new_task);
            self.tree.selected_path = vec![self.tasks.len() - 1];
        } else if as_child {
            if let Some(parent) = self.task_mut_at(&sel_path) {
                let parent_id = parent.id;
                parent.children.push(new_task);
                let mut new_path = sel_path.clone();
                new_path.push(parent.children.len() - 1);
                self.tree.selected_path = new_path;
                self.tree.expanded.insert(parent_id);
            }
        } else {
            let mut parent_path = sel_path.clone();
            let last = parent_path.pop().unwrap_or(0);

            if parent_path.is_empty() {
                let insert_at = (last + 1).min(self.tasks.len());
                self.tasks.insert(insert_at, new_task);
                self.tree.selected_path = vec![insert_at];
            } else if let Some(parent) = self.task_mut_at(&parent_path) {
                let insert_at = (last + 1).min(parent.children.len());
                parent.children.insert(insert_at, new_task);
                let mut new_path = parent_path.clone();
                new_path.push(insert_at);
                self.tree.selected_path = new_path;
            }
        }

        self.rebuild_visible();
        self.mark_dirty();
        self.message = if as_child {
            "Added child task"
        } else {
            "Added sibling task"
        }
        .to_string();

        self.start_edit();
    }

    /// Append a root-level task with the given title (`:add` command).
    pub(crate) fn add_named_task(&mut self, title: &str, open_edit: bool) {
        self.clear_pending_delete();
        self.push_undo();

        let new_task = Task::new(self.next_id, title);
        self.next_id += 1;
        self.tasks.push(new_task);
        self.tree.selected_path = vec![self.tasks.len() - 1];

        self.rebuild_visible();
        self.mark_dirty();
        self.message = format!("Added: {title}");

        if open_edit {
            self.start_edit();
        }
    }

    pub(crate) fn delete_current(&mut self) {
        let deleted = self.current_task().cloned();
        if deleted.is_none() {
            self.message = "Nothing to delete".to_string();
            return;
        }

        let last = match self.tree.selected_path.last().copied() {
            Some(l) => l,
            None => return,
        };

        self.push_undo();
        self.clear_pending_delete();

        if self.mode == Mode::Editing {
            self.cancel_edit();
        }

        let mut parent_path = self.tree.selected_path.clone();
        parent_path.pop();

        if parent_path.is_empty() {
            if last < self.tasks.len() {
                self.tasks.remove(last);
            }
        } else if let Some(parent) = self.task_mut_at(&parent_path)
            && last < parent.children.len()
        {
            parent.children.remove(last);
        }

        if let Some(task) = &deleted {
            let mut ids = HashSet::new();
            Self::collect_task_ids(task, &mut ids);
            for id in &ids {
                self.tree.expanded.remove(id);
            }
            if let Some(focus_path) = &self.tree.focus_path {
                if let Some(focused) = self.task_at(focus_path) {
                    if ids.contains(&focused.id) {
                        self.tree.focus_path = None;
                    }
                } else {
                    self.tree.focus_path = None;
                }
            }
        }

        self.tree.selected_path = if parent_path.is_empty() {
            if self.tasks.is_empty() {
                vec![]
            } else {
                vec![0]
            }
        } else {
            parent_path
        };

        self.rebuild_visible();
        self.mark_dirty();
        self.message = "Deleted task".to_string();
    }

    pub(crate) fn cycle_priority(&mut self) {
        let path = self.tree.selected_path.clone();
        if self.task_at(&path).is_none() {
            return;
        }
        self.push_undo();
        self.clear_pending_delete();
        if let Some(task) = self.task_mut_at(&path) {
            task.priority = task.priority.cycle();
        }
        self.rebuild_visible();
        self.mark_dirty();
    }

    pub(crate) fn cycle_status(&mut self) {
        let path = self.tree.selected_path.clone();
        if self.task_at(&path).is_none() {
            return;
        }
        self.push_undo();
        self.clear_pending_delete();
        if let Some(task) = self.task_mut_at(&path) {
            task.status = task.status.cycle();
        }
        self.rebuild_visible();
        self.mark_dirty();
    }

    pub(crate) fn toggle_done(&mut self) {
        let path = self.tree.selected_path.clone();
        if self.task_at(&path).is_none() {
            return;
        }
        self.push_undo();
        self.clear_pending_delete();
        if let Some(task) = self.task_mut_at(&path) {
            task.status = if task.status == Status::Done {
                Status::Todo
            } else {
                Status::Done
            };
        }
        self.rebuild_visible();
        self.mark_dirty();
    }
}
