//! Application state machine: task data, filters, undo, and command handling.

mod access;
mod commands;
mod data;
mod edit;
mod load;
mod manipulate;
mod mouse;
mod navigation;
mod progress;
mod save;
mod search;
mod state;
mod stats;
mod undo;
mod visibility;

#[cfg(test)]
mod tests;

pub use load::MissingFilePolicy;
pub use state::App;

pub(crate) use state::{
    EditState, FilterState, Mode, PersistState, TreeViewState, UndoSnapshot, VisibleItem,
};

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::date::parse_relative_date;
use crate::model::{Priority, Status, Task};
use crate::persist::LoadedProject;
use crate::sort::{SortMode, sorted_indices};
use chrono::{Local, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;

use stats::{count_all, count_done};
