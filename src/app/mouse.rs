//! Mouse interaction on the tree view.

use super::*;

impl App {
    pub(crate) fn handle_tree_click(&mut self, x: u16, y: u16, is_right_click: bool) {
        self.clear_pending_delete();
        let r = self.tree.tree_rect;
        if x < r.x || x >= r.x + r.width || y < r.y || y >= r.y + r.height {
            return;
        }

        let rel_y = y.saturating_sub(self.tree.tree_rect.y) as usize;
        let offset = self.tree.visible_state.offset();
        let clicked_idx = rel_y + offset;

        if self.tree.visible.is_empty() || clicked_idx >= self.tree.visible.len() {
            return;
        }

        let item = &self.tree.visible[clicked_idx];
        let clicked_path = item.path.clone();

        self.tree.selected_path = clicked_path.clone();
        self.tree.visible_state.select(Some(clicked_idx));

        let now = std::time::Instant::now();
        let is_double = if let Some((lx, ly, lt)) = self.tree.last_click {
            lx == x && ly == y && now.duration_since(lt).as_millis() < 350
        } else {
            false
        };
        self.tree.last_click = Some((x, y, now));

        if is_double {
            self.start_edit();
            return;
        }

        let tree_structure_width = 3u16 + (item.depth as u16 * 3).min(20);
        let relative_x = x.saturating_sub(self.tree.tree_rect.x);
        let clicked_icon_area = relative_x < tree_structure_width.max(6);

        if clicked_icon_area {
            if let Some(task) = self.task_at(&clicked_path) {
                let id = task.id;
                if self.tree.expanded.contains(&id) {
                    self.tree.expanded.remove(&id);
                } else {
                    self.tree.expanded.insert(id);
                }
            }
            self.mark_dirty();
            self.rebuild_visible();
            self.message = "Toggled expand/collapse".to_string();
        } else if is_right_click {
            let path = clicked_path.clone();
            self.push_undo();
            if let Some(task) = self.task_mut_at(&path) {
                task.status = task.status.cycle();
                self.rebuild_visible();
                self.mark_dirty();
                self.message = "Status cycled (right-click)".to_string();
            }
        }
    }
}
