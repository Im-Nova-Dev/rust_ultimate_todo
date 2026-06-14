//! Task statistics, ID sanitization, and markdown export.

use super::*;

use crate::log::fs_error;

impl App {
    pub(crate) fn count_overdue(&self) -> usize {
        let today = Local::now().date_naive();
        fn count(tasks: &[Task], today: NaiveDate) -> usize {
            let mut n = 0;
            for t in tasks {
                if let Some(d) = t.due
                    && d < today
                    && t.status != Status::Done
                {
                    n += 1;
                }
                n += count(&t.children, today);
            }
            n
        }
        count(&self.tasks, today)
    }

    /// Sanitize duplicate IDs on load (makes the app less bug prone against corrupted saves).
    /// Renumbers only duplicates, preserves unique IDs, updates next_id, marks dirty if fixed.
    /// Returns true if duplicates were found and fixed.
    pub(crate) fn sanitize_duplicate_ids(&mut self) -> bool {
        let mut seen = HashSet::new();
        let mut has_dup = false;
        let mut max_id = 0u64;

        fn scan(tasks: &[Task], seen: &mut HashSet<u64>, has: &mut bool, max: &mut u64) {
            for t in tasks {
                if !seen.insert(t.id) {
                    *has = true;
                }
                if t.id > *max {
                    *max = t.id;
                }
                scan(&t.children, seen, has, max);
            }
        }
        scan(&self.tasks, &mut seen, &mut has_dup, &mut max_id);

        if has_dup {
            seen.clear();
            let mut next = max_id + 1;
            fn renumber(tasks: &mut [Task], seen: &mut HashSet<u64>, next: &mut u64) {
                for t in tasks {
                    if !seen.insert(t.id) {
                        t.id = *next;
                        *next += 1;
                    } else if t.id >= *next {
                        *next = t.id + 1;
                    }
                    renumber(&mut t.children, seen, next);
                }
            }
            renumber(&mut self.tasks, &mut seen, &mut next);
            self.next_id = next;
            self.mark_dirty();
            true
        } else {
            if max_id + 1 > self.next_id {
                self.next_id = max_id + 1;
            }
            false
        }
    }

    /// Build a markdown representation of the current task tree.
    pub(crate) fn export_markdown(&self) -> String {
        fn render(tasks: &[Task], level: usize, today: NaiveDate) -> String {
            let mut out = String::new();
            let indent = "  ".repeat(level);
            for t in tasks {
                let status = if t.status == Status::Done {
                    "[x]"
                } else {
                    "[ ]"
                };
                let prio = match t.priority {
                    Priority::High => " **HIGH**",
                    Priority::Medium => "",
                    Priority::Low => " _low_",
                };
                let due = if let Some(d) = t.due {
                    if d < today {
                        format!(" **OVERDUE {d}**")
                    } else if d == today {
                        " **TODAY**".to_string()
                    } else {
                        format!(" due:{d}")
                    }
                } else {
                    String::new()
                };
                let tags = if t.tags.is_empty() {
                    String::new()
                } else {
                    format!(" #{}", t.tags.join(" #"))
                };
                out.push_str(&format!(
                    "{}- {} {}{}{}{}\n",
                    indent, status, t.title, prio, due, tags
                ));
                out.push_str(&render(&t.children, level + 1, today));
            }
            out
        }
        let today = Local::now().date_naive();
        render(&self.tasks, 0, today)
    }

    /// Write markdown export to a sidecar file next to the data path.
    pub(crate) fn write_export_markdown(&self, md: &str) {
        if let Some(parent) = self.persist.data_path.parent() {
            let export_path = parent.join("todos_export.md");
            let tmp = export_path.with_extension("md.tmp");
            if let Err(e) = std::fs::write(&tmp, md) {
                fs_error("write export temp file", e);
                return;
            }
            if let Err(e) = std::fs::rename(&tmp, &export_path) {
                fs_error("rename export file", e);
            }
        }
    }
}
