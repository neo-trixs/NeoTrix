use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

const JITTER_MAX_MS: u64 = 500;

pub(crate) fn rand_u64_splitmix64() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut z = COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

pub(crate) fn rand_range(max: u64) -> u64 {
    if max == 0 {
        return 0;
    }
    rand_u64_splitmix64() % max
}

pub(crate) fn jitter_sleep() {
    let ms = rand_range(JITTER_MAX_MS);
    if ms > 20 {
        thread::sleep(Duration::from_millis(ms));
    }
}

pub(crate) fn socks5_greeting_padded() -> Vec<u8> {
    let extra = rand_range(4) as u8;
    let n = 1 + extra;
    let mut msg = Vec::with_capacity(2 + n as usize);
    msg.push(5);
    msg.push(n);
    msg.resize(2 + n as usize, 0);
    msg
}

pub(crate) fn socks5_greeting_standard() -> Vec<u8> {
    vec![5, 1, 0]
}

pub(crate) fn padding_bytes(min: usize, max: usize) -> Vec<u8> {
    let len = if max <= min { min } else { min + rand_range((max - min) as u64) as usize };
    let mut buf = vec![0u8; len];
    // Fill with splitmix64 random bytes
    for chunk in buf.chunks_mut(8) {
        let r = rand_u64_splitmix64();
        let n = chunk.len().min(8);
        chunk.copy_from_slice(&r.to_le_bytes()[..n]);
    }
    buf
}
