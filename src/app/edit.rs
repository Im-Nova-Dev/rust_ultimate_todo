use super::*;

use crate::keys::{is_ctrl_char, is_edit_cancel_key, is_edit_save_key};

impl App {
    pub(crate) fn start_edit(&mut self) {
        self.tree.pending_delete_path = None;
        if let Some(task) = self.current_task() {
            let working = task.clone();
            let desc_lines: Vec<String> = if working.desc.is_empty() {
                vec![String::new()]
            } else {
                working.desc.lines().map(|l| l.to_string()).collect()
            };

            self.edit = Some(EditState {
                working,
                current_field: 0,
                desc_lines,
                desc_row: 0,
                desc_col: 0,
                title_buf: task.title.clone(),
                due_buf: task
                    .due
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                tag_buf: String::new(),
                parsed_due_preview: task.due,
            });
            self.mode = Mode::Editing;
        }
    }

    pub(crate) fn apply_edit(&mut self) {
        let Some(mut ed) = self.edit.take() else {
            return;
        };

        ed.working.title = ed.title_buf.trim().to_string();
        if ed.working.title.is_empty() {
            self.message = "Cannot save task with empty title".to_string();
            self.edit = Some(ed);
            return;
        }

        while ed.desc_lines.last().is_some_and(|l| l.trim().is_empty()) {
            ed.desc_lines.pop();
        }
        ed.working.desc = ed.desc_lines.join("\n");

        let today = Local::now().date_naive();
        let due_trimmed = ed.due_buf.trim();
        if !due_trimmed.is_empty() && parse_relative_date(due_trimmed, today).is_none() {
            self.message =
                "Invalid due date — fix the field or clear it to remove the due date".to_string();
            self.edit = Some(ed);
            return;
        }
        ed.working.due = if due_trimmed.is_empty() {
            None
        } else {
            parse_relative_date(due_trimmed, today)
        };

        let Some(path) = self.find_path_by_id(ed.working.id) else {
            self.message = "Task no longer exists — edit discarded".into();
            self.mode = Mode::Normal;
            return;
        };

        self.push_undo();
        if let Some(task) = self.task_mut_at(&path) {
            *task = ed.working;
        }

        self.mode = Mode::Normal;
        self.rebuild_visible();
        self.mark_dirty();
        self.message = "Task updated".into();
    }

    pub(crate) fn cancel_edit(&mut self) {
        self.edit = None;
        self.mode = Mode::Normal;
        self.message = "Edit cancelled".into();
    }

