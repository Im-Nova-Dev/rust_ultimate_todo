mod id;
mod nav;
mod task;

use super::*;

impl App {
    pub(crate) fn execute_command(&mut self) {
        let raw = self.command_buf.trim().to_string();
        let cmd = raw.to_lowercase();
        self.command_buf.clear();
        self.mode = Mode::Normal;

        if cmd.is_empty() {
            return;
        }

        if let Ok(id) = cmd.parse::<u64>() {
            if self.jump_to_id(id) {
                self.message = format!("Jumped to task #{id}");
            } else {
                self.message = format!("No task with ID #{id}");
            }
            return;
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();

        if parts.len() >= 2
            && matches!(parts[0], "jump" | "select" | "go" | "g")
            && let Ok(id) = parts[1].parse::<u64>()
        {
            if self.jump_to_id(id) {
                self.message = format!("Jumped to #{id}");
            } else {
                self.message = format!("No task #{id}");
            }
            return;
        }

        if id::try_execute(self, &parts) {
            return;
        }
        if nav::try_execute(self, &parts, &raw) {
            return;
        }
        task::execute(self, &parts, &cmd, &raw);
    }

    pub(crate) fn jump_to_id(&mut self, id: u64) -> bool {
        if let Some(path) = self.find_path_by_id(id) {
            self.clear_pending_delete();
            self.ensure_path_visible(&path);
            self.tree.selected_path = path;
            self.rebuild_visible();
            true
        } else {
            false
        }
    }
}
