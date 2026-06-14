//! Visible tree list construction and selection scrolling.

use super::*;

impl App {
    pub(crate) fn rebuild_visible(&mut self) {
        self.tree.visible.clear();

        let selected_id = self.task_at(&self.tree.selected_path).map(|t| t.id);
        let focus_id = self
            .tree
            .focus_path
            .as_ref()
            .and_then(|p| self.task_at(p).map(|t| t.id));

        if let Some(id) = selected_id
            && let Some(path) = self.find_path_by_id(id)
        {
            self.tree.selected_path = path;
        }
        if let Some(id) = focus_id {
            if let Some(path) = self.find_path_by_id(id) {
                self.tree.focus_path = Some(path);
            } else {
                self.tree.focus_path = None;
            }
        } else if self.tree.focus_path.is_some()
            && self
                .task_at(self.tree.focus_path.as_ref().unwrap())
                .is_none()
        {
            self.tree.focus_path = None;
        }

        let today = Local::now().date_naive();
        let search = if self.mode == Mode::Search {
            &self.search_buf
        } else {
            &self.filter.search
        };
        let search_lower = search.to_lowercase();
        let sort_mode = self.sort_mode;

        #[allow(clippy::too_many_arguments)]
        fn collect_visible(
            tasks: &[Task],
            base_path: Vec<usize>,
            depth: usize,
            filter: &FilterState,
            search_lower: &str,
            expanded: &HashSet<u64>,
            today: NaiveDate,
            sort_mode: SortMode,
            visible: &mut Vec<VisibleItem>,
        ) {
            let order = sorted_indices(tasks, sort_mode);
            for (pos, &i) in order.iter().enumerate() {
                let task = &tasks[i];
                let mut path = base_path.clone();
                path.push(i);

                let is_last = pos == order.len() - 1;

                let mut include = true;

                if filter.hide_done && task.status == Status::Done {
                    include = false;
                }
                if filter.only_high && task.priority != Priority::High {
                    include = false;
                }
                if filter.only_overdue_or_today {
                    if let Some(d) = task.due {
                        if d > today {
                            include = false;
                        }
                    } else {
                        include = false;
                    }
                }

                let search_match =
                    task.matches_search(search_lower) || task.has_matching_descendant(search_lower);

                let show = if search_lower.is_empty() {
                    include
                } else {
                    search_match
                };

                if show {
                    visible.push(VisibleItem {
                        path: path.clone(),
                        depth,
                        is_last,
                    });
                }

                if depth < 128 && expanded.contains(&task.id) && !task.children.is_empty() {
                    collect_visible(
                        &task.children,
                        path,
                        depth + 1,
                        filter,
                        search_lower,
                        expanded,
                        today,
                        sort_mode,
                        visible,
                    );
                }
            }
        }

        let focus = self.tree.focus_path.clone();
        if let Some(focus_path) = &focus {
            let focused_snapshot = self
                .task_at(focus_path)
                .map(|t| (t.id, t.children.clone(), t.children.is_empty()));
            if let Some((focused_id, focused_children, no_children)) = focused_snapshot {
                let is_last = no_children || !self.tree.expanded.contains(&focused_id);
                self.tree.visible.push(VisibleItem {
                    path: focus_path.clone(),
                    depth: 0,
                    is_last,
                });
                if self.tree.expanded.contains(&focused_id) && !focused_children.is_empty() {
                    collect_visible(
                        &focused_children,
                        focus_path.clone(),
                        1,
                        &self.filter,
                        &search_lower,
                        &self.tree.expanded,
                        today,
                        sort_mode,
                        &mut self.tree.visible,
                    );
                }
            }
        } else {
            collect_visible(
                &self.tasks,
                vec![],
                0,
                &self.filter,
                &search_lower,
                &self.tree.expanded,
                today,
                sort_mode,
                &mut self.tree.visible,
            );
        }

        let previous_offset = self.tree.visible_state.offset();

        self.tree.visible_state = ListState::default();
        if !self.tree.visible.is_empty() {
            if let Some(idx) = self.find_exact_visible_index(&self.tree.selected_path) {
                self.tree.visible_state.select(Some(idx));
            } else if self.task_at(&self.tree.selected_path).is_some() {
                // Task exists but is filtered out — keep path, don't highlight an ancestor.
                self.tree.visible_state.select(None);
            } else if !self.tree.selected_path.is_empty() {
                self.tree.selected_path = vec![];
                self.tree.visible_state.select(Some(0));
            } else {
                self.tree.visible_state.select(Some(0));
            }

            if let Some(sel) = self.tree.visible_state.selected() {
                let max_offset = self.tree.visible.len().saturating_sub(1);
                *self.tree.visible_state.offset_mut() = previous_offset.min(max_offset);
                self.ensure_selection_visible(sel);
            }
        } else if self.task_at(&self.tree.selected_path).is_none() {
            self.tree.selected_path = vec![];
        }

        if let Some(pending) = &self.tree.pending_delete_path
            && self.task_at(pending).is_none()
        {
            self.tree.pending_delete_path = None;
        }

        self.total_tasks = count_all(&self.tasks);
        self.done_tasks = count_done(&self.tasks);
    }

    fn ensure_selection_visible(&mut self, selected: usize) {
        let viewport = self.tree.tree_rect.height.saturating_sub(2).max(1) as usize;
        let offset = *self.tree.visible_state.offset_mut();
        if selected < offset {
            *self.tree.visible_state.offset_mut() = selected;
        } else if selected >= offset.saturating_add(viewport) {
            *self.tree.visible_state.offset_mut() =
                selected.saturating_sub(viewport.saturating_sub(1));
        }
    }

    pub(crate) fn sync_selection_from_visible(&mut self) {
        if let Some(idx) = self.tree.visible_state.selected()
            && let Some(item) = self.tree.visible.get(idx)
        {
            self.tree.selected_path = item.path.clone();
        }
    }

    pub(crate) fn toggle_expanded(&mut self) {
        if let Some(task) = self.current_task() {
            let id = task.id;
            if self.tree.expanded.contains(&id) {
                self.tree.expanded.remove(&id);
            } else {
                self.tree.expanded.insert(id);
            }
            self.clear_pending_delete();
            self.mark_dirty();
            self.rebuild_visible();
            self.message = "Toggled expand".to_string();
        }
    }

    pub(crate) fn move_selection(&mut self, delta: isize) {
        if self.tree.visible.is_empty() {
            return;
        }

        let cur = match self.tree.visible_state.selected() {
            Some(i) => i as isize,
            None => self
                .find_visible_index_for_path(&self.tree.selected_path)
                .map(|i| i as isize)
                .unwrap_or(0),
        };

        let new = (cur + delta).clamp(0, (self.tree.visible.len() - 1) as isize) as usize;
        self.tree.visible_state.select(Some(new));
        self.sync_selection_from_visible();
        self.ensure_selection_visible(new);
    }
}
