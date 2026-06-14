use chrono::Local;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::{self, LAVENDER, PEACH, SAGE};

pub(super) fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::new()
        .title(" 🔍 Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_idle());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(task) = app.current_task() else {
        let p = Paragraph::new("No task selected.\n\nNavigate with j/k.\nAdd with a or A.")
            .style(theme::muted());
        frame.render_widget(p, inner);
        return;
    };

    let (done, total) = app.subtask_progress(task);
    let progress = if total > 0 {
        format!(
            "Subtasks: {}/{} ({:.0}%)",
            done,
            total,
            (done as f32 / total as f32) * 100.0
        )
    } else {
        "No subtasks".to_string()
    };

    let due_line = if let Some(d) = task.due {
        let today = Local::now().date_naive();
        let rel = if d < today {
            format!("OVERDUE by {} day(s)", (today - d).num_days())
        } else if d == today {
            "DUE TODAY".to_string()
        } else {
            format!(
                "Due in {} day(s) — {}",
                (d - today).num_days(),
                d.format("%Y-%m-%d")
            )
        };
        format!("Due: {}\n", rel)
    } else {
        "Due: (none)\n".to_string()
    };

    let tags_line = if task.tags.is_empty() {
        "Tags: (none)".to_string()
    } else {
        format!("Tags: {}", task.tags.join(", "))
    };

    let desc = if task.desc.is_empty() {
        "(no description)".to_string()
    } else {
        task.desc.clone()
    };

    let filtered_out = !app
        .tree
        .visible
        .iter()
        .any(|v| v.path == app.tree.selected_path);
    let filter_note = if filtered_out {
        vec![Line::from(Span::styled(
            "(hidden by current filters — press F to show)",
            Style::default().fg(PEACH).italic(),
        ))]
    } else {
        vec![]
    };

    let mut content = vec![
        Line::from(vec![
            Span::styled("Title: ", theme::label()),
            Span::styled(&task.title, theme::label()),
        ]),
        Line::from(vec![
            Span::styled("Path: ", theme::label()),
            Span::styled(app.get_breadcrumb(), Style::default().fg(LAVENDER)),
        ]),
        Line::from(vec![
            Span::styled("Status: ", theme::label()),
            Span::styled(
                task.status.as_str(),
                Style::default().fg(task.status.color()).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Priority: ", theme::label()),
            Span::styled(
                task.priority.as_str(),
                Style::default().fg(task.priority.color()).bold(),
            ),
        ]),
        Line::from(vec![Span::styled(due_line, theme::body())]),
        Line::from(vec![Span::styled(tags_line, theme::body())]),
        Line::default(),
        Line::from(Span::styled("Description:", theme::label().underlined())),
        Line::from(Span::styled(desc, theme::body())),
        Line::default(),
        Line::from(Span::styled(progress, Style::default().fg(SAGE))),
    ];
    content.extend(filter_note);

    let p = Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .block(Block::new());

    frame.render_widget(p, inner);
}
