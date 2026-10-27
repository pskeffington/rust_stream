//! Test-only loopback HTTP fixture. Production source policy is never relaxed.
use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct Fixture {
    url: String,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl Fixture {
    fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let running = stop.clone();
        let worker = thread::spawn(move || {
            while !running.load(Ordering::SeqCst) {
                let Ok((mut socket, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                };
                socket
                    .set_read_timeout(Some(Duration::from_millis(500)))
                    .unwrap();
                socket
                    .set_write_timeout(Some(Duration::from_millis(500)))
                    .unwrap();
                let mut request = Vec::new();
                let mut byte = [0];
                while request.len() < 8192 && !request.ends_with(b"\r\n\r\n") {
                    match socket.read(&mut byte) {
                        Ok(1) => request.push(byte[0]),
                        _ => break,
                    }
                }
                let request = String::from_utf8_lossy(&request);
                let path = request.split_whitespace().nth(1).unwrap_or("");
                let (body, mime): (&[u8], &str) = match path {
                    "/index.m3u8" => (include_bytes!("../tests/fixtures/hls/index.m3u8"), "application/vnd.apple.mpegurl"),
                    "/segment00.ts" => (include_bytes!("../tests/fixtures/hls/segment00.ts"), "video/mp2t"),
                    "/segment01.ts" => (include_bytes!("../tests/fixtures/hls/segment01.ts"), "video/mp2t"),
                    "/segment02.ts" => (include_bytes!("../tests/fixtures/hls/segment02.ts"), "video/mp2t"),
                    "/broken.m3u8" => (b"#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXTINF:1,\nmissing.ts\n#EXT-X-ENDLIST\n", "application/vnd.apple.mpegurl"),
                    _ => (&[], "text/plain"),
                };
                let status = if body.is_empty() {
                    "404 Not Found"
                } else {
                    "200 OK"
                };
                let _ = write!(
                    socket,
                    "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = socket.write_all(body);
            }
        });
        Self {
            url,
            stop,
            worker: Some(worker),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.worker.take().unwrap().join().unwrap();
    }
}
#[test]
fn hls_http_fixture_decodes_advancing_frames_and_reports_eos() {
    let fixture = Fixture::serve();
    let mut session = open_pipeline(None, &format!("{}/index.m3u8", fixture.url)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut timestamps = Vec::new();
    let mut ended = false;
    while Instant::now() < deadline {
        match session.poll_latest_frame() {
            Ok(Some(frame)) => {
                assert_eq!((frame.width, frame.height), (640, 360));
                assert!(frame.pixels.chunks_exact(4).any(|p| p[0] != p[1]));
                timestamps.push(frame.pts_ms);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(StreamError::EndOfStream) => {
                ended = true;
                break;
            }
            Err(error) => panic!("HLS decode failed: {error}"),
        }
    }
    assert!(ended, "HLS fixture did not reach EOS");
    assert!(
        timestamps.len() >= 3,
        "No advancing decoded HLS video: {timestamps:?}"
    );
    assert!(timestamps.windows(2).all(|t| t[1] > t[0]));
    session.close();
}
#[test]
fn missing_hls_segment_becomes_a_decoder_error() {
    let fixture = Fixture::serve();
    let mut session = open_pipeline(None, &format!("{}/broken.m3u8", fixture.url)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut failed = false;
    while Instant::now() < deadline {
        if session.poll_latest_frame().is_err() {
            failed = true;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(failed, "Missing HLS segment remained silently loading");
}
#[test]
fn four_native_workers_decode_stop_and_restart() {
    let mut players: [crate::Playback; 4] = std::array::from_fn(|_| crate::Playback::default());
    for _ in 0..2 {
        for (i, player) in players.iter_mut().enumerate() {
            let backend = GstBackend::test_pattern(i);
            let source = backend.source().clone();
            player.start(Arc::new(backend), source);
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut decoded = [false; 4];
        while !decoded.iter().all(|v| *v) && Instant::now() < deadline {
            for (i, player) in players.iter_mut().enumerate() {
                let snapshot = player.poll();
                if snapshot.frames_decoded >= 3 && snapshot.frame.is_some() {
                    decoded[i] = true;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(decoded.iter().all(|v| *v), "four native slots: {decoded:?}");
        for player in &mut players {
            player.stop();
            assert!(player.poll().frame.is_none());
        }
    }
    drop(players);
}
