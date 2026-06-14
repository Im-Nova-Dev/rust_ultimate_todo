use super::*;

use crate::tree_walk::collect_task_ids;

impl App {
    pub(crate) fn task_mut_at(&mut self, path: &[usize]) -> Option<&mut Task> {
        if path.is_empty() {
            return None;
        }
        let mut cur = self.tasks.get_mut(path[0])?;
        for &idx in &path[1..] {
            cur = cur.children.get_mut(idx)?;
        }
        Some(cur)
    }

    pub(crate) fn task_at(&self, path: &[usize]) -> Option<&Task> {
        if path.is_empty() {
            return None;
        }
        let mut cur = self.tasks.get(path[0])?;
        for &idx in &path[1..] {
            cur = cur.children.get(idx)?;
        }
        Some(cur)
    }

    pub(crate) fn current_task(&self) -> Option<&Task> {
        self.task_at(&self.tree.selected_path)
    }

    pub(crate) fn find_path_by_id(&self, target_id: u64) -> Option<Vec<usize>> {
        let mut path = Vec::new();
        fn search(tasks: &[Task], path: &mut Vec<usize>, target: u64) -> bool {
            for (i, task) in tasks.iter().enumerate() {
                path.push(i);
                if task.id == target {
                    return true;
                }
                if search(&task.children, path, target) {
                    return true;
                }
                path.pop();
            }
            false
        }
        if search(&self.tasks, &mut path, target_id) {
            Some(path)
        } else {
            None
        }
    }

    pub(crate) fn ensure_path_visible(&mut self, path: &[usize]) {
        let mut changed = false;
        let mut current = vec![];
        for &idx in path.iter().take(path.len().saturating_sub(1)) {
            current.push(idx);
            if let Some(task) = self.task_at(&current)
                && self.tree.expanded.insert(task.id)
            {
                changed = true;
            }
        }
        if changed {
            self.mark_dirty();
        }
    }

    pub(crate) fn collect_task_ids(task: &Task, out: &mut HashSet<u64>) {
        collect_task_ids(task, out);
    }

    pub(crate) fn prune_expanded(&mut self) {
        let mut valid = HashSet::new();
        for t in &self.tasks {
            Self::collect_task_ids(t, &mut valid);
        }
        self.tree.expanded.retain(|id| valid.contains(id));
    }
}
