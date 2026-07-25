use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::feed::feed_source::FeedSource;
use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedInterest {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub last_interacted_at: Option<Timestamp>,
    pub topic: String,
    pub sources: Vec<FeedSource>,
    pub explicit_weight: Option<f64>,
    pub open_count: i32,
    pub save_count: i32,
    pub dismiss_count: i32,
}

impl FeedInterest {
    pub fn new(topic: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            last_interacted_at: None,
            topic: topic.into(),
            sources: Vec::new(),
            explicit_weight: None,
            open_count: 0,
            save_count: 0,
            dismiss_count: 0,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Stamp the most-recent-interaction time to now and bump `updated_at`.
    fn mark_interacted(&mut self) {
        let now = Utc::now();
        self.last_interacted_at = Some(now);
        self.updated_at = now;
    }

    // -- Interactions ----------------------------------------------------

    pub fn record_open(&mut self) {
        self.open_count += 1;
        self.mark_interacted();
    }

    pub fn record_save(&mut self) {
        self.save_count += 1;
        self.mark_interacted();
    }

    pub fn record_dismiss(&mut self) {
        self.dismiss_count += 1;
        self.mark_interacted();
    }

    pub fn set_weight(&mut self, weight: Option<f64>) {
        self.explicit_weight = weight;
        self.touch();
    }

    // -- Sources ---------------------------------------------------------

    pub fn add_source(&mut self, source: FeedSource) {
        if !self.has_source(&source.id) {
            self.sources.push(source);
            self.touch();
        }
    }

    pub fn remove_source(&mut self, source_id: &str) -> Option<FeedSource> {
        let idx = self.sources.iter().position(|s| s.id == source_id)?;
        let source = self.sources.remove(idx);
        self.touch();
        Some(source)
    }

    pub fn has_source(&self, source_id: &str) -> bool {
        self.sources.iter().any(|s| s.id == source_id)
    }

    // -- Scoring ---------------------------------------------------------

    /// Behavioural signal derived from interactions, decayed by the interest's
    /// age so long-lived interests don't dominate purely by accumulation.
    pub fn implicit_score(&self) -> f64 {
        let days = (Utc::now() - self.created_at).num_days().max(1) as f64;

        (self.open_count as f64 * 1.0
            + self.save_count as f64 * 2.5
            - self.dismiss_count as f64 * 1.5)
            / days
    }

    /// Overall ranking score: the user's explicit weight (if set) plus the
    /// implicit behavioural score.
    pub fn score(&self) -> f64 {
        self.explicit_weight.unwrap_or(0.0) + self.implicit_score()
    }
}
