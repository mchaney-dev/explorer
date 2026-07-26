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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tag::Tag;
    use crate::models::test_support::{future, past};

    // TC-TASK-001
    #[test]
    fn new_defaults() {
        let t = Task::new("Title", "Desc");
        assert!(t.status.is_none());
        assert!(t.priority.is_none());
        assert!(t.due_date.is_none());
        assert!(t.tags.is_empty());
        assert_eq!(t.created_at, t.updated_at);
    }

    // TC-TASK-002
    #[test]
    fn with_priority_sets_priority() {
        let t = Task::new("T", "").with_priority(Priority::High);
        assert!(matches!(t.priority, Some(Priority::High)));
    }

    // TC-TASK-003
    #[test]
    fn with_due_date_sets_due_date() {
        let due = future();
        let t = Task::new("T", "").with_due_date(due);
        assert_eq!(t.due_date, Some(due));
    }

    // TC-TASK-004
    #[test]
    fn with_project_sets_project_id() {
        let t = Task::new("T", "").with_project("proj-1");
        assert_eq!(t.project_id.as_deref(), Some("proj-1"));
    }

    // TC-TASK-005
    #[test]
    fn with_parent_sets_parent_id() {
        let t = Task::new("T", "").with_parent("parent-1");
        assert_eq!(t.parent_id.as_deref(), Some("parent-1"));
    }

    // TC-TASK-006
    #[test]
    fn with_recurrence_sets_recurrence_id() {
        let t = Task::new("T", "").with_recurrence("rec-1");
        assert_eq!(t.recurrence_id.as_deref(), Some("rec-1"));
    }

    // TC-TASK-007
    #[test]
    fn set_title_changes_title() {
        let mut t = Task::new("T", "");
        t.set_title("New");
        assert_eq!(t.title, "New");
        assert!(t.updated_at >= t.created_at);
    }

    // TC-TASK-008
    #[test]
    fn set_description_changes_description() {
        let mut t = Task::new("T", "");
        t.set_description("New");
        assert_eq!(t.description, "New");
        assert!(t.updated_at >= t.created_at);
    }

    // TC-TASK-009
    #[test]
    fn set_priority_changes_priority() {
        let mut t = Task::new("T", "");
        t.set_priority(Some(Priority::Low));
        assert!(matches!(t.priority, Some(Priority::Low)));
        assert!(t.updated_at >= t.created_at);
    }

    // TC-TASK-010
    #[test]
    fn set_due_date_changes_due_date() {
        let mut t = Task::new("T", "");
        let due = future();
        t.set_due_date(Some(due));
        assert_eq!(t.due_date, Some(due));
        assert!(t.updated_at >= t.created_at);
    }

    // TC-TASK-011
    #[test]
    fn mark_as_todo_sets_status_and_clears_completed_at() {
        let mut t = Task::new("T", "");
        t.mark_as_done();
        t.mark_as_todo();
        assert!(matches!(t.status, Some(Status::Todo)));
        assert!(t.completed_at.is_none());
    }

    // TC-TASK-012
    #[test]
    fn mark_as_in_progress_sets_status_and_clears_completed_at() {
        let mut t = Task::new("T", "");
        t.mark_as_done();
        t.mark_as_in_progress();
        assert!(matches!(t.status, Some(Status::InProgress)));
        assert!(t.completed_at.is_none());
    }

    // TC-TASK-013
    #[test]
    fn mark_as_done_sets_completed_at() {
        let mut t = Task::new("T", "");
        t.mark_as_done();
        assert!(matches!(t.status, Some(Status::Done)));
        assert!(t.completed_at.is_some());
    }

    // TC-TASK-014
    #[test]
    fn mark_as_cancelled_sets_status_and_clears_completed_at() {
        let mut t = Task::new("T", "");
        t.mark_as_done();
        t.mark_as_cancelled();
        assert!(matches!(t.status, Some(Status::Cancelled)));
        assert!(t.completed_at.is_none());
    }

    // TC-TASK-015
    #[test]
    fn is_done_only_when_done() {
        let mut t = Task::new("T", "");
        assert!(!t.is_done());
        t.mark_as_done();
        assert!(t.is_done());
    }

    // TC-TASK-016
    #[test]
    fn is_closed_when_done_or_cancelled() {
        let mut t = Task::new("T", "");
        assert!(!t.is_closed());
        t.mark_as_done();
        assert!(t.is_closed());
        t.mark_as_cancelled();
        assert!(t.is_closed());
    }

    // TC-TASK-017
    #[test]
    fn is_subtask_when_parent_set() {
        assert!(Task::new("T", "").with_parent("p").is_subtask());
        assert!(!Task::new("T", "").is_subtask());
    }

    // TC-TASK-018
    #[test]
    fn is_recurring_when_recurrence_set() {
        assert!(Task::new("T", "").with_recurrence("r").is_recurring());
        assert!(!Task::new("T", "").is_recurring());
    }

    // TC-TASK-019
    #[test]
    fn is_overdue_when_past_and_open() {
        let mut t = Task::new("T", "");
        t.due_date = Some(past());
        assert!(t.is_overdue());
    }

    // TC-TASK-020
    #[test]
    fn is_overdue_false_when_closed() {
        let mut t = Task::new("T", "");
        t.due_date = Some(past());
        t.mark_as_done();
        assert!(!t.is_overdue());
    }

    // TC-TASK-021
    #[test]
    fn is_overdue_false_without_due_date() {
        assert!(!Task::new("T", "").is_overdue());
    }

    // TC-TASK-022
    #[test]
    fn is_overdue_false_when_future() {
        let mut t = Task::new("T", "");
        t.due_date = Some(future());
        assert!(!t.is_overdue());
    }

    // TC-TASK-023
    #[test]
    fn add_tag_adds() {
        let mut t = Task::new("T", "");
        t.add_tag(Tag::new("a", None));
        assert_eq!(t.tags.len(), 1);
    }

    // TC-TASK-024
    #[test]
    fn add_tag_ignores_duplicate() {
        let mut t = Task::new("T", "");
        let tag = Tag::new("a", None);
        t.add_tag(tag.clone());
        t.add_tag(tag);
        assert_eq!(t.tags.len(), 1);
    }

    // TC-TASK-025
    #[test]
    fn remove_tag_present_returns_true() {
        let mut t = Task::new("T", "");
        let tag = Tag::new("a", None);
        let id = tag.id.clone();
        t.add_tag(tag);
        assert!(t.remove_tag(&id));
        assert!(t.tags.is_empty());
    }

    // TC-TASK-026
    #[test]
    fn remove_tag_absent_returns_false() {
        let mut t = Task::new("T", "");
        assert!(!t.remove_tag("nope"));
    }

    // TC-TASK-027
    #[test]
    fn has_tag_reflects_membership() {
        let mut t = Task::new("T", "");
        let tag = Tag::new("a", None);
        let id = tag.id.clone();
        t.add_tag(tag);
        assert!(t.has_tag(&id));
        assert!(!t.has_tag("nope"));
    }
}