    pub(crate) fn edit_handle_key(&mut self, key: KeyEvent) {
        if is_edit_cancel_key(&key) {
            self.cancel_edit();
            return;
        }

        if is_edit_save_key(&key) {
            self.apply_edit();
            return;
        }

        if key.code == KeyCode::Esc {
            self.cancel_edit();
            return;
        }

        let Some(ed) = &mut self.edit else { return };

        match key.code {
            KeyCode::Tab => {
                ed.current_field = (ed.current_field + 1) % 6;
            }
            KeyCode::BackTab => {
                ed.current_field = (ed.current_field + 5) % 6;
            }
            _ => match ed.current_field {
                0 => match key.code {
                    KeyCode::Char(c) if ed.title_buf.len() < 500 => ed.title_buf.push(c),
                    KeyCode::Backspace => {
                        ed.title_buf.pop();
                    }
                    _ => {}
                },
                1 => {
                    let lines = &mut ed.desc_lines;
                    let row = &mut ed.desc_row;
                    let col = &mut ed.desc_col;

                    match key.code {
                        KeyCode::Char(_) if is_ctrl_char(&key, 'a') => {
                            *col = 0;
                        }
                        KeyCode::Char(_) if is_ctrl_char(&key, 'e') && *row < lines.len() => {
                            *col = lines[*row].len();
                        }
                        KeyCode::Char(_) if is_ctrl_char(&key, 'k') && *row < lines.len() => {
                            let line = &mut lines[*row];
                            if *col < line.len() {
                                line.truncate(*col);
                            }
                        }
                        KeyCode::Char(_) if is_ctrl_char(&key, 'u') && *row < lines.len() => {
                            let line = &mut lines[*row];
                            if *col <= line.len() {
                                line.drain(0..*col);
                                *col = 0;
                            }
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            while *row >= lines.len() {
                                lines.push(String::new());
                            }
                            let line = &mut lines[*row];
                            if *col > line.len() {
                                *col = line.len();
                            }
                            if line.len() < 10_000 {
                                line.insert(*col, c);
                                *col += 1;
                            }
                        }
                        KeyCode::Backspace => {
                            if *row >= lines.len() {
                                *row = lines.len().saturating_sub(1);
                            }
                            if *col > 0 {
                                let line = &mut lines[*row];
                                line.remove(*col - 1);
                                *col -= 1;
                            } else if *row > 0 {
                                let prev_len = lines[*row - 1].len();
                                let current = lines.remove(*row);
                                lines[*row - 1].push_str(&current);
                                *row -= 1;
                                *col = prev_len;
                            }
                        }
                        KeyCode::Enter => {
                            while *row >= lines.len() {
                                lines.push(String::new());
                            }
                            let rest = lines[*row].split_off(*col);
                            *row += 1;
                            lines.insert(*row, rest);
                            *col = 0;
                        }
                        KeyCode::Up if *row > 0 => {
                            *row -= 1;
                            *col = (*col).min(lines[*row].len());
                        }
                        KeyCode::Down if *row + 1 < lines.len() => {
                            *row += 1;
                            *col = (*col).min(lines[*row].len());
                        }
                        KeyCode::Left => {
                            if *col > 0 {
                                *col -= 1;
                            } else if *row > 0 {
                                *row -= 1;
                                *col = lines[*row].len();
                            }
                        }
                        KeyCode::Right => {
                            if *row < lines.len() && *col < lines[*row].len() {
                                *col += 1;
                            } else if *row + 1 < lines.len() {
                                *row += 1;
                                *col = 0;
                            }
                        }
                        KeyCode::Home => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                *row = 0;
                                *col = 0;
                            } else {
                                *col = 0;
                            }
                        }
                        KeyCode::End => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                if !lines.is_empty() {
                                    *row = lines.len() - 1;
                                    *col = lines[*row].len();
                                }
                            } else if *row < lines.len() {
                                *col = lines[*row].len();
                            }
                        }
                        KeyCode::Delete if *row < lines.len() => {
                            if *col < lines[*row].len() {
                                lines[*row].remove(*col);
                            } else if *row + 1 < lines.len() {
                                let next = lines.remove(*row + 1);
                                lines[*row].push_str(&next);
                            }
                        }
                        _ => {}
                    }
                }
                2 => match key.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        ed.working.priority = match ed.working.priority {
                            Priority::High => Priority::Medium,
                            Priority::Medium => Priority::Low,
                            Priority::Low => Priority::High,
                        };
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        ed.working.priority = ed.working.priority.cycle();
                    }
                    _ => {}
                },
                3 => match key.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        ed.working.status = match ed.working.status {
                            Status::Blocked => Status::Done,
                            Status::Done => Status::Doing,
                            Status::Doing => Status::Todo,
                            Status::Todo => Status::Blocked,
                        };
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        ed.working.status = ed.working.status.cycle();
                    }
                    _ => {}
                },
                4 => match key.code {
                    KeyCode::Char(c) if ed.due_buf.len() < 40 => {
                        ed.due_buf.push(c);
                    }
                    KeyCode::Backspace => {
                        ed.due_buf.pop();
                    }
                    _ => {}
                },
                5 => match key.code {
                    KeyCode::Char(c) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && c.eq_ignore_ascii_case(&'d')
                        {
                            ed.working.tags.pop();
                        } else if ed.tag_buf.len() < 80 {
                            ed.tag_buf.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        ed.tag_buf.pop();
                    }
                    KeyCode::Enter => {
                        let t = ed.tag_buf.trim().to_string();
                        if !t.is_empty() && !ed.working.tags.contains(&t) {
                            ed.working.tags.push(t);
                        }
                        ed.tag_buf.clear();
                    }
                    _ => {}
                },
                _ => {}
            },
        }

        if let Some(ed) = &mut self.edit
            && ed.current_field == 4
        {
            let today = Local::now().date_naive();
            ed.parsed_due_preview = parse_relative_date(&ed.due_buf, today);
        }
    }
}
