//! Selection navigation, focus mode, and duplication.

use super::*;

impl App {
    pub(crate) fn go_to_parent(&mut self) {
        if self.tree.selected_path.len() > 1 {
            let mut p = self.tree.selected_path.clone();
            p.pop();
            self.tree.selected_path = p;
            let path_to_ensure = self.tree.selected_path.clone();
            self.ensure_path_visible(&path_to_ensure);
            self.clear_pending_delete();
            self.rebuild_visible();
        }
    }

    pub(crate) fn go_to_top(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.tree.selected_path = vec![0];
        self.clear_pending_delete();
        self.rebuild_visible();
    }

    pub(crate) fn go_to_bottom(&mut self) {
        if self.tree.visible.is_empty() {
            return;
        }
        let last = self.tree.visible.len() - 1;
        self.tree.visible_state.select(Some(last));
        self.sync_selection_from_visible();
        self.clear_pending_delete();
    }

    /// Visible list index for an exact path match only.
    pub(crate) fn find_exact_visible_index(&self, path: &[usize]) -> Option<usize> {
        self.tree
            .visible
            .iter()
            .position(|v| v.path == path)
    }

    /// Find the best visible list index for a path: exact match first, then nearest ancestor.
    pub(crate) fn find_visible_index_for_path(&self, path: &[usize]) -> Option<usize> {
        if let Some(idx) = self.find_exact_visible_index(path) {
            return Some(idx);
        }
        if path.is_empty() {
            return None;
        }
        for (i, v) in self.tree.visible.iter().enumerate() {
            if v.path == path {
                return Some(i);
            }
        }
        let mut ancestor = path.to_vec();
        while !ancestor.is_empty() {
            for (i, v) in self.tree.visible.iter().enumerate() {
                if v.path == ancestor {
                    return Some(i);
                }
            }
            ancestor.pop();
        }
        None
    }

    pub(crate) fn focus_on(&mut self, id: u64) {
        if let Some(path) = self.find_path_by_id(id) {
            self.tree.focus_path = Some(path.clone());
            self.ensure_path_visible(&path);
            self.tree.selected_path = path;
            self.clear_pending_delete();
            self.rebuild_visible();
            self.message = format!("Focused on #{id}");
        } else {
            self.message = format!("No task #{id} to focus");
        }
    }

    pub(crate) fn unfocus(&mut self) {
        if self.tree.focus_path.is_some() {
            self.tree.focus_path = None;
            self.clear_pending_delete();
            self.rebuild_visible();
            self.message = "Unfocused (showing full tree)".to_string();
        }
    }

    pub(crate) fn assign_fresh_ids(task: &mut Task, next_id: &mut u64) {
        task.id = *next_id;
        *next_id += 1;
        for child in &mut task.children {
            Self::assign_fresh_ids(child, next_id);
        }
    }

    pub(crate) fn duplicate_current(&mut self) {
        if self.tree.selected_path.is_empty() {
            return;
        }
        self.push_undo();
        self.clear_pending_delete();
        let path_for_action = self.tree.selected_path.clone();
        if let Some(task) = self.task_at(&path_for_action) {
            let mut new_task = task.clone();
            Self::assign_fresh_ids(&mut new_task, &mut self.next_id);
            let mut parent_path = path_for_action.clone();
            let last = parent_path.pop().unwrap_or(0);
            if parent_path.is_empty() {
                let insert_at = (last + 1).min(self.tasks.len());
                self.tasks.insert(insert_at, new_task);
                self.tree.selected_path = vec![insert_at];
            } else if let Some(parent) = self.task_mut_at(&parent_path) {
                let insert_at = (last + 1).min(parent.children.len());
                parent.children.insert(insert_at, new_task);
                let mut new_p = parent_path.clone();
                new_p.push(insert_at);
                self.tree.selected_path = new_p;
            }
            self.rebuild_visible();
            let new_id = self
                .task_at(&self.tree.selected_path)
                .map(|t| t.id)
                .unwrap_or(0);
            self.mark_dirty();
            self.message = format!("Duplicated task as #{new_id}");
        }
    }

    pub(crate) fn get_breadcrumb(&self) -> String {
        if self.tree.selected_path.is_empty() {
            return String::new();
        }
        let mut parts = vec![];
        let mut cur = vec![];
        for &i in &self.tree.selected_path {
            cur.push(i);
            if let Some(t) = self.task_at(&cur) {
                parts.push(format!("#{}", t.id));
            }
        }
        parts.join(" > ")
    }
}
