use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::hex_color::HexColor;
use crate::models::shared::Timestamp;
use crate::models::tag::Tag;
use crate::models::tasks::task::Task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub tasks: Vec<Task>,
    pub is_archived: bool,
    pub title: String,
    pub description: String,
    pub color: Option<HexColor>,
    pub tags: Vec<Tag>,
}

impl Project {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            tasks: Vec::new(),
            is_archived: false,
            title: title.into(),
            description: description.into(),
            color: None,
            tags: Vec::new(),
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_color(mut self, color: HexColor) -> Self {
        self.color = Some(color);
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

    pub fn set_color(&mut self, color: Option<HexColor>) {
        self.color = color;
        self.touch();
    }

    pub fn archive(&mut self) {
        self.is_archived = true;
        self.touch();
    }

    pub fn unarchive(&mut self) {
        self.is_archived = false;
        self.touch();
    }

    // -- Tasks -----------------------------------------------------------

    pub fn add_task(&mut self, mut task: Task) {
        task.project_id = Some(self.id.clone());
        self.tasks.push(task);
        self.touch();
    }

    pub fn remove_task(&mut self, task_id: &str) -> Option<Task> {
        let idx = self.tasks.iter().position(|t| t.id == task_id)?;
        let task = self.tasks.remove(idx);
        self.touch();
        Some(task)
    }

    pub fn find_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    pub fn find_task_mut(&mut self, task_id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id)
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn completed_task_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.is_done()).count()
    }

    /// Completion ratio in `0.0..=1.0`; an empty project reports `0.0`.
    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        self.completed_task_count() as f64 / self.tasks.len() as f64
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tag::Tag;
    use crate::models::tasks::task::Task;

    fn done_task() -> Task {
        let mut t = Task::new("done", "");
        t.mark_as_done();
        t
    }

    // TC-PROJ-001
    #[test]
    fn new_defaults() {
        let p = Project::new("P", "d");
        assert!(!p.is_archived);
        assert!(p.tasks.is_empty());
        assert!(p.tags.is_empty());
    }

    // TC-PROJ-002
    #[test]
    fn with_color_sets_color() {
        let p = Project::new("P", "d").with_color("#111".to_string());
        assert_eq!(p.color.as_deref(), Some("#111"));
    }

    // TC-PROJ-003
    #[test]
    fn setters_change_fields() {
        let mut p = Project::new("P", "d");
        p.set_title("New");
        p.set_description("nd");
        p.set_color(Some("#222".to_string()));
        assert_eq!(p.title, "New");
        assert_eq!(p.description, "nd");
        assert_eq!(p.color.as_deref(), Some("#222"));
        assert!(p.updated_at >= p.created_at);
    }

    // TC-PROJ-004
    #[test]
    fn archive_and_unarchive() {
        let mut p = Project::new("P", "d");
        p.archive();
        assert!(p.is_archived);
        p.unarchive();
        assert!(!p.is_archived);
    }

    // TC-PROJ-005
    #[test]
    fn add_task_appends_and_stamps_project_id() {
        let mut p = Project::new("P", "d");
        p.add_task(Task::new("t", ""));
        assert_eq!(p.task_count(), 1);
        assert_eq!(p.tasks[0].project_id.as_deref(), Some(p.id.as_str()));
    }

    // TC-PROJ-006
    #[test]
    fn remove_task_present_returns_it() {
        let mut p = Project::new("P", "d");
        let t = Task::new("t", "");
        let id = t.id.clone();
        p.add_task(t);
        assert!(p.remove_task(&id).is_some());
        assert!(p.is_empty());
    }

    // TC-PROJ-007
    #[test]
    fn find_task_and_mut() {
        let mut p = Project::new("P", "d");
        let t = Task::new("t", "");
        let id = t.id.clone();
        p.add_task(t);
        assert!(p.find_task(&id).is_some());
        p.find_task_mut(&id).unwrap().set_title("Edited");
        assert_eq!(p.find_task(&id).unwrap().title, "Edited");
    }

    // TC-PROJ-008
    #[test]
    fn task_count_and_is_empty() {
        let mut p = Project::new("P", "d");
        assert!(p.is_empty());
        p.add_task(Task::new("t", ""));
        assert_eq!(p.task_count(), 1);
        assert!(!p.is_empty());
    }

    // TC-PROJ-009
    #[test]
    fn completed_task_count_counts_done() {
        let mut p = Project::new("P", "d");
        p.add_task(done_task());
        p.add_task(Task::new("open", ""));
        assert_eq!(p.completed_task_count(), 1);
    }

    // TC-PROJ-010
    #[test]
    fn progress_is_completed_over_total() {
        let mut p = Project::new("P", "d");
        p.add_task(done_task());
        p.add_task(Task::new("open", ""));
        assert_eq!(p.progress(), 0.5);
    }

    // TC-PROJ-011
    #[test]
    fn progress_zero_for_empty_project() {
        assert_eq!(Project::new("P", "d").progress(), 0.0);
    }

    // TC-PROJ-012
    #[test]
    fn add_tag_adds() {
        let mut p = Project::new("P", "d");
        p.add_tag(Tag::new("t", None));
        assert_eq!(p.tags.len(), 1);
    }

    // TC-PROJ-013
    #[test]
    fn add_tag_ignores_duplicate() {
        let mut p = Project::new("P", "d");
        let tag = Tag::new("t", None);
        p.add_tag(tag.clone());
        p.add_tag(tag);
        assert_eq!(p.tags.len(), 1);
    }

    // TC-PROJ-014
    #[test]
    fn remove_tag_present_returns_true() {
        let mut p = Project::new("P", "d");
        let tag = Tag::new("t", None);
        let id = tag.id.clone();
        p.add_tag(tag);
        assert!(p.remove_tag(&id));
    }

    // TC-PROJ-015
    #[test]
    fn remove_tag_absent_returns_false() {
        assert!(!Project::new("P", "d").remove_tag("nope"));
    }

    // TC-PROJ-016
    #[test]
    fn has_tag_reflects_membership() {
        let mut p = Project::new("P", "d");
        let tag = Tag::new("t", None);
        let id = tag.id.clone();
        p.add_tag(tag);
        assert!(p.has_tag(&id));
        assert!(!p.has_tag("nope"));
    }
}
