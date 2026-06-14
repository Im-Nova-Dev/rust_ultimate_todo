use super::*;

use crate::log::fs_error;
use crate::persist::{sidecar_path, write_project_to_path};

impl App {
    /// Persist to disk without disturbing the status bar (autosave / background).
    pub(crate) fn save(&mut self) {
        self.save_inner(false);
    }

    /// User-initiated save (Ctrl+S) — confirms success in the status bar.
    pub(crate) fn save_explicit(&mut self) {
        let _ = self.save_inner(true);
    }

    /// Returns `true` if the project was written (or was already clean).
    pub(crate) fn save_inner(&mut self, announce: bool) -> bool {
        if !self.persist.dirty {
            if announce {
                self.message = "Already saved".to_string();
            }
            return true;
        }

        if let Some(parent) = self.persist.data_path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            fs_error("create data directory", e);
            self.message = "Save failed (could not create data directory)".to_string();
            self.persist.last_save_failed = Some(std::time::Instant::now());
            return false;
        }

        let tmp_path = sidecar_path(&self.persist.data_path, "tmp");
        if let Err(e) = fs::remove_file(&tmp_path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            fs_error("remove stale temp file", e);
        }

        let expanded: Vec<u64> = self.tree.expanded.iter().copied().collect();
        if let Err(e) = write_project_to_path(
            &tmp_path,
            &self.tasks,
            &expanded,
            self.next_id,
            self.sort_mode,
        ) {
            fs_error("write temp save file", e);
            self.message = "Save failed (write error - disk full?)".to_string();
            self.persist.last_save_failed = Some(std::time::Instant::now());
            return false;
        }

        if self.persist.data_path.exists() {
            let backup_path = sidecar_path(&self.persist.data_path, "bak");
            if let Err(e) = fs::copy(&self.persist.data_path, &backup_path) {
                fs_error("write backup file", e);
            }
        }
        if let Err(e) = fs::rename(&tmp_path, &self.persist.data_path) {
            let _ = fs::remove_file(&tmp_path);
            fs_error("rename temp save file", e);
            self.message = "Save failed (rename error)".to_string();
            self.persist.last_save_failed = Some(std::time::Instant::now());
            return false;
        }

        if announce {
            self.message = format!("Saved ✓  ({})", self.persist.data_path.display());
        }
        self.persist.dirty = false;
        self.persist.last_saved = Some(std::time::Instant::now());
        self.persist.last_save_failed = None;
        true
    }

    /// Flush pending edits and persist before the process exits.
    pub(crate) fn save_on_exit(&mut self) -> bool {
        self.flush_pending_edits();
        self.persist.last_save_failed = None;
        self.save_inner(false)
    }

    /// Commit in-progress modal edits when possible so exit save includes them.
    pub(crate) fn flush_pending_edits(&mut self) {
        if self.mode != Mode::Editing {
            return;
        }
        let can_save = self.edit.as_ref().is_some_and(|ed| {
            if ed.title_buf.trim().is_empty() {
                return false;
            }
            let due_trimmed = ed.due_buf.trim();
            due_trimmed.is_empty()
                || parse_relative_date(due_trimmed, Local::now().date_naive()).is_some()
        });
        if can_save {
            self.apply_edit();
        } else {
            self.cancel_edit();
        }
    }

    pub(crate) fn mark_dirty(&mut self) {
        self.persist.dirty = true;
        self.persist.last_change = Some(std::time::Instant::now());
    }

    pub(crate) fn clear_pending_delete(&mut self) {
        self.tree.pending_delete_path = None;
    }
}
