//! Core task model: priority, status, and nested task tree data.

use chrono::NaiveDate;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::theme;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub(crate) fn cycle(&self) -> Self {
        match self {
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::Low,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "LOW",
            Priority::Medium => "MED",
            Priority::High => "HIGH",
        }
    }

    pub(crate) fn color(&self) -> Color {
        theme::priority(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Todo,
    Doing,
    Done,
    Blocked,
}

impl Status {
    pub(crate) fn cycle(&self) -> Self {
        match self {
            Status::Todo => Status::Doing,
            Status::Doing => Status::Done,
            Status::Done => Status::Todo,
            Status::Blocked => Status::Todo,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Status::Todo => "TODO",
            Status::Doing => "DOING",
            Status::Done => "DONE",
            Status::Blocked => "BLOCK",
        }
    }

    pub(crate) fn symbol(&self) -> &'static str {
        match self {
            Status::Todo => "☐",
            Status::Doing => "◐",
            Status::Done => "✔",
            Status::Blocked => "✖",
        }
    }

    pub(crate) fn color(&self) -> Color {
        theme::status(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub desc: String,
    pub priority: Priority,
    pub status: Status,
    #[serde(default)]
    pub due: Option<NaiveDate>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub children: Vec<Task>,
}

impl Task {
    pub fn new(id: u64, title: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            desc: String::new(),
            priority: Priority::Medium,
            status: Status::Todo,
            due: None,
            tags: vec![],
            children: vec![],
        }
    }

    pub(crate) fn matches_search(&self, query_lower: &str) -> bool {
        if query_lower.is_empty() {
            return true;
        }
        contains_insensitive(&self.title, query_lower)
            || contains_insensitive(&self.desc, query_lower)
            || self
                .tags
                .iter()
                .any(|t| contains_insensitive(t, query_lower))
    }

    pub(crate) fn has_matching_descendant(&self, query_lower: &str) -> bool {
        if self.matches_search(query_lower) {
            return true;
        }
        self.children
            .iter()
            .any(|c| c.has_matching_descendant(query_lower))
    }
}

fn contains_insensitive(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    haystack.to_lowercase().contains(needle_lower)
}
