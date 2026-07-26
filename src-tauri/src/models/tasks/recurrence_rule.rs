use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub frequency: Option<Frequency>,
    pub interval: Option<i32>,
    pub days_of_week: Vec<DayOfWeek>,
    pub day_of_month: Option<i32>,
    pub end_date: Option<Timestamp>,
}

impl RecurrenceRule {
    pub fn new() -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            frequency: None,
            interval: None,
            days_of_week: Vec::new(),
            day_of_month: None,
            end_date: None,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_frequency(mut self, frequency: Frequency) -> Self {
        self.frequency = Some(frequency);
        self
    }

    pub fn with_interval(mut self, interval: i32) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn with_days_of_week(mut self, days: Vec<DayOfWeek>) -> Self {
        self.days_of_week = days;
        self
    }

    pub fn with_day_of_month(mut self, day: i32) -> Self {
        self.day_of_month = Some(day);
        self
    }

    pub fn with_end_date(mut self, end_date: Timestamp) -> Self {
        self.end_date = Some(end_date);
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_frequency(&mut self, frequency: Option<Frequency>) {
        self.frequency = frequency;
        self.touch();
    }

    pub fn set_interval(&mut self, interval: Option<i32>) {
        self.interval = interval;
        self.touch();
    }

    pub fn add_day(&mut self, day: DayOfWeek) {
        if !self.days_of_week.contains(&day) {
            self.days_of_week.push(day);
            self.touch();
        }
    }

    pub fn remove_day(&mut self, day: DayOfWeek) {
        let before = self.days_of_week.len();
        self.days_of_week.retain(|d| *d != day);
        if self.days_of_week.len() != before {
            self.touch();
        }
    }

    // -- Queries ---------------------------------------------------------

    /// The effective repeat interval, treating an unset interval as `1`.
    pub fn effective_interval(&self) -> i32 {
        self.interval.unwrap_or(1).max(1)
    }

    /// True once the rule's end date is in the past.
    pub fn has_ended(&self) -> bool {
        match self.end_date {
            Some(end) => end < Utc::now(),
            None => false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.frequency.is_some() && !self.has_ended()
    }
}

impl Default for RecurrenceRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_support::{future, past};

    // TC-RECUR-001
    #[test]
    fn new_defaults() {
        let r = RecurrenceRule::new();
        assert!(r.frequency.is_none());
        assert!(r.interval.is_none());
        assert!(r.day_of_month.is_none());
        assert!(r.days_of_week.is_empty());
        assert!(r.end_date.is_none());
    }

    // TC-RECUR-002
    #[test]
    fn with_frequency_sets_it() {
        let r = RecurrenceRule::new().with_frequency(Frequency::Weekly);
        assert!(matches!(r.frequency, Some(Frequency::Weekly)));
    }

    // TC-RECUR-003
    #[test]
    fn with_interval_sets_it() {
        assert_eq!(RecurrenceRule::new().with_interval(3).interval, Some(3));
    }

    // TC-RECUR-004
    #[test]
    fn with_days_of_week_sets_them() {
        let r = RecurrenceRule::new().with_days_of_week(vec![DayOfWeek::Monday, DayOfWeek::Friday]);
        assert_eq!(r.days_of_week.len(), 2);
    }

    // TC-RECUR-005
    #[test]
    fn with_day_of_month_sets_it() {
        assert_eq!(
            RecurrenceRule::new().with_day_of_month(15).day_of_month,
            Some(15)
        );
    }

    // TC-RECUR-006
    #[test]
    fn with_end_date_sets_it() {
        let end = future();
        assert_eq!(RecurrenceRule::new().with_end_date(end).end_date, Some(end));
    }

    // TC-RECUR-007
    #[test]
    fn set_frequency_and_interval() {
        let mut r = RecurrenceRule::new();
        r.set_frequency(Some(Frequency::Daily));
        r.set_interval(Some(2));
        assert!(matches!(r.frequency, Some(Frequency::Daily)));
        assert_eq!(r.interval, Some(2));
        assert!(r.updated_at >= r.created_at);
    }

    // TC-RECUR-008
    #[test]
    fn add_day_adds() {
        let mut r = RecurrenceRule::new();
        r.add_day(DayOfWeek::Monday);
        assert_eq!(r.days_of_week.len(), 1);
    }

    // TC-RECUR-009
    #[test]
    fn add_day_ignores_duplicate() {
        let mut r = RecurrenceRule::new();
        r.add_day(DayOfWeek::Monday);
        r.add_day(DayOfWeek::Monday);
        assert_eq!(r.days_of_week.len(), 1);
    }

    // TC-RECUR-010
    #[test]
    fn remove_day_removes() {
        let mut r = RecurrenceRule::new();
        r.add_day(DayOfWeek::Monday);
        r.remove_day(DayOfWeek::Monday);
        assert!(r.days_of_week.is_empty());
    }

    // TC-RECUR-011
    #[test]
    fn effective_interval_returns_set_value() {
        assert_eq!(
            RecurrenceRule::new().with_interval(4).effective_interval(),
            4
        );
    }

    // TC-RECUR-012
    #[test]
    fn effective_interval_defaults_to_one() {
        assert_eq!(RecurrenceRule::new().effective_interval(), 1);
    }

    // TC-RECUR-013
    #[test]
    fn effective_interval_floors_at_one() {
        assert_eq!(
            RecurrenceRule::new().with_interval(0).effective_interval(),
            1
        );
    }

    // TC-RECUR-014
    #[test]
    fn has_ended_true_when_past() {
        assert!(RecurrenceRule::new().with_end_date(past()).has_ended());
    }

    // TC-RECUR-015
    #[test]
    fn has_ended_false_when_future() {
        assert!(!RecurrenceRule::new().with_end_date(future()).has_ended());
    }

    // TC-RECUR-016
    #[test]
    fn has_ended_false_without_end_date() {
        assert!(!RecurrenceRule::new().has_ended());
    }

    // TC-RECUR-017
    #[test]
    fn is_active_when_frequency_set_and_not_ended() {
        let r = RecurrenceRule::new()
            .with_frequency(Frequency::Daily)
            .with_end_date(future());
        assert!(r.is_active());
    }

    // TC-RECUR-018
    #[test]
    fn is_active_false_without_frequency() {
        assert!(!RecurrenceRule::new().is_active());
    }

    // TC-RECUR-019
    #[test]
    fn is_active_false_when_ended() {
        let r = RecurrenceRule::new()
            .with_frequency(Frequency::Daily)
            .with_end_date(past());
        assert!(!r.is_active());
    }
}
