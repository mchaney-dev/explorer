use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::feed::feed_interest::FeedInterest;
use crate::models::shared::{HexColor, Timestamp};
use crate::models::user::features::Features;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DefaultView {
    Graph,
    Feed,
    Tasks,
    Notes,
    Calendar,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Preferences {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub feed_interests: Vec<FeedInterest>,
    pub display_name: String,
    pub theme: Theme,
    pub accent_color: Option<HexColor>,
    pub features: Features,
    pub sidebar_collapsed: bool,
    pub default_view: DefaultView,
}

impl Preferences {
    pub fn new() -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            feed_interests: Vec::new(),
            display_name: String::new(),
            theme: Theme::Light,
            accent_color: None,
            features: Features::new(),
            sidebar_collapsed: false,
            default_view: DefaultView::Graph,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_display_name(&mut self, display_name: impl Into<String>) {
        self.display_name = display_name.into();
        self.touch();
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.touch();
    }

    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        self.touch();
    }

    pub fn set_accent_color(&mut self, accent_color: Option<HexColor>) {
        self.accent_color = accent_color;
        self.touch();
    }

    pub fn set_default_view(&mut self, default_view: DefaultView) {
        self.default_view = default_view;
        self.touch();
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool) {
        self.sidebar_collapsed = collapsed;
        self.touch();
    }

    pub fn toggle_sidebar(&mut self) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        self.touch();
    }

    // -- Feed interests --------------------------------------------------

    pub fn add_feed_interest(&mut self, interest: FeedInterest) {
        self.feed_interests.push(interest);
        self.touch();
    }

    pub fn remove_feed_interest(&mut self, interest_id: &str) -> Option<FeedInterest> {
        let idx = self
            .feed_interests
            .iter()
            .position(|i| i.id == interest_id)?;
        let interest = self.feed_interests.remove(idx);
        self.touch();
        Some(interest)
    }

    pub fn find_feed_interest(&self, interest_id: &str) -> Option<&FeedInterest> {
        self.feed_interests.iter().find(|i| i.id == interest_id)
    }

    pub fn find_feed_interest_mut(&mut self, interest_id: &str) -> Option<&mut FeedInterest> {
        self.feed_interests.iter_mut().find(|i| i.id == interest_id)
    }

    /// Feed interests sorted by descending ranking score — the order the feed
    /// would surface them in.
    pub fn ranked_interests(&self) -> Vec<&FeedInterest> {
        let mut interests: Vec<&FeedInterest> = self.feed_interests.iter().collect();
        interests.sort_by(|a, b| {
            b.score()
                .partial_cmp(&a.score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        interests
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-PREF-001
    #[test]
    fn new_defaults() {
        let p = Preferences::new();
        assert_eq!(p.theme, Theme::Light);
        assert_eq!(p.default_view, DefaultView::Graph);
        assert!(!p.sidebar_collapsed);
        assert!(p.feed_interests.is_empty());
    }

    // TC-PREF-002
    #[test]
    fn set_display_name_changes_it() {
        let mut p = Preferences::new();
        p.set_display_name("Max");
        assert_eq!(p.display_name, "Max");
        assert!(p.updated_at >= p.created_at);
    }

    // TC-PREF-003
    #[test]
    fn set_theme_changes_it() {
        let mut p = Preferences::new();
        p.set_theme(Theme::Dark);
        assert_eq!(p.theme, Theme::Dark);
        assert!(p.updated_at >= p.created_at);
    }

    // TC-PREF-004
    #[test]
    fn toggle_theme_flips() {
        let mut p = Preferences::new();
        p.toggle_theme();
        assert_eq!(p.theme, Theme::Dark);
        p.toggle_theme();
        assert_eq!(p.theme, Theme::Light);
    }

    // TC-PREF-005
    #[test]
    fn set_accent_color_sets_it() {
        let mut p = Preferences::new();
        p.set_accent_color(Some("#abc".to_string()));
        assert_eq!(p.accent_color.as_deref(), Some("#abc"));
    }

    // TC-PREF-006
    #[test]
    fn set_default_view_sets_it() {
        let mut p = Preferences::new();
        p.set_default_view(DefaultView::Tasks);
        assert_eq!(p.default_view, DefaultView::Tasks);
    }

    // TC-PREF-007
    #[test]
    fn set_sidebar_collapsed_sets_it() {
        let mut p = Preferences::new();
        p.set_sidebar_collapsed(true);
        assert!(p.sidebar_collapsed);
    }

    // TC-PREF-008
    #[test]
    fn toggle_sidebar_flips() {
        let mut p = Preferences::new();
        p.toggle_sidebar();
        assert!(p.sidebar_collapsed);
        p.toggle_sidebar();
        assert!(!p.sidebar_collapsed);
    }

    // TC-PREF-009
    #[test]
    fn add_feed_interest_adds() {
        let mut p = Preferences::new();
        p.add_feed_interest(FeedInterest::new("rust"));
        assert_eq!(p.feed_interests.len(), 1);
    }

    // TC-PREF-010
    #[test]
    fn remove_feed_interest_present_returns_it() {
        let mut p = Preferences::new();
        let interest = FeedInterest::new("rust");
        let id = interest.id.clone();
        p.add_feed_interest(interest);
        assert!(p.remove_feed_interest(&id).is_some());
    }

    // TC-PREF-011
    #[test]
    fn find_feed_interest_and_mut() {
        let mut p = Preferences::new();
        let interest = FeedInterest::new("rust");
        let id = interest.id.clone();
        p.add_feed_interest(interest);
        assert!(p.find_feed_interest(&id).is_some());
        p.find_feed_interest_mut(&id).unwrap().set_weight(Some(1.0));
        assert_eq!(
            p.find_feed_interest(&id).unwrap().explicit_weight,
            Some(1.0)
        );
    }

    // TC-PREF-012
    #[test]
    fn ranked_interests_sorted_by_descending_score() {
        let mut p = Preferences::new();
        let mut low = FeedInterest::new("low");
        low.set_weight(Some(1.0));
        let mut high = FeedInterest::new("high");
        high.set_weight(Some(5.0));
        let mut mid = FeedInterest::new("mid");
        mid.set_weight(Some(3.0));
        p.add_feed_interest(low);
        p.add_feed_interest(high);
        p.add_feed_interest(mid);

        let ranked = p.ranked_interests();
        assert_eq!(ranked[0].topic, "high");
        assert_eq!(ranked[1].topic, "mid");
        assert_eq!(ranked[2].topic, "low");
    }
}
