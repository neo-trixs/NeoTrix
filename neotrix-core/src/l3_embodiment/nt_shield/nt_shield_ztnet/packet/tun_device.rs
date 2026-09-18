//! C2: TUN设备抽象层
//!
//! 跨平台TUN设备封装 (macOS/Linux/Windows)

/// TUN设备错误
#[derive(Debug, thiserror::Error)]
pub enum _TunError {
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("platform not supported")]
    UnsupportedPlatform,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "_MockTunDevice not yet implemented"]
    fn mock_tun_read_write() {
        // TODO: implement _MockTunDevice
    }
}
