//! C3: TURN Client (SANS-IO)
//!
//! RFC 8656 TURN协议实现，零IO纯状态机。

use std::net::SocketAddr;
use std::time::Instant;
use bytes::{Bytes, BufMut};

/// TURN方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _TurnMethod {
    Allocate,
    Refresh,
    Send,
    Data,
    CreatePermission,
    ChannelBind,
}

/// TURN属性
#[derive(Debug, Clone)]
pub enum _TurnAttribute {
    /// Channel Number
    ChannelNumber(u16),
    /// Lifetime
    Lifetime(u32),
    /// XOR-RELAYED-ADDRESS
    XorRelayedAddress(SocketAddr),
    /// XOR-MAPPED-ADDRESS
    XorMappedAddress(SocketAddr),
    /// Data
    Data(Vec<u8>),
    /// Realm
    Realm(String),
    /// Nonce
    Nonce(String),
    /// ERROR-CODE
    ErrorCode { code: u16, reason: String },
    /// Requested-Transport
    RequestedTransport(u8),
    /// DONT-FRAGMENT
    DontFragment,
}

/// TURN消息
#[derive(Debug, Clone)]
pub struct _TurnMessage {
    pub method: _TurnMethod,
    pub transaction_id: [u8; 12],
    pub attributes: Vec<_TurnAttribute>,
}

impl _TurnMessage {
    /// 编码TURN消息
    pub fn encode(&self) -> Bytes {
        let mut buf = Vec::with_capacity(20);

        // Header
        let method_num = match self.method {
            _TurnMethod::Allocate => 0x0003,
            _TurnMethod::Refresh => 0x0004,
            _TurnMethod::Send => 0x0006,
            _TurnMethod::Data => 0x0007,
            _TurnMethod::CreatePermission => 0x0008,
            _TurnMethod::ChannelBind => 0x0009,
        };

        buf.put_u16(0x0000 | method_num);
        buf.put_u16(0);  // Length placeholder
        buf.put_u32(0x2112A442);  // Magic Cookie
        buf.put_slice(&self.transaction_id);

        // Attributes
        let mut attr_len = 0u16;
        for attr in &self.attributes {
            let (attr_type, attr_data) = match attr {
                _TurnAttribute::ChannelNumber(ch) => {
                    (0x000C, ch.to_be_bytes().to_vec())
                }
                _TurnAttribute::Lifetime(lifetime) => {
                    (0x000D, lifetime.to_be_bytes().to_vec())
                }
                _TurnAttribute::XorRelayedAddress(addr) => {
                    (0x0016, encode_xor_address(addr, &self.transaction_id))
                }
                _TurnAttribute::XorMappedAddress(addr) => {
                    (0x0020, encode_xor_address(addr, &self.transaction_id))
                }
                _TurnAttribute::Data(data) => {
                    (0x0013, data.clone())
                }
                _TurnAttribute::RequestedTransport(proto) => {
                    (0x0019, vec![*proto, 0, 0, 0])
                }
                _TurnAttribute::DontFragment => {
                    (0x001A, vec![])
                }
                _ => continue,
            };

            buf.put_u16(attr_type);
            buf.put_u16(attr_data.len() as u16);
            buf.put_slice(&attr_data);
            attr_len += 4 + attr_data.len() as u16;

            let padding = (4 - (attr_data.len() % 4)) % 4;
            buf.put_slice(&vec![0u8; padding]);
            attr_len += padding as u16;
        }

        buf[2..4].copy_from_slice(&attr_len.to_be_bytes());
        Bytes::from(buf)
    }

    /// 解码TURN消息
    pub fn decode(data: &[u8]) -> Result<Self, super::stun::StunError> {
        if data.len() < 20 {
            return Err(super::stun::StunError::MessageTooShort);
        }

        let magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        if magic_cookie != 0x2112A442 {
            return Err(super::stun::StunError::InvalidMagicCookie);
        }

        let msg_type = u16::from_be_bytes([data[0], data[1]]);
        let method = match msg_type & 0x3EEF {
            0x0003 => _TurnMethod::Allocate,
            0x0004 => _TurnMethod::Refresh,
            0x0006 => _TurnMethod::Send,
            0x0007 => _TurnMethod::Data,
            0x0008 => _TurnMethod::CreatePermission,
            0x0009 => _TurnMethod::ChannelBind,
            other => return Err(super::stun::StunError::UnknownMethod(other)),
        };

        let mut transaction_id = [0u8; 12];
        transaction_id.copy_from_slice(&data[8..20]);

        Ok(_TurnMessage { method, transaction_id, attributes: vec![] })
    }
}

/// 编码XOR地址
pub(crate) fn encode_xor_address(addr: &SocketAddr, transaction_id: &[u8; 12]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8);
    buf.push(0);  // Reserved

    match addr {
        SocketAddr::V4(v4) => {
            buf.push(1);  // Family: IPv4
            buf.put_u16(addr.port() ^ 0x2112);
            let mut octets = v4.ip().octets();
            octets[0] ^= 0x21;
            octets[1] ^= 0x12;
            octets[2] ^= 0xA4;
            octets[3] ^= 0x42;
            buf.put_slice(&octets);
        }
        SocketAddr::V6(v6) => {
            buf.push(2);  // Family: IPv6
            buf.put_u16(addr.port() ^ 0x2112);
            let mut octets = v6.ip().octets();
            for i in 0..4 {
                octets[i] ^= 0x21;
                octets[i + 1] ^= 0x12;
                octets[i + 2] ^= 0xA4;
                octets[i + 3] ^= 0x42;
            }
            for i in 0..12 {
                octets[i + 4] ^= transaction_id[i];
            }
            buf.put_slice(&octets);
        }
    }

    buf
}

