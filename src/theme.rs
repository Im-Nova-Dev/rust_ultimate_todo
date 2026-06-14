//! Soft dusk palette — all colors are explicit RGB, no terminal named colors.

use chrono::NaiveDate;
use ratatui::style::{Color, Modifier, Style};

use crate::model::{Priority, Status};

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

// Base tones
pub const CREAM: Color = rgb(228, 224, 214);
pub const MIST: Color = rgb(168, 164, 158);
pub const FOG: Color = rgb(108, 106, 114);
pub const SLATE: Color = rgb(62, 64, 72);
pub const DEEP: Color = rgb(48, 50, 58);

// Accents
pub const LAVENDER: Color = rgb(186, 178, 212);
pub const SAGE: Color = rgb(156, 188, 168);
pub const SAND: Color = rgb(210, 186, 148);
pub const ROSE: Color = rgb(212, 158, 158);
pub const SKY: Color = rgb(148, 180, 204);
pub const PLUM: Color = rgb(188, 162, 194);
pub const PEACH: Color = rgb(220, 196, 168);
pub const MOSS: Color = rgb(134, 164, 148);
pub const DUST: Color = rgb(148, 142, 152);

pub fn priority(p: &Priority) -> Color {
    match p {
        Priority::Low => SAGE,
        Priority::Medium => SAND,
        Priority::High => ROSE,
    }
}

pub fn status(s: &Status) -> Color {
    match s {
        Status::Todo => DUST,
        Status::Doing => SKY,
        Status::Done => SAGE,
        Status::Blocked => PLUM,
    }
}

pub fn due(today: NaiveDate, due: Option<NaiveDate>) -> Color {
    match due {
        Some(d) if d < today => ROSE,
        Some(d) if d == today => SAND,
        Some(_) => SKY,
        None => MIST,
    }
}

pub fn border_focused() -> Style {
    Style::default().fg(SKY)
}

pub fn border_idle() -> Style {
    Style::default().fg(FOG)
}

pub fn label() -> Style {
    Style::default().fg(CREAM).bold()
}

pub fn body() -> Style {
    Style::default().fg(CREAM)
}

pub fn muted() -> Style {
    Style::default().fg(MIST)
}

pub fn field_active() -> Style {
    Style::default().bg(SLATE).fg(CREAM)
}

pub fn field_active_priority(p: &Priority) -> Style {
    Style::default().bg(SLATE).fg(priority(p))
}

pub fn field_active_status(s: &Status) -> Style {
    Style::default().bg(SLATE).fg(status(s))
}

pub fn tree_row_selected() -> Style {
    Style::default().fg(CREAM).bg(DEEP).bold()
}

pub fn tree_row_done() -> Style {
    Style::default()
        .fg(MIST)
        .add_modifier(Modifier::CROSSED_OUT)
}

pub fn tree_row_pending_delete() -> Style {
    Style::default().fg(CREAM).bg(rgb(108, 72, 78)).bold()
}

pub fn tree_connector() -> Style {
    Style::default().fg(FOG)
}
