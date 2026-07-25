use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::Timestamp;
use crate::models::tag::Tag;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub recurrence_id: Option<String>,
    pub parent_id: Option<String>,
    pub project_id: Option<String>,
    pub completed_at: Option<Timestamp>,
    pub title: String,
    pub description: String,
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub due_date: Option<Timestamp>,
    pub tags: Vec<Tag>,
}

impl Task {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            recurrence_id: None,
            parent_id: None,
            project_id: None,
            completed_at: None,
            title: title.into(),
            description: description.into(),
            status: None,
            priority: None,
            due_date: None,
            tags: Vec::new(),
        }
    }

    /// Bump `updated_at` to now. Called by every mutator so the timestamp
    /// stays authoritative without repeating the assignment everywhere.
    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders (chain off `new`) --------------------------------------

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_due_date(mut self, due_date: Timestamp) -> Self {
        self.due_date = Some(due_date);
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    pub fn with_recurrence(mut self, recurrence_id: impl Into<String>) -> Self {
        self.recurrence_id = Some(recurrence_id.into());
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.touch();
    }

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.touch();
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        self.priority = priority;
        self.touch();
    }

    pub fn set_due_date(&mut self, due_date: Option<Timestamp>) {
        self.due_date = due_date;
        self.touch();
    }

    // -- Status transitions ---------------------------------------------

    pub fn mark_as_todo(&mut self) {
        self.status = Some(Status::Todo);
        self.completed_at = None;
        self.touch();
    }

    pub fn mark_as_in_progress(&mut self) {
        self.status = Some(Status::InProgress);
        self.completed_at = None;
        self.touch();
    }

    pub fn mark_as_done(&mut self) {
        let now = Utc::now();
        self.status = Some(Status::Done);
        self.completed_at = Some(now);
        self.updated_at = now;
    }

    pub fn mark_as_cancelled(&mut self) {
        self.status = Some(Status::Cancelled);
        self.completed_at = None;
        self.touch();
    }

    // -- Tags ------------------------------------------------------------

    pub fn add_tag(&mut self, tag: Tag) {
        if !self.has_tag(&tag.id) {
            self.tags.push(tag);
            self.touch();
        }
    }

    pub fn remove_tag(&mut self, tag_id: &str) -> bool {
        let before = self.tags.len();
        self.tags.retain(|t| t.id != tag_id);
        let removed = self.tags.len() != before;
        if removed {
            self.touch();
        }
        removed
    }

    pub fn has_tag(&self, tag_id: &str) -> bool {
        self.tags.iter().any(|t| t.id == tag_id)
    }

    // -- Queries ---------------------------------------------------------

    pub fn is_done(&self) -> bool {
        matches!(self.status, Some(Status::Done))
    }

    /// Done or cancelled — i.e. no longer actionable.
    pub fn is_closed(&self) -> bool {
        matches!(self.status, Some(Status::Done) | Some(Status::Cancelled))
    }

    pub fn is_subtask(&self) -> bool {
        self.parent_id.is_some()
    }

    pub fn is_recurring(&self) -> bool {
        self.recurrence_id.is_some()
    }

    /// True when the task has a due date in the past and is still open.
    pub fn is_overdue(&self) -> bool {
        if self.is_closed() {
            return false;
        }
        match self.due_date {
            Some(due) => due < Utc::now(),
            None => false,
        }
    }
}
