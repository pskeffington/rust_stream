//! One reusable decoder worker per player; UI operations never call the backend.
use crate::{FrameRgba, MediaBackend, MediaSession, RetryBackoff, Source};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Offline,
    Loading,
    Live,
    Stalled,
    Reconnecting,
    Failed,
}
#[derive(Debug, Clone)]
pub struct PlaybackConfig {
    pub stall_after: Duration,
    pub timeout: Duration,
    /// Number of reconnects after the first failed attempt. Reset by operator restart.
    pub max_retries: u32,
}
impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            stall_after: Duration::from_secs(5),
            timeout: Duration::from_secs(15),
            max_retries: 3,
        }
    }
}
pub struct PlaybackSnapshot {
    pub status: PlaybackStatus,
    /// Taken from a single replaceable mailbox, never a frame history.
    pub frame: Option<FrameRgba>,
    pub frames_decoded: u64,
    pub retries: u32,
}
#[derive(Clone)]
struct Request {
    backend: Arc<dyn MediaBackend>,
    source: Source,
}
struct Shared {
    generation: u64,
    desired: Option<Request>,
    shutdown: bool,
    snapshot: PlaybackSnapshot,
}
/// Start, stop and replacement coalesce into one desired state. A stopped generation
/// cannot publish frames into a newer session. Drop joins the worker, including teardown.
/// Backends must honor their own connection/teardown timeouts; Rust cannot kill a blocked thread.
pub struct Playback {
    shared: Arc<Mutex<Shared>>,
    worker: Option<JoinHandle<()>>,
}
impl Default for Playback {
    fn default() -> Self {
        Self::new(PlaybackConfig::default())
    }
}
impl Playback {
    pub fn new(config: PlaybackConfig) -> Self {
        let shared = Arc::new(Mutex::new(Shared {
            generation: 0,
            desired: None,
            shutdown: false,
            snapshot: PlaybackSnapshot {
                status: PlaybackStatus::Offline,
                frame: None,
                frames_decoded: 0,
                retries: 0,
            },
        }));
        let state = shared.clone();
        let worker = thread::Builder::new()
            .name("stream-decoder".into())
            .spawn(move || run(state, config))
            .expect("create decoder worker");
        Self {
            shared,
            worker: Some(worker),
        }
    }
    pub fn start(&mut self, backend: Arc<dyn MediaBackend>, source: Source) {
        self.replace(Some(Request { backend, source }));
    }
    pub fn stop(&mut self) {
        self.replace(None);
    }
    fn replace(&mut self, request: Option<Request>) {
        let mut state = self.shared.lock().unwrap();
        state.generation = state.generation.wrapping_add(1);
        state.snapshot = PlaybackSnapshot {
            status: if request.is_some() {
                PlaybackStatus::Loading
            } else {
                PlaybackStatus::Offline
            },
            frame: None,
            frames_decoded: 0,
            retries: 0,
        };
        state.desired = request;
        self.worker.as_ref().unwrap().thread().unpark();
    }
    /// Nonblocking with respect to decoder/network operations; only a short mailbox lock.
    pub fn poll(&mut self) -> PlaybackSnapshot {
        let mut state = self.shared.lock().unwrap();
        if self.worker.as_ref().is_some_and(|w| w.is_finished()) && !state.shutdown {
            state.snapshot.status = PlaybackStatus::Failed;
        }
        PlaybackSnapshot {
            status: state.snapshot.status,
            frame: state.snapshot.frame.take(),
            frames_decoded: state.snapshot.frames_decoded,
            retries: state.snapshot.retries,
        }
    }
}
impl Drop for Playback {
    fn drop(&mut self) {
        self.shared.lock().unwrap().shutdown = true;
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            let _ = worker.join();
        }
    }
}
fn close(session: &mut Option<Box<dyn MediaSession>>) {
    if let Some(mut old) = session.take() {
        old.close();
    }
}
fn run(shared: Arc<Mutex<Shared>>, config: PlaybackConfig) {
    let mut generation = 0;
    let mut request: Option<Request> = None;
    let mut session: Option<Box<dyn MediaSession>> = None;
    let mut retries = 0;
    let mut backoff = RetryBackoff::new();
    let mut next_attempt = Instant::now();
    let mut last_frame = Instant::now();
    let mut failed = false;
    loop {
        let (shutdown, changed) = {
            let state = shared.lock().unwrap();
            (
                state.shutdown,
                if state.generation != generation {
                    Some((state.generation, state.desired.clone()))
                } else {
                    None
                },
            )
        };
        if shutdown {
            break;
        }
        if let Some((next, desired)) = changed {
            close(&mut session);
            generation = next;
            request = desired;
            retries = 0;
            backoff.reset();
            failed = false;
            next_attempt = Instant::now();
        }
        let mut status = None;
        let mut frame = None;
        let mut error = false;
        if let Some(active) = &request {
            if !failed && session.is_none() && Instant::now() >= next_attempt {
                match active.backend.open(&active.source) {
                    Ok(opened) => {
                        session = Some(opened);
                        last_frame = Instant::now();
                        status = Some(PlaybackStatus::Loading);
                    }
                    Err(_) => error = true,
                }
            }
            if let Some(opened) = session.as_mut() {
                match opened.poll_latest_frame() {
                    Ok(Some(decoded)) => {
                        match FrameRgba::new(
                            decoded.width,
                            decoded.height,
                            decoded.pixels,
                            decoded.pts_ms,
                        ) {
                            Ok(decoded) => {
                                frame = Some(decoded);
                                last_frame = Instant::now();
                                status = Some(PlaybackStatus::Live);
                            }
                            Err(_) => error = true,
                        }
                    }
                    Ok(None) => {
                        let elapsed = last_frame.elapsed();
                        if elapsed >= config.timeout {
                            error = true;
                        } else if elapsed >= config.stall_after {
                            status = Some(PlaybackStatus::Stalled);
                        }
                    }
                    Err(_) => error = true,
                }
            }
            if error {
                close(&mut session);
                if retries >= config.max_retries {
                    failed = true;
                    status = Some(PlaybackStatus::Failed);
                } else {
                    retries += 1;
                    next_attempt = Instant::now() + backoff.next_delay();
                    status = Some(PlaybackStatus::Reconnecting);
                }
            }
        }
        {
            let mut state = shared.lock().unwrap();
            if state.generation == generation && !state.shutdown {
                if let Some(status) = status {
                    state.snapshot.status = status;
                }
                if let Some(frame) = frame {
                    state.snapshot.frame = Some(frame);
                    state.snapshot.frames_decoded = state.snapshot.frames_decoded.saturating_add(1);
                }
                state.snapshot.retries = retries;
            }
        }
        thread::park_timeout(Duration::from_millis(10));
    }
    close(&mut session);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceKind, SourceScope, StreamError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Backend {
        mode: u8,
        opened: Arc<AtomicUsize>,
        closed: Arc<AtomicUsize>,
    }
    struct Session {
        mode: u8,
        closed: Arc<AtomicUsize>,
    }
    impl MediaBackend for Backend {
        fn open(&self, _: &Source) -> Result<Box<dyn MediaSession>, StreamError> {
            self.opened.fetch_add(1, Ordering::SeqCst);
            if self.mode == 2 {
                return Err(StreamError::MediaBackend);
            }
            Ok(Box::new(Session {
                mode: self.mode,
                closed: self.closed.clone(),
            }))
        }
    }
    impl MediaSession for Session {
        fn poll_latest_frame(&mut self) -> Result<Option<FrameRgba>, StreamError> {
            if self.mode == 1 {
                Ok(None)
            } else if self.mode == 255 {
                Ok(Some(FrameRgba {
                    width: 2,
                    height: 2,
                    pixels: vec![0; 3],
                    pts_ms: 0,
                }))
            } else {
                FrameRgba::new(1, 1, vec![self.mode; 4], 0).map(Some)
            }
        }
        fn close(&mut self) {
            self.closed.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn source() -> Source {
        Source {
            id: "test".into(),
            label: "Test".into(),
            endpoint: String::new(),
            kind: SourceKind::Hls,
            scope: SourceScope::Public,
            enabled: true,
        }
    }
    fn backend(mode: u8) -> Arc<Backend> {
        Arc::new(Backend {
            mode,
            opened: Arc::default(),
            closed: Arc::default(),
        })
    }
    fn wait(
        player: &mut Playback,
        predicate: impl Fn(&PlaybackSnapshot) -> bool,
    ) -> PlaybackSnapshot {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let snapshot = player.poll();
            if predicate(&snapshot) {
                return snapshot;
            }
            assert!(Instant::now() < deadline, "status: {:?}", snapshot.status);
            thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn four_players_replace_latest_frames_and_isolate_failure() {
        let mut players: [Playback; 4] = std::array::from_fn(|_| Playback::default());
        let backends: [_; 4] =
            std::array::from_fn(|i| backend(if i == 3 { 2 } else { (i + 3) as u8 }));
        for (p, b) in players.iter_mut().zip(&backends) {
            p.start(b.clone(), source());
        }
        for p in &mut players[..3] {
            wait(p, |s| s.status == PlaybackStatus::Live);
        }
        wait(&mut players[3], |s| s.status == PlaybackStatus::Failed);
        assert_eq!(backends[3].opened.load(Ordering::SeqCst), 4);
        thread::sleep(Duration::from_millis(60)); // A slow renderer never blocks the producer.
        for p in &mut players[..3] {
            assert!(p.poll().frames_decoded > 3);
        }
        drop(players);
        for b in &backends[..3] {
            assert_eq!(
                b.opened.load(Ordering::SeqCst),
                b.closed.load(Ordering::SeqCst)
            );
        }
    }
    #[test]
    fn stop_and_switch_do_not_publish_old_generations() {
        let a = backend(3);
        let b = backend(4);
        let mut p = Playback::default();
        for _ in 0..5 {
            p.start(a.clone(), source());
            wait(&mut p, |s| s.frame.is_some());
            p.stop();
            assert_eq!(p.poll().status, PlaybackStatus::Offline);
            assert!(p.poll().frame.is_none());
            p.start(b.clone(), source());
            let snapshot = wait(&mut p, |s| s.frame.is_some());
            assert_eq!(snapshot.frame.unwrap().pixels, vec![4; 4]);
        }
        drop(p);
        assert_eq!(
            a.opened.load(Ordering::SeqCst),
            a.closed.load(Ordering::SeqCst)
        );
        assert_eq!(
            b.opened.load(Ordering::SeqCst),
            b.closed.load(Ordering::SeqCst)
        );
    }
    #[test]
    fn stalled_stream_times_out_and_can_be_restarted() {
        let mut p = Playback::new(PlaybackConfig {
            stall_after: Duration::from_millis(30),
            timeout: Duration::from_millis(100),
            max_retries: 0,
        });
        p.start(backend(1), source());
        wait(&mut p, |s| s.status == PlaybackStatus::Stalled);
        wait(&mut p, |s| s.status == PlaybackStatus::Failed);
        p.start(backend(3), source());
        wait(&mut p, |s| s.status == PlaybackStatus::Live);
    }
    #[test]
    fn malformed_backend_frames_never_reach_renderer() {
        let mut p = Playback::new(PlaybackConfig {
            max_retries: 0,
            ..Default::default()
        });
        p.start(backend(255), source());
        let snapshot = wait(&mut p, |s| s.status == PlaybackStatus::Failed);
        assert!(snapshot.frame.is_none());
        assert_eq!(snapshot.frames_decoded, 0);
    }
    #[test]
    fn stop_and_switch_remain_responsive_during_a_blocked_open() {
        use std::sync::mpsc;
        struct Delayed {
            entered: mpsc::Sender<()>,
            release: Mutex<mpsc::Receiver<()>>,
            closed: Arc<AtomicUsize>,
        }
        impl MediaBackend for Delayed {
            fn open(&self, _: &Source) -> Result<Box<dyn MediaSession>, StreamError> {
                self.entered.send(()).unwrap();
                self.release
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
                Ok(Box::new(Session {
                    mode: 3,
                    closed: self.closed.clone(),
                }))
            }
        }
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let closed = Arc::new(AtomicUsize::new(0));
        let mut p = Playback::default();
        p.start(
            Arc::new(Delayed {
                entered: entered_tx,
                release: Mutex::new(release_rx),
                closed: closed.clone(),
            }),
            source(),
        );
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let before = Instant::now();
        p.stop();
        assert_eq!(p.poll().status, PlaybackStatus::Offline);
        p.start(backend(4), source());
        assert!(before.elapsed() < Duration::from_millis(100));
        release_tx.send(()).unwrap();
        assert_eq!(
            wait(&mut p, |s| s.frame.is_some()).frame.unwrap().pixels,
            [4; 4]
        );
        drop(p);
        assert_eq!(closed.load(Ordering::SeqCst), 1);
    }
}
