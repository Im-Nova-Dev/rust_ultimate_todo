//! App construction and loading persisted task data.

use super::*;

use crate::log::fs_error;
use crate::persist::{LoadedProject, expand_user_path, parse_project_data, sidecar_path};

#[derive(Clone, Copy)]
pub enum MissingFilePolicy {
    /// Built-in sample tree (default data directory on first run).
    SampleData,
    /// Blank project (portable `.tdl` path that does not exist yet).
    EmptyProject,
}

impl App {
    pub fn new(data_path: PathBuf) -> Self {
        Self::new_with_policy(data_path, MissingFilePolicy::SampleData)
    }

    pub fn new_portable(data_path: PathBuf) -> Self {
        Self::new_with_policy(data_path, MissingFilePolicy::EmptyProject)
    }

    fn new_with_policy(data_path: PathBuf, on_missing: MissingFilePolicy) -> Self {
        let (tasks, next_id, expanded, sort_mode, load_msg) =
            Self::load_project(&data_path, on_missing);
        let selected_path = if tasks.is_empty() { vec![] } else { vec![0] };

        let mut app = Self {
            tasks,
            tree: TreeViewState {
                selected_path,
                visible: vec![],
                visible_state: ListState::default(),
                expanded,
                pending_delete_path: None,
                focus_path: None,
                tree_rect: Rect::default(),
                last_click: None,
            },
            persist: PersistState {
                data_path,
                dirty: false,
                last_change: None,
                last_saved: None,
                last_save_failed: None,
            },
            filter: FilterState::default(),
            sort_mode,
            mode: Mode::Normal,
            search_buf: String::new(),
            command_buf: String::new(),
            edit: None,
            message: "Welcome to Deep Todo — Press ? for help".to_string(),
            undo_stack: vec![],
            next_id,
            total_tasks: 0,
            done_tasks: 0,
        };

        app.prune_expanded();
        app.rebuild_visible();

        if app.sanitize_duplicate_ids() {
            app.message = "Duplicate IDs detected and renumbered in loaded data.".to_string();
        } else if let Some(msg) = load_msg {
            app.message = msg;
            app.mark_dirty();
        } else if app.persist.data_path.exists() {
            app.persist.last_saved = Some(std::time::Instant::now());
            app.message = format!("Opened {}", app.persist.data_path.display());
        } else if matches!(on_missing, MissingFilePolicy::EmptyProject) {
            app.message = format!(
                "New project — will save to {}",
                app.persist.data_path.display()
            );
            app.mark_dirty();
        }
        app
    }

    pub(crate) fn sample_data() -> (Vec<Task>, u64, HashSet<u64>) {
        let mut tasks = vec![
            Task {
                id: 1,
                title: "Build the ultimate Rust TUI todo".to_string(),
                desc: "Make a deeply nested, feature-rich hierarchical todo list.\n\nFeatures wanted:\n- Tree navigation\n- Rich metadata\n- Powerful filters\n- Full edit experience\n- Persistence & undo".to_string(),
                priority: Priority::High,
                status: Status::Doing,
                due: Some(NaiveDate::from_ymd_opt(2026, 6, 25).expect("valid sample date")),
                tags: vec!["rust".into(), "tui".into(), "ambitious".into()],
                children: vec![
                    Task::new(2, "Design data model (nested Task + rich fields)"),
                    Task::new(3, "Implement tree rendering with connectors"),
                    Task {
                        id: 4,
                        title: "Build the edit modal (very deep form)".to_string(),
                        desc: "Support title, multi-line desc, priority, status, due date parser, tags management.".to_string(),
                        priority: Priority::High,
                        status: Status::Todo,
                        due: Some(NaiveDate::from_ymd_opt(2026, 6, 18).expect("valid sample date")),
                        tags: vec!["ui".into()],
                        children: vec![
                            Task::new(5, "Title + due date input with live parsing"),
                            Task::new(6, "Multi-line description editor"),
                            Task::new(7, "Tag add/remove UI"),
                        ],
                    },
                ],
            },
            Task {
                id: 8,
                title: "Write documentation and examples".to_string(),
                desc: String::new(),
                priority: Priority::Medium,
                status: Status::Todo,
                due: Some(NaiveDate::from_ymd_opt(2026, 6, 30).expect("valid sample date")),
                tags: vec!["docs".into()],
                children: vec![],
            },
            Task {
                id: 9,
                title: "Polish & release".to_string(),
                desc: "Make sure undo, persistence, filters, and navigation feel buttery.".to_string(),
                priority: Priority::Low,
                status: Status::Todo,
                due: None,
                tags: vec!["release".into()],
                children: vec![
                    Task::new(10, "Add more relative date parsing"),
                    Task::new(11, "Bulk actions (visual mode)"),
                ],
            },
        ];

        tasks[0].children[0].status = Status::Done;

        let max_id = 11;
        let expanded: HashSet<u64> = [1, 4].into_iter().collect();

        (tasks, max_id + 1, expanded)
    }

