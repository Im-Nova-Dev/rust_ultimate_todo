use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, Mode};
use crate::theme::{self, MOSS, SAND};

fn truncate_to_width(text: &str, max_cols: u16) -> String {
    let max = max_cols as usize;
    if max == 0 {
        return String::new();
    }
    let char_count = text.chars().count();
    if char_count <= max {
        return text.to_string();
    }
    if max <= 1 {
        return "…".to_string();
    }
    format!("{}…", text.chars().take(max - 1).collect::<String>())
}

pub(super) fn render(frame: &mut Frame, area: Rect, app: &App) {
    let (text, style) = match app.mode {
        Mode::Command => (
            format!(":{}", app.command_buf),
            Style::default().fg(SAND).bold(),
        ),
        _ => {
            let mut hints = vec![
                "j/k nav",
                "h/l/Spc expand",
                "a/A add",
                "e edit",
                "p/m cycle",
                "x toggle done",
                "/ search",
                ": cmd",
                "? help",
            ];
            if !app.undo_stack.is_empty() {
                hints.push("Ctrl+Z undo");
            }
            (hints.join("  •  "), theme::muted())
        }
    };

    let max_cols = area.width.saturating_sub(2);
    let line = if !app.message.is_empty() && app.mode == Mode::Normal {
        let msg = truncate_to_width(&app.message, max_cols.saturating_sub(text.len() as u16 + 5));
        Line::from(vec![
            Span::styled(msg, Style::default().fg(MOSS)),
            Span::raw("   •   "),
            Span::styled(text, style),
        ])
    } else {
        Line::from(Span::styled(truncate_to_width(&text, max_cols), style))
    };

    let block = Block::new()
        .borders(Borders::TOP)
        .border_style(theme::border_idle());

    let p = Paragraph::new(line)
        .alignment(Alignment::Center)
        .block(block);

    frame.render_widget(p, area);
}
