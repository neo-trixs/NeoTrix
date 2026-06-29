use std::net::IpAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::metrics::ClientRateLimiter;

pub(crate) struct ConnectionGuard(pub(crate) Arc<AtomicU32>);

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) struct PerClientGuard {
    pub(crate) limiter: Arc<ClientRateLimiter>,
    pub(crate) addr: IpAddr,
}

impl Drop for PerClientGuard {
    fn drop(&mut self) {
        self.limiter.release(&self.addr);
    }
}
