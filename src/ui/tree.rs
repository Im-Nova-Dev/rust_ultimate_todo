use chrono::Local;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};

use crate::app::{App, Mode};
use crate::model::Status;
use crate::theme::{self, PLUM, ROSE};

pub(super) fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_focused = matches!(app.mode, Mode::Normal | Mode::Search);

    let title = if app.mode == Mode::Search {
        format!(" 🌲 Tree · search: {} ", app.search_buf)
    } else if let Some(focus_path) = &app.tree.focus_path {
        if let Some(t) = app.task_at(focus_path) {
            format!(" 🌲 Tree · focus #{} ", t.id)
        } else {
            " 🌲 Tree · focus ".to_string()
        }
    } else {
        " 🌲 Tree ".to_string()
    };

    let block = Block::new()
        .title(title)
        .borders(Borders::ALL)
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        })
        .border_style(if is_focused {
            theme::border_focused()
        } else {
            theme::border_idle()
        });

    let inner = block.inner(area);
    app.tree.tree_rect = inner;
    frame.render_widget(block, area);

    if app.tree.visible.is_empty() {
        let msg = if app.tasks.is_empty() {
            "No tasks yet.\nPress 'a' to add your first task."
        } else if app.current_task().is_some() {
            "No tasks match current filters.\nYour selection is kept — press F to clear filters."
        } else {
            "No tasks match current filters.\nPress F to clear filters or / to search."
        };
        let empty = Paragraph::new(msg).style(theme::muted());
        frame.render_widget(empty, inner);
        return;
    }

    let tree_list = {
        let mut items = Vec::new();
        let today = Local::now().date_naive();

        for item in &app.tree.visible {
            if let Some(task) = app.task_at(&item.path) {
                let is_selected = app.tree.selected_path == item.path;
                let is_pending_delete = app.tree.pending_delete_path.as_ref() == Some(&item.path);

                let mut prefix = String::new();
                for _ in 0..item.depth {
                    prefix.push_str("│   ");
                }
                if item.depth > 0 {
                    if item.is_last {
                        prefix.push_str("└── ");
                    } else {
                        prefix.push_str("├── ");
                    }
                }

                let exp = if task.children.is_empty() {
                    "  "
                } else if app.tree.expanded.contains(&task.id) {
                    "▼ "
                } else {
                    "▶ "
                };

                let status = Span::styled(
                    format!("{} ", task.status.symbol()),
                    Style::default().fg(task.status.color()),
                );

                let prio = Span::styled(
                    format!("[{}] ", task.priority.as_str()),
                    Style::default().fg(task.priority.color()).bold(),
                );

                let due_str = if let Some(d) = task.due {
                    if d < today {
                        format!("OVERDUE({}) ", (today - d).num_days())
                    } else if d == today {
                        "TODAY ".to_string()
                    } else {
                        format!("due {} ", d.format("%m/%d"))
                    }
                } else {
                    String::new()
                };

                let due_style = Style::default().fg(theme::due(today, task.due)).bold();

                let due = Span::styled(due_str, due_style);

                let title_style = if is_pending_delete {
                    theme::tree_row_pending_delete()
                } else if task.status == Status::Done {
                    theme::tree_row_done()
                } else if is_selected {
                    theme::tree_row_selected()
                } else {
                    theme::body()
                };

                let tags_str = if task.tags.is_empty() {
                    String::new()
                } else {
                    format!(
                        " {}",
                        task.tags
                            .iter()
                            .map(|t| format!("#{t}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                };

                let id_prefix = format!("#{} ", task.id);
                let delete_suffix = if is_pending_delete {
                    " [CONFIRM DELETE]"
                } else {
                    ""
                };
                let line = Line::from(vec![
                    Span::styled(prefix, theme::tree_connector()),
                    Span::styled(exp, theme::muted()),
                    status,
                    prio,
                    due,
                    Span::styled(id_prefix, theme::muted()),
                    Span::styled(task.title.clone(), title_style),
                    Span::styled(tags_str, Style::default().fg(PLUM)),
                    Span::styled(delete_suffix, Style::default().fg(ROSE).bold()),
                ]);

                items.push(ListItem::new(line));
            }
        }

        List::new(items).highlight_style(Style::default())
    };

    frame.render_stateful_widget(tree_list, inner, &mut app.tree.visible_state);
}
