mod bottom_bar;
mod details;
mod edit;
mod header;
mod help;
mod tree;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::app::{App, Mode};

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let root = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(3),
    ])
    .split(area);

    header::render(frame, root[0], app);
    let main_chunks =
        Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)]).split(root[1]);
    tree::render(frame, main_chunks[0], app);
    details::render(frame, main_chunks[1], app);
    bottom_bar::render(frame, root[2], app);

    if app.mode == Mode::Editing {
        edit::render(frame, app);
    } else if app.mode == Mode::Help {
        help::render(frame);
    }
}
