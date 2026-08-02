//! Test-only helpers for building deterministic timestamps.
//!
//! Time-based behaviors (is_overdue, has_ended, feed scoring) need fixed points
//! in time to test reliably. Model fields are `pub`, so a test can assign
//! `created_at`/`due_date`/`end_date` directly from these helpers.

use chrono::{Duration, Utc};

use crate::models::shared::Timestamp;

/// A timestamp two days in the past.
pub fn past() -> Timestamp {
    Utc::now() - Duration::days(2)
}

/// A timestamp two days in the future.
pub fn future() -> Timestamp {
    Utc::now() + Duration::days(2)
}
