use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::shared::Timestamp;
use crate::models::user::preferences::Preferences;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub preferences: Preferences
}

impl User {
    pub fn new() -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            preferences: Preferences::new()
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn set_preferences(&mut self, preferences: Preferences) {
        self.preferences = preferences;
        self.touch();
    }

    /// Mutable access to preferences that keeps the user's own `updated_at`
    /// in sync with the edit.
    pub fn preferences_mut(&mut self) -> &mut Preferences {
        self.touch();
        &mut self.preferences
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}
