//! C2: TUN设备抽象层
//!
//! 跨平台TUN设备封装 (macOS/Linux/Windows)

use std::io::{Read, Write};

/// TUN设备配置
#[derive(Debug, Clone)]
pub struct TunConfig {
    /// 设备名称 (None=自动分配)
    pub name: Option<String>,
    /// IP地址
    pub address: std::net::Ipv4Addr,
    /// 子网掩码
    pub netmask: std::net::Ipv4Addr,
    /// MTU
    pub mtu: u16,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            name: None,
            address: std::net::Ipv4Addr::new(10, 0, 0, 1),
            netmask: std::net::Ipv4Addr::new(255, 255, 255, 0),
            mtu: 1420,  // WireGuard标准MTU
        }
    }
}

/// TUN设备错误
#[derive(Debug, thiserror::Error)]
pub enum TunError {
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("platform not supported")]
    UnsupportedPlatform,
}

/// TUN设备 trait
pub trait TunDevice: Send + Sync {
    /// 读取数据包
    fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, TunError>;

    /// 写入数据包
    fn write_packet(&mut self, data: &[u8]) -> Result<usize, TunError>;

    /// 获取设备名称
    fn name(&self) -> &str;

    /// 关闭设备
    fn close(&mut self) -> Result<(), TunError>;
}

/// 模拟TUN设备 (用于测试)
pub struct MockTunDevice {
    name: String,
    rx_queue: std::collections::VecDeque<Vec<u8>>,
    tx_queue: std::collections::VecDeque<Vec<u8>>,
}

impl MockTunDevice {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rx_queue: std::collections::VecDeque::new(),
            tx_queue: std::collections::VecDeque::new(),
        }
    }

    /// 注入待读取的数据包
    pub fn inject_rx(&mut self, data: Vec<u8>) {
        self.rx_queue.push_back(data);
    }

    /// 获取已写入的数据包
    pub fn take_tx(&mut self) -> Option<Vec<u8>> {
        self.tx_queue.pop_front()
    }
}

impl TunDevice for MockTunDevice {
    fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, TunError> {
        match self.rx_queue.pop_front() {
            Some(data) => {
                let len = std::cmp::min(data.len(), buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            }
            None => Ok(0), // 无数据
        }
    }

    fn write_packet(&mut self, data: &[u8]) -> Result<usize, TunError> {
        self.tx_queue.push_back(data.to_vec());
        Ok(data.len())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn close(&mut self) -> Result<(), TunError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_tun_read_write() {
        let mut device = MockTunDevice::new("test0");

        // 注入数据
        device.inject_rx(vec![0x45, 0x00, 0x00, 0x1c]); // IPv4 header start

        // 读取
        let mut buf = [0u8; 100];
        let n = device.read_packet(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf[..4], &[0x45, 0x00, 0x00, 0x1c]);

        // 写入
        let written = device.write_packet(&[0x45, 0x00]).unwrap();
        assert_eq!(written, 2);

        // 获取写入的数据
        let tx = device.take_tx().unwrap();
        assert_eq!(tx, vec![0x45, 0x00]);
    }
}
