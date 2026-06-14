use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::*;

/// Max undo steps kept regardless of tree size.
const MAX_UNDO_ENTRIES: usize = 12;
/// Total estimated heap for all undo snapshots (~32 MiB).
const MAX_UNDO_BYTES: usize = 32 * 1024 * 1024;

fn hash_tasks(tasks: &[Task], h: &mut impl Hasher) {
    for t in tasks {
        t.id.hash(h);
        t.title.hash(h);
        t.desc.hash(h);
        t.status.hash(h);
        t.priority.hash(h);
        t.due.hash(h);
        t.tags.hash(h);
        t.children.len().hash(h);
        hash_tasks(&t.children, h);
    }
}

/// Session-local fingerprint for deduplicating undo snapshots.
/// Uses `DefaultHasher` intentionally — stability across Rust versions is not required.
fn fingerprint(
    tasks: &[Task],
    expanded: &HashSet<u64>,
    next_id: u64,
    sort_mode: SortMode,
    selected_path: &[usize],
    focus_path: &Option<Vec<usize>>,
) -> u64 {
    let mut h = DefaultHasher::new();
    hash_tasks(tasks, &mut h);
    let mut ids: Vec<_> = expanded.iter().copied().collect();
    ids.sort_unstable();
    ids.hash(&mut h);
    next_id.hash(&mut h);
    sort_mode.hash(&mut h);
    selected_path.hash(&mut h);
    focus_path.hash(&mut h);
    h.finish()
}

fn estimate_tasks_heap(tasks: &[Task]) -> usize {
    let mut n = 0usize;
    fn walk(ts: &[Task], n: &mut usize) {
        for t in ts {
            *n += std::mem::size_of::<Task>()
                + t.title.capacity()
                + t.desc.capacity()
                + t.tags.iter().map(|s| s.capacity()).sum::<usize>()
                + t.children.capacity() * std::mem::size_of::<Task>();
            walk(&t.children, n);
        }
    }
    walk(tasks, &mut n);
    n
}

fn estimate_snapshot_bytes(tasks: &[Task], expanded: &HashSet<u64>) -> usize {
    estimate_tasks_heap(tasks) + expanded.len() * std::mem::size_of::<u64>()
}

fn trim_stack(stack: &mut Vec<UndoSnapshot>) {
    while stack.len() > MAX_UNDO_ENTRIES {
        stack.remove(0);
    }
    let mut total: usize = stack.iter().map(|s| s.estimated_bytes).sum();
    while total > MAX_UNDO_BYTES && stack.len() > 1 {
        if let Some(removed) = stack.first() {
            total = total.saturating_sub(removed.estimated_bytes);
        }
        stack.remove(0);
    }
}

impl App {
    pub(crate) fn push_undo(&mut self) {
        let fp = fingerprint(
            &self.tasks,
            &self.tree.expanded,
            self.next_id,
            self.sort_mode,
            &self.tree.selected_path,
            &self.tree.focus_path,
        );
        if self.undo_stack.last().is_some_and(|s| s.fingerprint == fp) {
            return;
        }

        let bytes = estimate_snapshot_bytes(&self.tasks, &self.tree.expanded);
        self.undo_stack.push(UndoSnapshot {
            tasks: Arc::new(self.tasks.clone()),
            expanded: Arc::new(self.tree.expanded.clone()),
            next_id: self.next_id,
            sort_mode: self.sort_mode,
            selected_path: self.tree.selected_path.clone(),
            focus_path: self.tree.focus_path.clone(),
            fingerprint: fp,
            estimated_bytes: bytes,
        });
        trim_stack(&mut self.undo_stack);
    }

    pub(crate) fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.tasks = Arc::try_unwrap(prev.tasks).unwrap_or_else(|arc| (*arc).clone());
            self.tree.expanded =
                Arc::try_unwrap(prev.expanded).unwrap_or_else(|arc| (*arc).clone());
            self.next_id = prev.next_id;
            self.sort_mode = prev.sort_mode;
            self.tree.selected_path = prev.selected_path;
            self.tree.focus_path = prev.focus_path;
            self.tree.pending_delete_path = None;
            self.edit = None;
            if self.mode == Mode::Editing {
                self.mode = Mode::Normal;
            }
            self.rebuild_visible();
            self.mark_dirty();
            self.message = "Undid last change".into();
        } else {
            self.message = "Nothing to undo".into();
        }
    }
}
