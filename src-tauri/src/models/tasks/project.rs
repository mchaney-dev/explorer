use serde::{Deserialize, Serialize};
use chrono::Utc;
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
    pub tags: Vec<Tag>
}

impl Project {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>
    ) -> Self {
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
            tags: Vec::new()
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
