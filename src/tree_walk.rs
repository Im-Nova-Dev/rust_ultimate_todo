//! Shared read-only walks over the task tree.

use std::collections::HashSet;

use crate::model::Task;

pub(crate) fn max_id_in_tree(task: &Task) -> u64 {
    let mut m = task.id;
    for c in &task.children {
        m = m.max(max_id_in_tree(c));
    }
    m
}

pub(crate) fn collect_task_ids(task: &Task, out: &mut HashSet<u64>) {
    out.insert(task.id);
    for c in &task.children {
        collect_task_ids(c, out);
    }
}

pub(crate) fn expand_all_ids(tasks: &[Task], set: &mut HashSet<u64>, depth: usize) {
    if depth > 128 {
        return;
    }
    for t in tasks {
        set.insert(t.id);
        expand_all_ids(&t.children, set, depth + 1);
    }
}
