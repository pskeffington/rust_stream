use crate::source::{Source, SourcePolicy, SourceValidationError};
use thiserror::Error;

pub const QUAD_SLOTS: usize = 4;
pub const MAX_FRAME_BYTES: usize = 1920 * 1080 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotStatus {
    Empty,
    Configured,
    Connecting,
    Live,
    Stalled,
    Reconnecting,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub index: usize,
    pub source: Option<Source>,
    pub status: SlotStatus,
}

#[derive(Debug, Clone)]
pub struct Quad {
    slots: [Slot; QUAD_SLOTS],
    focused: Option<usize>,
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("slot index must be 0..3")]
    InvalidSlot,
    #[error("invalid configured source: {0}")]
    InvalidSource(#[from] SourceValidationError),
    #[error("frame is too large, invalid or not RGBA")]
    InvalidFrame,
    #[error("media backend error")]
    MediaBackend,
}

#[derive(Debug, Clone)]
pub struct FrameRgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub pts_ms: u64,
}

impl FrameRgba {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>, pts_ms: u64) -> Result<Self, StreamError> {
        let expected = (width as usize).checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4)).ok_or(StreamError::InvalidFrame)?;
        if width == 0 || height == 0 || expected > MAX_FRAME_BYTES || pixels.len() != expected {
            return Err(StreamError::InvalidFrame);
        }
        Ok(Self { width, height, pixels, pts_ms })
    }
}

/// Implemented by a network/decoder adapter. Do not render or block the UI thread.
pub trait MediaSession: Send {
    fn poll_latest_frame(&mut self) -> Result<Option<FrameRgba>, StreamError>;
    fn close(&mut self);
}
pub trait MediaBackend: Send + Sync {
    fn open(&self, source: &Source) -> Result<Box<dyn MediaSession>, StreamError>;
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|index| Slot {
                index, source: None, status: SlotStatus::Empty,
            }),
            focused: None,
        }
    }
}

impl Quad {
    pub fn slots(&self) -> &[Slot; QUAD_SLOTS] { &self.slots }
    pub fn focused(&self) -> Option<usize> { self.focused }

    pub fn assign(&mut self, index: usize, source: Source, policy: &SourcePolicy)
        -> Result<(), StreamError>
    {
        if index >= QUAD_SLOTS { return Err(StreamError::InvalidSlot); }
        source.validate_definition(policy)?;
        self.slots[index].source = Some(source);
        self.slots[index].status = SlotStatus::Configured;
        Ok(())
    }

    pub fn clear(&mut self, index: usize) -> Result<(), StreamError> {
        if index >= QUAD_SLOTS { return Err(StreamError::InvalidSlot); }
        self.slots[index].source = None;
        self.slots[index].status = SlotStatus::Empty;
        if self.focused == Some(index) { self.focused = None; }
        Ok(())
    }

    pub fn set_status(&mut self, index: usize, status: SlotStatus) -> Result<(), StreamError> {
        if index >= QUAD_SLOTS { return Err(StreamError::InvalidSlot); }
        if self.slots[index].source.is_none() && status != SlotStatus::Empty {
            return Err(StreamError::InvalidSlot);
        }
        self.slots[index].status = status;
        Ok(())
    }

    pub fn set_focus(&mut self, index: Option<usize>) -> Result<(), StreamError> {
        if index.is_some_and(|n| n >= QUAD_SLOTS) { return Err(StreamError::InvalidSlot); }
        self.focused = index;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_slots_only() {
        let mut q = Quad::default();
        assert_eq!(q.slots().len(), 4);
        assert!(q.set_focus(Some(4)).is_err());
        q.set_focus(Some(2)).unwrap();
        assert_eq!(q.focused(), Some(2));
    }
    #[test]
    fn frame_checks_length_and_size() {
        assert!(FrameRgba::new(2, 2, vec![0; 16], 0).is_ok());
        assert!(FrameRgba::new(2, 2, vec![0; 15], 0).is_err());
        assert!(FrameRgba::new(10000, 10000, vec![], 0).is_err());
    }
}
