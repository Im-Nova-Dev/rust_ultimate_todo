//! Search and filter state transitions.

use super::*;

impl App {
    pub(crate) fn start_search(&mut self) {
        self.mode = Mode::Search;
        self.search_buf.clear();
        self.clear_pending_delete();
        self.rebuild_visible();
    }

    pub(crate) fn update_search(&mut self) {
        self.rebuild_visible();
    }

    pub(crate) fn end_search(&mut self) {
        self.filter.search = self.search_buf.clone();
        self.mode = Mode::Normal;
        self.rebuild_visible();
    }

    pub(crate) fn clear_filters(&mut self) {
        self.filter = FilterState::default();
        self.search_buf.clear();
        self.tree.pending_delete_path = None;
        self.rebuild_visible();
        self.message = "Filters cleared".into();
    }

    pub(crate) fn cycle_quick_filter(&mut self) {
        if self.filter.hide_done {
            self.filter.hide_done = false;
            self.filter.only_high = true;
            self.filter.only_overdue_or_today = false;
            self.message = "Filter: High priority only".to_string();
        } else if self.filter.only_high {
            self.filter.only_high = false;
            self.filter.only_overdue_or_today = true;
            self.filter.hide_done = true;
            self.message = "Filter: Overdue / Today + hide done".to_string();
        } else if self.filter.only_overdue_or_today {
            self.filter = FilterState::default();
            self.message = "Filter: All tasks".to_string();
        } else {
            self.filter.hide_done = true;
            self.message = "Filter: Hide completed".to_string();
        }
        self.rebuild_visible();
    }
}
