use std::time::Duration;

/// Saturating deterministic exponential backoff. Add jitter in the production adapter.
#[derive(Debug, Clone, Default)]
pub struct RetryBackoff {
    failures: u32,
}

impl RetryBackoff {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn failures(&self) -> u32 {
        self.failures
    }
    pub fn reset(&mut self) {
        self.failures = 0;
    }
    pub fn next_delay(&mut self) -> Duration {
        self.failures = self.failures.saturating_add(1);
        let shift = (self.failures - 1).min(6);
        Duration::from_millis((500_u64 << shift).min(30_000))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_and_resettable() {
        let mut b = RetryBackoff::new();
        assert_eq!(b.next_delay(), Duration::from_millis(500));
        assert_eq!(b.next_delay(), Duration::from_secs(1));
        for _ in 0..50 {
            b.next_delay();
        }
        assert_eq!(b.next_delay(), Duration::from_secs(30));
        b.reset();
        assert_eq!(b.next_delay(), Duration::from_millis(500));
    }
}
