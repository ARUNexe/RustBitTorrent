use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

pub struct Benchmark {
    pub started: Instant,
    active_peers: AtomicUsize,
    peak_peers: AtomicUsize,
    received_bytes: AtomicU64,
    verified_bytes: AtomicU64,
}

impl Benchmark {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            active_peers: AtomicUsize::new(0),
            peak_peers: AtomicUsize::new(0),
            received_bytes: AtomicU64::new(0),
            verified_bytes: AtomicU64::new(0),
        }
    }

    pub fn peer_connected(&self) {
        let active = self.active_peers.fetch_add(1, Ordering::Relaxed) + 1;
        self.peak_peers.fetch_max(active, Ordering::Relaxed);
    }

    pub fn peer_disconnected(&self) {
        self.active_peers.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn add_verified_bytes(&self, bytes: usize) {
        self.verified_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn add_received_bytes(&self, bytes: usize) {
        self.received_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> (usize, usize, u64, u64) {
        (
            self.active_peers.load(Ordering::Relaxed),
            self.peak_peers.load(Ordering::Relaxed),
            self.received_bytes.load(Ordering::Relaxed),
            self.verified_bytes.load(Ordering::Relaxed),
        )
    }
}
