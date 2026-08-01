use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::feed::feed_source::FeedSource;
use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
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

        (self.open_count as f64 * 1.0 + self.save_count as f64 * 2.5
            - self.dismiss_count as f64 * 1.5)
            / days
    }

    /// Overall ranking score: the user's explicit weight (if set) plus the
    /// implicit behavioural score.
    pub fn score(&self) -> f64 {
        self.explicit_weight.unwrap_or(0.0) + self.implicit_score()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-FEEDINT-001
    #[test]
    fn new_defaults() {
        let i = FeedInterest::new("rust");
        assert_eq!(i.open_count, 0);
        assert_eq!(i.save_count, 0);
        assert_eq!(i.dismiss_count, 0);
        assert!(i.sources.is_empty());
        assert!(i.explicit_weight.is_none());
        assert!(i.last_interacted_at.is_none());
    }

    // TC-FEEDINT-002
    #[test]
    fn record_open_increments_and_stamps() {
        let mut i = FeedInterest::new("rust");
        i.record_open();
        assert_eq!(i.open_count, 1);
        assert!(i.last_interacted_at.is_some());
    }

    // TC-FEEDINT-003
    #[test]
    fn record_save_increments_and_stamps() {
        let mut i = FeedInterest::new("rust");
        i.record_save();
        assert_eq!(i.save_count, 1);
        assert!(i.last_interacted_at.is_some());
    }

    // TC-FEEDINT-004
    #[test]
    fn record_dismiss_increments_and_stamps() {
        let mut i = FeedInterest::new("rust");
        i.record_dismiss();
        assert_eq!(i.dismiss_count, 1);
        assert!(i.last_interacted_at.is_some());
    }

    // TC-FEEDINT-005
    #[test]
    fn set_weight_sets_it() {
        let mut i = FeedInterest::new("rust");
        i.set_weight(Some(2.0));
        assert_eq!(i.explicit_weight, Some(2.0));
    }

    // TC-FEEDINT-006
    #[test]
    fn add_source_adds() {
        let mut i = FeedInterest::new("rust");
        i.add_source(FeedSource::new("s", "u"));
        assert_eq!(i.sources.len(), 1);
    }

    // TC-FEEDINT-007
    #[test]
    fn add_source_ignores_duplicate() {
        let mut i = FeedInterest::new("rust");
        let s = FeedSource::new("s", "u");
        i.add_source(s.clone());
        i.add_source(s);
        assert_eq!(i.sources.len(), 1);
    }

    // TC-FEEDINT-008
    #[test]
    fn remove_source_present_returns_it() {
        let mut i = FeedInterest::new("rust");
        let s = FeedSource::new("s", "u");
        let id = s.id.clone();
        i.add_source(s);
        assert!(i.remove_source(&id).is_some());
    }

    // TC-FEEDINT-009
    #[test]
    fn has_source_reflects_membership() {
        let mut i = FeedInterest::new("rust");
        let s = FeedSource::new("s", "u");
        let id = s.id.clone();
        i.add_source(s);
        assert!(i.has_source(&id));
        assert!(!i.has_source("nope"));
    }

    // TC-FEEDINT-010
    #[test]
    fn implicit_score_moves_in_expected_direction() {
        let baseline = FeedInterest::new("rust");

        let mut active = FeedInterest::new("rust");
        active.record_open();
        active.record_open();
        active.record_save();

        let mut dismissed = FeedInterest::new("rust");
        dismissed.record_dismiss();

        assert!(active.implicit_score() > baseline.implicit_score());
        assert!(dismissed.implicit_score() < baseline.implicit_score());
    }

    // TC-FEEDINT-011
    #[test]
    fn score_ranks_weighted_interest_higher() {
        let plain = FeedInterest::new("rust");
        let mut weighted = FeedInterest::new("rust");
        weighted.set_weight(Some(10.0));
        assert!(weighted.score() > plain.score());
    }
}
