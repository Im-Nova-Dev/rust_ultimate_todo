use ratatui::Frame;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::app::App;
use crate::theme::{self, LAVENDER, PLUM, SAGE, SAND};

use super::help::centered_rect;

pub(super) fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect(78, 72, frame.area());
    frame.render_widget(Clear, area);

    let Some(ed) = &app.edit else { return };

    let block = Block::new()
        .title(format!(
            " ✏️  Editing Task #{} — Tab fields • Ctrl+Enter / Ctrl+S save • Esc cancel ",
            ed.working.id
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(SAND));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let field_names = [
        "Title",
        "Description (multi-line)",
        "Priority (←/→)",
        "Status (←/→)",
        "Due date (relative or YYYY-MM-DD)",
        "Tags (type + Enter to add)",
    ];

    let mut lines = vec![
        Line::from(Span::styled(
            format!(
                "Field {} / 6: {}",
                ed.current_field + 1,
                field_names[ed.current_field]
            ),
            Style::default().fg(LAVENDER).bold(),
        )),
        Line::default(),
    ];

    let title_style = if ed.current_field == 0 {
        theme::field_active()
    } else {
        theme::body()
    };
    lines.push(Line::from(vec![
        Span::styled("Title: ", theme::label()),
        Span::styled(&ed.title_buf, title_style),
    ]));
    lines.push(Line::default());

    let desc_style = if ed.current_field == 1 {
        theme::field_active()
    } else {
        theme::body()
    };
    lines.push(Line::from(Span::styled(
        "Description (Ctrl+A/E/K/U in field):",
        theme::label(),
    )));
    for (i, line) in ed.desc_lines.iter().enumerate() {
        let marker = if ed.current_field == 1 && i == ed.desc_row {
            "▶ "
        } else {
            "  "
        };
        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(line.clone(), desc_style),
        ]));
    }
    lines.push(Line::default());

    let prio_style = if ed.current_field == 2 {
        theme::field_active_priority(&ed.working.priority)
    } else {
        Style::default().fg(ed.working.priority.color())
    };
    lines.push(Line::from(vec![
        Span::styled("Priority: ", theme::label()),
        Span::styled(ed.working.priority.as_str(), prio_style),
    ]));
    lines.push(Line::default());

    let status_style = if ed.current_field == 3 {
        theme::field_active_status(&ed.working.status)
    } else {
        Style::default().fg(ed.working.status.color())
    };
    lines.push(Line::from(vec![
        Span::styled("Status: ", theme::label()),
        Span::styled(ed.working.status.as_str(), status_style),
    ]));
    lines.push(Line::default());

    let due_style = if ed.current_field == 4 {
        theme::field_active()
    } else {
        theme::body()
    };
    let due_preview = ed
        .parsed_due_preview
        .map(|d| format!("  → parsed: {d}"))
        .unwrap_or_else(|| "  → (invalid or empty)".to_string());
    lines.push(Line::from(vec![
        Span::styled("Due: ", theme::label()),
        Span::styled(&ed.due_buf, due_style),
        Span::styled(due_preview, Style::default().fg(SAGE)),
    ]));
    lines.push(Line::default());

    let tags_display = ed.working.tags.join(", ");
    let tag_style = if ed.current_field == 5 {
        theme::field_active()
    } else {
        theme::body()
    };
    lines.push(Line::from(vec![
        Span::styled("Tags: ", theme::label()),
        Span::styled(tags_display, Style::default().fg(PLUM)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Add tag: "),
        Span::styled(&ed.tag_buf, tag_style),
        Span::raw("  (Ctrl+D to remove last)"),
    ]));

    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(theme::body());

    frame.render_widget(p, inner);
}
