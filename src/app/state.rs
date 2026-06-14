use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use chrono::NaiveDate;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;

use crate::model::Task;
use crate::sort::SortMode;

pub(crate) struct UndoSnapshot {
    pub(crate) tasks: Arc<Vec<Task>>,
    pub(crate) expanded: Arc<HashSet<u64>>,
    pub(crate) next_id: u64,
    pub(crate) sort_mode: SortMode,
    pub(crate) selected_path: Vec<usize>,
    pub(crate) focus_path: Option<Vec<usize>>,
    pub(crate) fingerprint: u64,
    pub(crate) estimated_bytes: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Mode {
    Normal,
    Search,
    Command,
    Editing,
    Help,
}

#[derive(Clone, Default)]
pub(crate) struct FilterState {
    pub search: String,
    pub hide_done: bool,
    pub only_high: bool,
    pub only_overdue_or_today: bool,
}

#[derive(Clone)]
pub(crate) struct VisibleItem {
    pub path: Vec<usize>,
    pub depth: usize,
    pub is_last: bool,
}

#[derive(Clone)]
pub(crate) struct EditState {
    pub working: Task,
    pub current_field: usize,
    pub desc_lines: Vec<String>,
    pub desc_row: usize,
    pub desc_col: usize,
    pub title_buf: String,
    pub due_buf: String,
    pub tag_buf: String,
    pub parsed_due_preview: Option<NaiveDate>,
}

/// Autosave and on-disk location.
pub(crate) struct PersistState {
    pub data_path: PathBuf,
    pub dirty: bool,
    pub last_change: Option<Instant>,
    pub last_saved: Option<Instant>,
    /// When set, background autosave waits before retrying after a failed write.
    pub last_save_failed: Option<Instant>,
}

/// Tree selection, visibility, and expand/collapse UI state.
pub(crate) struct TreeViewState {
    pub selected_path: Vec<usize>,
    pub visible: Vec<VisibleItem>,
    pub visible_state: ListState,
    pub expanded: HashSet<u64>,
    pub pending_delete_path: Option<Vec<usize>>,
    pub focus_path: Option<Vec<usize>>,
    pub tree_rect: Rect,
    pub last_click: Option<(u16, u16, Instant)>,
}

pub struct App {
    pub(crate) tasks: Vec<Task>,
    pub(crate) tree: TreeViewState,
    pub(crate) persist: PersistState,
    pub(crate) filter: FilterState,
    pub(crate) sort_mode: SortMode,

    pub(crate) total_tasks: usize,
    pub(crate) done_tasks: usize,

    pub(crate) mode: Mode,
    pub(crate) search_buf: String,
    pub(crate) command_buf: String,

    pub(crate) edit: Option<EditState>,
    pub(crate) message: String,

    pub(crate) undo_stack: Vec<UndoSnapshot>,
    pub(crate) next_id: u64,
}
