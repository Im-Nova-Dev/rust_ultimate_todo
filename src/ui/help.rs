use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::theme::{self, LAVENDER};

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub(super) fn render(frame: &mut Frame) {
    let area = centered_rect(85, 80, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::new()
        .title(" ❓ Deep Todo — Keybindings & Philosophy ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = r#"DEEP HIERARCHICAL TODO LIST

NAVIGATION
  j / k / ↑ / ↓     Move selection in the visible tree
  g / G             Jump to first / last visible item
  h / l / Space     Collapse / expand current task
  Mouse wheel       Scroll the list

MANIPULATION (very deep tree support)
  a                 Add new task as sibling after current
  A                 Add new task as CHILD of current (great for nesting)
  d                 Delete (press d twice to confirm)
  D                 Force delete without confirmation
  e / Enter         Open rich edit modal
  c                 Duplicate current task (with subtree)
  J / K             Move task up/down among siblings
  > / <  (L / H)    Indent / outdent (change parent)
  p / P             Cycle priority (Low → Med → High)
  m / M             Cycle status (Todo → Doing → Done → Blocked)
  x / X             Toggle done / todo
  b / u             Go to parent task

SEARCH & FILTER
  /                 Live search (title, desc, tags)
  f                 Cycle quick filters (hide done → high → due soon → all)
  F                 Clear all filters
  s                 Cycle sort (manual → priority → due → title)

COMMANDS & POWER
  :                 Command mode (vim-style)
  :42               Jump to task #42
  :add Buy milk     Quick add at root
  :due +3           Set due date on selection
  :priority high
  :status doing
  :tag +work -old
  :expand all
  :sort due
  :export           Export to markdown
  :open path.tdl confirm   Open a portable project file
  :save-as path.tdl Save to a new portable .tdl file
  :focus 42         Zoom tree to task #42
  :unfocus          Show full tree again

UNDO & PERSISTENCE
  Ctrl+Z            Undo last change
  Ctrl+S            Force manual save (autosave runs every ~2s when idle)
  :reload confirm   Reload current project from disk
  q / Ctrl+C        Quit from normal/help (cancels search/command/edit elsewhere)

PORTABLE PROJECTS (.tdl)
  rust_tui ~/my-project.tdl     Open/create a portable project from the shell
  Files store the full task tree, expanded nodes, next ID, and sort mode.
  Default (no argument): ~/.local/share/rust_tui/default.tdl

Header shows live filter/sort/stats + last saved time.
Right panel shows full details + subtask progress.

Philosophy: deeply nested projects deserve a tool that respects hierarchy,
not a flat checklist. This TUI is built for that.

Projects save as portable .tdl files (JSON inside). See :open and :save-as.
Press ? or Esc to close this help."#;

    let p = Paragraph::new(help_text)
        .wrap(Wrap { trim: true })
        .style(theme::body().fg(LAVENDER));

    frame.render_widget(p, inner);
}