/// TURN分配状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _TurnAllocationState {
    /// 初始
    Initial,
    /// 分配请求中
    Allocating,
    /// 已分配
    Allocated,
    /// 刷新中
    Refreshing,
    /// 失败
    Failed { reason: String },
}

/// TURN客户端状态机
pub struct _TurnClient {
    /// 状态
    pub state: _TurnAllocationState,
    /// TURN服务器地址
    pub server: SocketAddr,
    /// 本地接口地址
    pub local_addr: SocketAddr,
    /// 分配的中继地址
    pub relay_addr: Option<SocketAddr>,
    /// 生命周期 (秒)
    pub lifetime: u32,
    /// 最后刷新时间
    pub last_refresh: Option<Instant>,
    /// 事务ID
    transaction_id: [u8; 12],
}

impl _TurnClient {
    /// 创建新TURN客户端
    pub fn new(server: SocketAddr, local_addr: SocketAddr) -> Self {
        let mut transaction_id = [0u8; 12];
        use rand::Rng;
        rand::thread_rng().fill(&mut transaction_id);

        Self {
            state: _TurnAllocationState::Initial,
            server,
            local_addr,
            relay_addr: None,
            lifetime: 600,  // 10分钟
            last_refresh: None,
            transaction_id,
        }
    }

    /// 生成分配请求
    pub fn _create_allocation_request(&mut self) -> Bytes {
        use rand::Rng;
        rand::thread_rng().fill(&mut self.transaction_id);

        let msg = _TurnMessage {
            method: _TurnMethod::Allocate,
            transaction_id: self.transaction_id,
            attributes: vec![
                _TurnAttribute::RequestedTransport(17),  // UDP
                _TurnAttribute::DontFragment,
            ],
        };

        self.state = _TurnAllocationState::Allocating;
        msg.encode()
    }

    /// 处理分配响应
    pub fn _handle_allocation_response(&mut self, data: &[u8]) -> bool {
        if let Ok(msg) = _TurnMessage::decode(data) {
            if msg.transaction_id == self.transaction_id {
                // 查找中继地址
                for attr in &msg.attributes {
                    if let _TurnAttribute::XorRelayedAddress(addr) = attr {
                        self.relay_addr = Some(*addr);
                        self.state = _TurnAllocationState::Allocated;
                        self.last_refresh = Some(Instant::now());
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 生成刷新请求
    pub fn _create_refresh_request(&mut self) -> Bytes {
        use rand::Rng;
        rand::thread_rng().fill(&mut self.transaction_id);

        let msg = _TurnMessage {
            method: _TurnMethod::Refresh,
            transaction_id: self.transaction_id,
            attributes: vec![
                _TurnAttribute::Lifetime(self.lifetime),
            ],
        };

        self.state = _TurnAllocationState::Refreshing;
        msg.encode()
    }

    /// 处理刷新响应
    pub fn _handle_refresh_response(&mut self, data: &[u8]) -> bool {
        if let Ok(msg) = _TurnMessage::decode(data) {
            if msg.transaction_id == self.transaction_id {
                self.state = _TurnAllocationState::Allocated;
                self.last_refresh = Some(Instant::now());
                return true;
            }
        }
        false
    }

    /// 生成创建权限请求
    pub fn _create_permission_request(&mut self, peer_addr: SocketAddr) -> Bytes {
        use rand::Rng;
        rand::thread_rng().fill(&mut self.transaction_id);

        let msg = _TurnMessage {
            method: _TurnMethod::CreatePermission,
            transaction_id: self.transaction_id,
            attributes: vec![
                _TurnAttribute::XorMappedAddress(peer_addr),
            ],
        };

        msg.encode()
    }

    /// 生成ChannelBind请求
    pub fn _create_channel_bind_request(&mut self, peer_addr: SocketAddr, channel: u16) -> Bytes {
        use rand::Rng;
        rand::thread_rng().fill(&mut self.transaction_id);

        let msg = _TurnMessage {
            method: _TurnMethod::ChannelBind,
            transaction_id: self.transaction_id,
            attributes: vec![
                _TurnAttribute::ChannelNumber(channel),
                _TurnAttribute::XorMappedAddress(peer_addr),
            ],
        };

        msg.encode()
    }

    /// 检查是否需要刷新
    pub fn needs_refresh(&self) -> bool {
        if let Some(last) = self.last_refresh {
            let elapsed = last.elapsed().as_secs();
            elapsed > (self.lifetime as u64 * 2 / 3)  // 生命周期的2/3
        } else {
            false
        }
    }

    /// 重置
    pub fn reset(&mut self) {
        self.state = _TurnAllocationState::Initial;
        self.relay_addr = None;
        self.last_refresh = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn turn_client_creation() {
        let server = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)), 3478);
        let local = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 5000);

        let client = _TurnClient::new(server, local);
        assert_eq!(client.state, _TurnAllocationState::Initial);
        assert_eq!(client.server, server);
    }

    #[test]
    fn allocation_request_encoding() {
        let server = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)), 3478);
        let local = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 5000);

        let mut client = _TurnClient::new(server, local);
        let req = client._create_allocation_request();

        assert!(!req.is_empty());
        assert_eq!(client.state, _TurnAllocationState::Allocating);
    }
}
