//! Service switching for the single-view Free TV panel. No DRM or network bypass.
use crate::{Source, SourcePolicy, SourceValidationError};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeTvService {
    Pluto,
    Plex,
    Roku,
    Tubi,
    Samsung,
}
impl FreeTvService {
    pub const ALL: [Self; 5] = [
        Self::Pluto,
        Self::Plex,
        Self::Roku,
        Self::Tubi,
        Self::Samsung,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pluto => "Pluto TV",
            Self::Plex => "Plex Live TV",
            Self::Roku => "Roku Channel",
            Self::Tubi => "Tubi",
            Self::Samsung => "Samsung TV+",
        }
    }
    pub const fn official_site(self) -> &'static str {
        match self {
            Self::Pluto => "https://www.pluto.tv/",
            Self::Plex => "https://www.plex.tv/watch-free-tv/",
            Self::Roku => "https://therokuchannel.roku.com/",
            Self::Tubi => "https://tubitv.com/",
            Self::Samsung => "https://www.samsungtvplus.com/",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    Unconfigured,
    Ready,
    Playing,
    Paused,
    Stopped,
    Failed,
}
#[derive(Debug)]
pub struct FreeTvPlayer {
    service: FreeTvService,
    source: Option<Source>,
    status: PlayerStatus,
    generation: u64,
}
impl Default for FreeTvPlayer {
    fn default() -> Self {
        Self {
            service: FreeTvService::Pluto,
            source: None,
            status: PlayerStatus::Unconfigured,
            generation: 0,
        }
    }
}
impl FreeTvPlayer {
    pub fn service(&self) -> FreeTvService {
        self.service
    }
    pub fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }
    pub fn status(&self) -> PlayerStatus {
        self.status
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    /// The caller MUST close the old decoder before opening a new one.
    /// Incremented generation invalidates late frames/events from prior sessions.
    pub fn select(&mut self, service: FreeTvService) {
        if self.service != service {
            self.generation = self.generation.wrapping_add(1);
            self.service = service;
            self.source = None;
            self.status = PlayerStatus::Unconfigured;
        }
    }
    /// Requires an explicitly approved media source; never derives a stream URL
    /// from a commercial service's homepage or hidden player APIs.
    pub fn configure(
        &mut self,
        source: Source,
        policy: &SourcePolicy,
    ) -> Result<(), SourceValidationError> {
        source.validate_definition(policy)?;
        self.generation = self.generation.wrapping_add(1);
        self.source = Some(source);
        self.status = PlayerStatus::Ready;
        Ok(())
    }
    pub fn update_status(&mut self, generation: u64, status: PlayerStatus) -> bool {
        if generation != self.generation || self.source.is_none() {
            return false;
        }
        self.status = status;
        true
    }
    pub fn stop(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.status = if self.source.is_some() {
            PlayerStatus::Stopped
        } else {
            PlayerStatus::Unconfigured
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pluto_is_default_but_not_playing() {
        let p = FreeTvPlayer::default();
        assert_eq!(p.service(), FreeTvService::Pluto);
        assert_eq!(p.status(), PlayerStatus::Unconfigured);
    }
    #[test]
    fn switching_invalidates_old_sessions() {
        let mut p = FreeTvPlayer::default();
        let generation = p.generation();
        p.select(FreeTvService::Plex);
        assert_ne!(generation, p.generation());
        assert!(!p.update_status(generation, PlayerStatus::Playing));
        assert_eq!(p.status(), PlayerStatus::Unconfigured);
    }
    #[test]
    fn unchanged_selection_does_not_restart() {
        let mut p = FreeTvPlayer::default();
        p.select(FreeTvService::Pluto);
        assert_eq!(p.generation(), 0);
    }
}
