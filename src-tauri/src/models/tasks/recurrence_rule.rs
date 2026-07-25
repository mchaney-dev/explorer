use serde::{Deserialize, Serialize};
use chrono::Utc;
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
