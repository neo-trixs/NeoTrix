use std::sync::atomic::{AtomicU64, Ordering};

pub struct Session {
    pub id: u64,
    pub auth_key: Option<[u8; 256]>,
    pub server_salt: u64,
    pub seq_no: u32,
    pub msg_id: AtomicU64,
    pub user_id: Option<i64>,
}

impl Session {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            auth_key: None,
            server_salt: rand::random(),
            seq_no: 0,
            msg_id: AtomicU64::new(0),
            user_id: None,
        }
    }

    pub fn next_msg_id(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let base = (now as i64) << 32;
        let prev = self.msg_id.fetch_add(1, Ordering::SeqCst);
        base | (prev as i64 & 0xFFFFFFFF)
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth_key.is_some()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_unique_ids() {
        let a = Session::new();
        let b = Session::new();
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn test_auth_state() {
        let s = Session::new();
        assert!(!s.is_authenticated());
        let mut s2 = Session::new();
        s2.auth_key = Some([0x42u8; 256]);
        assert!(s2.is_authenticated());
    }

    #[test]
    fn test_msg_id_monotonic() {
        let s = Session::new();
        let a = s.next_msg_id();
        let b = s.next_msg_id();
        assert!(b > a);
    }

    #[test]
    fn test_server_salt_random() {
        let a = Session::new();
        let b = Session::new();
        assert_ne!(a.server_salt, b.server_salt);
    }
}
