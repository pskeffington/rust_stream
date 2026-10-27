//! Media collections and handheld-friendly presets shared with Rust-Boy.
//! Pure policy-independent view state. No media connections are opened.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedCollection {
    Security,
    News,
    Traffic,
    Custom,
}

impl FeedCollection {
    pub const ALL: [Self; 4] = [Self::Security, Self::News, Self::Traffic, Self::Custom];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Security => "SECURITY",
            Self::News => "NEWS",
            Self::Traffic => "TRAFFIC",
            Self::Custom => "CUSTOM",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Quad,
    Focused(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaView {
    collection: FeedCollection,
    mode: ViewMode,
}

impl Default for MediaView {
    fn default() -> Self {
        Self {
            collection: FeedCollection::Security,
            mode: ViewMode::Quad,
        }
    }
}

impl MediaView {
    pub const fn collection(&self) -> FeedCollection {
        self.collection
    }
    pub const fn mode(&self) -> ViewMode {
        self.mode
    }

    /// Switching collections always returns to the four-tile overview.
    pub fn select_collection(&mut self, collection: FeedCollection) {
        self.collection = collection;
        self.mode = ViewMode::Quad;
    }

    pub fn focus(&mut self, slot: usize) -> Result<(), &'static str> {
        if slot >= 4 {
            return Err("feed slot must be 0..3");
        }
        self.mode = ViewMode::Focused(slot);
        Ok(())
    }

    pub fn back_to_quad(&mut self) {
        self.mode = ViewMode::Quad;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collection_switch_resets_focus() {
        let mut media = MediaView::default();
        media.focus(3).unwrap();
        media.select_collection(FeedCollection::Traffic);
        assert_eq!(media.collection(), FeedCollection::Traffic);
        assert_eq!(media.mode(), ViewMode::Quad);
    }
    #[test]
    fn invalid_focus_does_not_change_state() {
        let mut media = MediaView::default();
        assert!(media.focus(4).is_err());
        assert_eq!(media.mode(), ViewMode::Quad);
    }
}
