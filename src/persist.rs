//! Portable `.tdl` project files and legacy JSON compatibility.

use std::fs::File;
use std::io::{BufWriter, Result as IoResult};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::Task;
use crate::sort::SortMode;
use crate::tree_walk::max_id_in_tree;

pub const TDL_FORMAT: &str = "rust_tui/tdl";
pub const TDL_VERSION: u32 = 1;

/// On-disk portable project format (JSON inside a `.tdl` file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdlFile {
    pub format: String,
    pub version: u32,
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub expanded: Vec<u64>,
    #[serde(default)]
    pub next_id: Option<u64>,
    #[serde(default)]
    pub sort_mode: Option<SortMode>,
}

/// Parsed project payload independent of file envelope version.
#[derive(Debug, Clone)]
pub struct LoadedProject {
    pub tasks: Vec<Task>,
    pub next_id: u64,
    pub expanded: Vec<u64>,
    pub sort_mode: SortMode,
}

/// Pre-1.0 flat JSON saves (`tasks.json`).
#[derive(Deserialize)]
struct LegacySavedData {
    tasks: Vec<Task>,
    #[serde(default)]
    expanded: Vec<u64>,
}

#[derive(Serialize)]
struct TdlFileRef<'a> {
    format: &'static str,
    version: u32,
    tasks: &'a [Task],
    expanded: &'a [u64],
    next_id: u64,
    sort_mode: SortMode,
}

pub(crate) fn sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project.tdl");
    let stem = file_name
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(file_name);
    path.with_file_name(format!("{stem}.{suffix}"))
}

pub(crate) fn expand_user_path(path: &str) -> PathBuf {
    let trimmed = path.trim();
    if trimmed == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(trimmed));
    }
    if let Some(rest) = trimmed.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(trimmed)
}

fn next_id_from_tasks(tasks: &[Task], explicit: Option<u64>) -> u64 {
    let derived = tasks
        .iter()
        .map(max_id_in_tree)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    match explicit {
        Some(id) if id >= derived => id,
        _ => derived,
    }
}

/// Parse a `.tdl` or legacy JSON project file.
pub(crate) fn parse_project_data(data: &str) -> Option<LoadedProject> {
    let value = serde_json::from_str::<serde_json::Value>(data).ok()?;

    if value.get("format").and_then(|f| f.as_str()) == Some(TDL_FORMAT) {
        let tdl: TdlFile = serde_json::from_value(value).ok()?;
        if tdl.version > TDL_VERSION {
            return None;
        }
        let next_id = next_id_from_tasks(&tdl.tasks, tdl.next_id);
        return Some(LoadedProject {
            tasks: tdl.tasks,
            next_id,
            expanded: tdl.expanded,
            sort_mode: tdl.sort_mode.unwrap_or_default(),
        });
    }

    if value.get("tasks").is_some() {
        let legacy: LegacySavedData = serde_json::from_value(value).ok()?;
        return Some(LoadedProject {
            next_id: next_id_from_tasks(&legacy.tasks, None),
            tasks: legacy.tasks,
            expanded: legacy.expanded,
            sort_mode: SortMode::Manual,
        });
    }

    None
}

/// Serialize a portable project to disk (atomic write uses a `.tmp` sidecar).
pub(crate) fn write_project_to_path(
    path: &Path,
    tasks: &[Task],
    expanded: &[u64],
    next_id: u64,
    sort_mode: SortMode,
) -> IoResult<()> {
    let data = TdlFileRef {
        format: TDL_FORMAT,
        version: TDL_VERSION,
        tasks,
        expanded,
        next_id,
        sort_mode,
    };
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &data).map_err(std::io::Error::other)
}
