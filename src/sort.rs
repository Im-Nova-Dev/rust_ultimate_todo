use serde::{Deserialize, Serialize};

use crate::model::{Priority, Task};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortMode {
    #[default]
    Manual,
    Priority,
    DueDate,
    Title,
}

impl std::fmt::Display for SortMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortMode::Manual => write!(f, "Manual"),
            SortMode::Priority => write!(f, "Priority"),
            SortMode::DueDate => write!(f, "Due date"),
            SortMode::Title => write!(f, "Title"),
        }
    }
}

fn priority_rank(p: &Priority) -> u8 {
    match p {
        Priority::High => 0,
        Priority::Medium => 1,
        Priority::Low => 2,
    }
}

fn compare_tasks_for_sort(a: &Task, b: &Task, mode: SortMode) -> std::cmp::Ordering {
    match mode {
        SortMode::Manual => std::cmp::Ordering::Equal,
        SortMode::Priority => priority_rank(&a.priority).cmp(&priority_rank(&b.priority)),
        SortMode::DueDate => match (a.due, b.due) {
            (Some(da), Some(db)) => da.cmp(&db),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
        },
        SortMode::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
    }
}

/// View-only sort order — does not mutate the task tree.
pub(crate) fn sorted_indices(tasks: &[Task], mode: SortMode) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..tasks.len()).collect();
    if mode != SortMode::Manual {
        indices.sort_by(|&a, &b| compare_tasks_for_sort(&tasks[a], &tasks[b], mode));
    }
    indices
}
