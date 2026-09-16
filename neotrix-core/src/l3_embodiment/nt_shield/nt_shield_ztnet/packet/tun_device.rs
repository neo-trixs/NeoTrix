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
    fn mock_tun_read_write() {
        let mut device = _MockTunDevice::new("test0");

        // 注入数据
        device._inject_rx(vec![0x45, 0x00, 0x00, 0x1c]); // IPv4 header start

        // 读取
        let mut buf = [0u8; 100];
        let n = device.read_packet(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf[..4], &[0x45, 0x00, 0x00, 0x1c]);

        // 写入
        let written = device.write_packet(&[0x45, 0x00]).unwrap();
        assert_eq!(written, 2);

        // 获取写入的数据
        let tx = device._take_tx().unwrap();
        assert_eq!(tx, vec![0x45, 0x00]);
    }
}
