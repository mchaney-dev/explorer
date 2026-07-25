use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::feed::feed_interest::FeedInterest;
use crate::models::shared::{HexColor, Timestamp};
use crate::models::user::features::Features;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultView {
    Graph,
    Feed,
    Tasks,
    Notes,
    Calendar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
