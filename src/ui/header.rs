use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};

use crate::app::App;
use crate::theme::{self, LAVENDER, PEACH, PLUM, ROSE, SAGE, SAND, SKY};

fn truncate_chars(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let n: usize = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

pub(super) fn render(frame: &mut Frame, area: Rect, app: &App) {
    let title = Span::styled(" 🦀 rust_tui ", Style::default().fg(SAND).bold());

    let filter_summary = {
        let mut parts = vec![];
        if !app.filter.search.is_empty() {
            parts.push(format!("search:\"{}\"", app.filter.search));
        }
        if app.filter.hide_done {
            parts.push("no-done".into());
        }
        if app.filter.only_high {
            parts.push("high-only".into());
        }
        if app.filter.only_overdue_or_today {
            parts.push("due-soon".into());
        }
        if parts.is_empty() {
            "all".to_string()
        } else {
            truncate_chars(&parts.join(" "), 28)
        }
    };

    let stats = format!("{} / {} done", app.done_tasks, app.total_tasks);

    let project_name = truncate_chars(
        app.persist
            .data_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project.tdl"),
        20,
    );

    let save_status = if app.persist.dirty {
        "● unsaved".to_string()
    } else {
        match app.persist.last_saved {
            Some(last) => {
                let secs = std::time::Instant::now().duration_since(last).as_secs();
                if secs < 10 {
                    "saved".to_string()
                } else if secs < 60 {
                    format!("saved {secs}s ago")
                } else {
                    format!("saved {}m ago", secs / 60)
                }
            }
            None => "not saved yet".to_string(),
        }
    };

    let current_info = if let Some(task) = app.current_task() {
        let short_title: String = task.title.chars().take(25).collect();
        format!("#{} {short_title}", task.id)
    } else {
        "No selection".to_string()
    };

    let pending_warn = if app.tree.pending_delete_path.is_some() {
        " ⚠ PENDING DELETE "
    } else {
        ""
    };

    let header_line = Line::from(vec![
        title,
        Span::raw("  •  "),
        Span::styled(
            format!("Filter: {filter_summary}"),
            Style::default().fg(LAVENDER),
        ),
        Span::raw("  •  "),
        Span::styled(stats, Style::default().fg(SAGE)),
        Span::raw("  •  "),
        Span::styled(project_name, Style::default().fg(theme::MIST)),
        Span::raw("  •  "),
        Span::styled(
            format!("Sort: {}", app.sort_mode),
            Style::default().fg(PLUM),
        ),
        Span::raw("  •  "),
        Span::styled(
            save_status,
            if app.persist.dirty {
                Style::default().fg(ROSE)
            } else {
                Style::default().fg(SKY)
            },
        ),
        Span::raw("  •  "),
        Span::styled(
            format!("Current: {current_info}"),
            Style::default().fg(PEACH),
        ),
        if app.tree.pending_delete_path.is_some() {
            Span::styled(pending_warn, Style::default().fg(ROSE).bold())
        } else {
            Span::raw("")
        },
    ]);

    let block = Block::new()
        .title(header_line)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_idle());

    frame.render_widget(block, area);
}
