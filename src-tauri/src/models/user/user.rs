use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::Timestamp;
use crate::models::user::preferences::Preferences;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub preferences: Preferences,
}

impl User {
    pub fn new() -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            preferences: Preferences::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_support::past;

    // TC-USER-001
    #[test]
    fn new_has_preferences_and_equal_timestamps() {
        let u = User::new();
        assert!(!u.preferences.id.is_empty());
        assert_eq!(u.created_at, u.updated_at);
    }

    // TC-USER-002
    #[test]
    fn set_preferences_replaces_and_touches() {
        let mut u = User::new();
        let mut prefs = Preferences::new();
        prefs.set_display_name("Max");
        u.set_preferences(prefs);
        assert_eq!(u.preferences.display_name, "Max");
        assert!(u.updated_at >= u.created_at);
    }

    // TC-USER-003
    #[test]
    fn preferences_mut_bumps_updated_at() {
        let mut u = User::new();
        u.updated_at = past(); // pretend the user was last touched two days ago
        u.preferences_mut().set_display_name("Max");
        assert!(
            u.updated_at > past(),
            "editing via preferences_mut should refresh updated_at"
        );
    }

    // TC-USER-004
    #[test]
    fn default_matches_new() {
        let u = User::default();
        assert_eq!(
            u.preferences.theme,
            crate::models::user::preferences::Theme::Light
        );
    }
}