    pub(crate) fn load_project(
        path: &PathBuf,
        on_missing: MissingFilePolicy,
    ) -> (Vec<Task>, u64, HashSet<u64>, SortMode, Option<String>) {
        if path.exists() {
            if let Ok(data) = fs::read_to_string(path)
                && let Some(parsed) = parse_project_data(&data)
            {
                return (
                    parsed.tasks,
                    parsed.next_id,
                    parsed.expanded.into_iter().collect(),
                    parsed.sort_mode,
                    None,
                );
            }

            let backup_path = sidecar_path(path, "bak");
            if let Ok(backup_data) = fs::read_to_string(&backup_path)
                && let Some(parsed) = parse_project_data(&backup_data)
            {
                return (
                    parsed.tasks,
                    parsed.next_id,
                    parsed.expanded.into_iter().collect(),
                    parsed.sort_mode,
                    Some(format!(
                        "Main save at {} was corrupt; restored from backup.",
                        path.display()
                    )),
                );
            }

            let corrupt_path = sidecar_path(path, "corrupt");
            if let Err(e) = fs::copy(path, &corrupt_path) {
                fs_error("preserve corrupt save file", e);
            }

            // Never substitute sample data when a file existed but could not be parsed —
            // that would overwrite the user's project on the next save.
            let fallback = (vec![], 1, HashSet::new());
            return (
                fallback.0,
                fallback.1,
                fallback.2,
                SortMode::Manual,
                Some(format!(
                    "Could not load {} — original preserved at {}. Starting fresh.",
                    path.display(),
                    corrupt_path.display()
                )),
            );
        }

        match on_missing {
            MissingFilePolicy::SampleData => {
                let (tasks, next_id, expanded) = Self::sample_data();
                (tasks, next_id, expanded, SortMode::Manual, None)
            }
            MissingFilePolicy::EmptyProject => (vec![], 1, HashSet::new(), SortMode::Manual, None),
        }
    }

    pub(crate) fn apply_loaded_project(&mut self, project: LoadedProject) {
        self.tasks = project.tasks;
        self.next_id = project.next_id;
        self.tree.expanded = project.expanded.into_iter().collect();
        self.sort_mode = project.sort_mode;
        self.tree.pending_delete_path = None;
        self.tree.focus_path = None;
        self.tree.selected_path = if self.tasks.is_empty() {
            vec![]
        } else {
            vec![0]
        };
        self.undo_stack.clear();
        self.edit = None;
        self.mode = Mode::Normal;
        self.filter = FilterState::default();
        self.search_buf.clear();
        self.command_buf.clear();
        self.prune_expanded();
        self.rebuild_visible();
    }

    pub(crate) fn open_project_file(&mut self, path: PathBuf, force: bool) {
        if self.persist.dirty && !force {
            self.message = format!("Unsaved changes — type :open {} confirm", path.display());
            return;
        }

        let (tasks, next_id, expanded, sort_mode, load_msg) =
            Self::load_project(&path, MissingFilePolicy::EmptyProject);

        self.persist.data_path = path.clone();
        self.apply_loaded_project(LoadedProject {
            tasks,
            next_id,
            expanded: expanded.into_iter().collect(),
            sort_mode,
        });

        if self.sanitize_duplicate_ids() {
            self.message = format!("Opened {} — duplicate IDs were renumbered.", path.display());
            self.mark_dirty();
        } else if let Some(msg) = load_msg {
            self.message = msg;
            self.mark_dirty();
        } else if path.exists() {
            self.persist.dirty = false;
            self.persist.last_saved = Some(std::time::Instant::now());
            self.message = format!("Opened {}", path.display());
        } else {
            self.message = format!("New project — will save to {}", path.display());
            self.mark_dirty();
        }
    }

    pub(crate) fn save_project_as(&mut self, path: PathBuf) {
        let previous_path = self.persist.data_path.clone();
        self.persist.data_path = path;
        self.mark_dirty();
        if !self.save_inner(true) {
            self.persist.data_path = previous_path;
        }
    }

    /// Parse `:open` preserving path casing from the raw command line.
    pub(crate) fn parse_open_path_raw(raw: &str) -> Option<(PathBuf, bool)> {
        let mut words: Vec<&str> = raw.split_whitespace().collect();
        if words.is_empty() || !words[0].eq_ignore_ascii_case("open") {
            return None;
        }
        words.remove(0);
        let confirm = words.last().is_some_and(|w| w.eq_ignore_ascii_case("confirm"));
        if confirm {
            words.pop();
        }
        if words.is_empty() {
            return None;
        }
        Some((expand_user_path(&words.join(" ")), confirm))
    }

    /// Parse `:save-as` preserving path casing from the raw command line.
    pub(crate) fn parse_save_as_path_raw(raw: &str) -> Option<PathBuf> {
        let words: Vec<&str> = raw.split_whitespace().collect();
        if words.len() < 2 {
            return None;
        }
        let cmd = words[0].to_ascii_lowercase();
        if cmd != "save-as" && cmd != "saveas" {
            return None;
        }
        Some(expand_user_path(&words[1..].join(" ")))
    }
}
