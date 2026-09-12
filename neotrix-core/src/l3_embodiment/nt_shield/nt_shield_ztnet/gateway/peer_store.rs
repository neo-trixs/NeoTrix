//! C4: PeerStore — 对等节点管理
//!
//! 管理已发现的对等节点，支持快速查找、评分、持久化。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// 对等节点
#[derive(Debug, Clone)]
pub struct Peer {
    /// 节点ID (公钥哈希)
    pub id: [u8; 32],
    /// 公钥
    pub public_key: [u8; 32],
    /// 已知地址
    pub endpoints: Vec<SocketAddr>,
    /// 连接状态
    pub connection_state: _ConnectionState,
    /// 最后活跃时间
    pub last_seen: Instant,
    /// 连接质量评分 (0-100)
    pub quality_score: u8,
    /// 延迟 (毫秒)
    pub latency_ms: Option<u64>,
    /// 标签
    pub tags: Vec<String>,
    /// 节点元数据
    pub metadata: HashMap<String, String>,
}

/// 连接状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _ConnectionState {
    /// 未知
    Unknown,
    /// 正在连接
    Connecting,
    /// 已连接
    Connected,
    /// 空闲
    Idle,
    /// 正在断开
    Disconnecting,
    /// 已断开
    Disconnected,
}

/// 对等节点存储
pub struct PeerStore {
    /// 节点映射 (ID → Peer)
    peers: HashMap<[u8; 32], Peer>,
    /// 地址→ID索引
    addr_index: HashMap<SocketAddr, [u8; 32]>,
    /// 最大节点数
    max_peers: usize,
    /// 节点过期时间
    expiry: Duration,
}

impl PeerStore {
    /// 创建新的对等节点存储
    pub fn new(max_peers: usize, expiry: Duration) -> Self {
        Self {
            peers: HashMap::new(),
            addr_index: HashMap::new(),
            max_peers,
            expiry,
        }
    }

    /// 添加节点
    pub fn add_peer(&mut self, peer: Peer) -> bool {
        if self.peers.len() >= self.max_peers {
            // 淘汰最低评分的节点
            if let Some(worst_id) = self.find_worst_peer() {
                if self.peers[&worst_id].quality_score < peer.quality_score {
                    self._remove_peer(&worst_id);
                } else {
                    return false;
                }
            }
        }

        // 索引地址
        for addr in &peer.endpoints {
            self.addr_index.insert(*addr, peer.id);
        }

        self.peers.insert(peer.id, peer);
        true
    }

    /// 移除节点
    pub fn _remove_peer(&mut self, id: &[u8; 32]) -> Option<Peer> {
        if let Some(peer) = self.peers.remove(id) {
            for addr in &peer.endpoints {
                self.addr_index.remove(addr);
            }
            Some(peer)
        } else {
            None
        }
    }

    /// 查找节点
    pub fn get_peer(&self, id: &[u8; 32]) -> Option<&Peer> {
        self.peers.get(id)
    }

    /// 通过地址查找节点
    pub fn _get_peer_by_addr(&self, addr: &SocketAddr) -> Option<&Peer> {
        self.addr_index.get(addr).and_then(|id| self.peers.get(id))
    }

    /// 查找最低评分的节点
    fn find_worst_peer(&self) -> Option<[u8; 32]> {
        self.peers.values()
            .min_by_key(|p| p.quality_score)
            .map(|p| p.id)
    }

    /// 更新节点质量评分
    pub fn update_quality(&mut self, id: &[u8; 32], score: u8) {
        if let Some(peer) = self.peers.get_mut(id) {
            peer.quality_score = score;
            peer.last_seen = Instant::now();
        }
    }

    /// 更新节点延迟
    pub fn update_latency(&mut self, id: &[u8; 32], latency_ms: u64) {
        if let Some(peer) = self.peers.get_mut(id) {
            peer.latency_ms = Some(latency_ms);
            peer.last_seen = Instant::now();
        }
    }

    /// 更新节点状态
    pub fn update_state(&mut self, id: &[u8; 32], state: _ConnectionState) {
        if let Some(peer) = self.peers.get_mut(id) {
            peer.connection_state = state;
            peer.last_seen = Instant::now();
        }
    }

    /// 获取所有活跃节点 (未过期)
    pub fn _active_peers(&self) -> Vec<&Peer> {
        let now = Instant::now();
        self.peers.values()
            .filter(|p| now.duration_since(p.last_seen) < self.expiry)
            .collect()
    }

    /// 按质量评分排序获取节点
    pub fn _peers_by_quality(&self) -> Vec<&Peer> {
        let mut peers: Vec<&Peer> = self.peers.values().collect();
        peers.sort_by(|a, b| b.quality_score.cmp(&a.quality_score));
        peers
    }

    /// 按延迟排序获取节点
    pub fn _peers_by_latency(&self) -> Vec<&Peer> {
        let mut peers: Vec<&Peer> = self.peers.values().collect();
        peers.sort_by(|a, b| {
            a.latency_ms.unwrap_or(u64::MAX).cmp(&b.latency_ms.unwrap_or(u64::MAX))
        });
        peers
    }

    /// 获取节点数量
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// 清理过期节点
    pub fn cleanup_expired(&mut self) -> Vec<[u8; 32]> {
        let now = Instant::now();
        let expired: Vec<[u8; 32]> = self.peers.iter()
            .filter(|(_, p)| now.duration_since(p.last_seen) >= self.expiry)
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            self._remove_peer(id);
        }

        expired
    }

    /// 获取统计信息
    pub fn stats(&self) -> _PeerStoreStats {
        let total = self.peers.len();
        let connected = self.peers.values()
            .filter(|p| p.connection_state == _ConnectionState::Connected)
            .count();
        let avg_quality = if total > 0 {
            self.peers.values().map(|p| p.quality_score as u32).sum::<u32>() / total as u32
        } else {
            0
        };
        let avg_latency = {
            let latencies: Vec<u64> = self.peers.values()
                .filter_map(|p| p.latency_ms)
                .collect();
            if latencies.is_empty() {
                None
            } else {
                Some(latencies.iter().sum::<u64>() / latencies.len() as u64)
            }
        };

        _PeerStoreStats {
            total,
            connected,
            avg_quality,
            avg_latency,
        }
    }
}

/// 对等节点存储统计
#[derive(Debug, Clone)]
pub struct _PeerStoreStats {
    pub total: usize,
    pub connected: usize,
    pub avg_quality: u32,
    pub avg_latency: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn create_test_peer(id: u8, addr: SocketAddr) -> Peer {
        Peer {
            id: [id; 32],
            public_key: [id + 100; 32],
            endpoints: vec![addr],
            connection_state: _ConnectionState::Connected,
            last_seen: Instant::now(),
            quality_score: 80,
            latency_ms: Some(50),
            tags: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn add_and_get_peer() {
        let mut store = PeerStore::new(100, Duration::from_secs(300));
        let peer = create_test_peer(1, "192.168.1.1:51820".parse().unwrap());

        assert!(store.add_peer(peer));
        assert_eq!(store.len(), 1);

        let id = [1u8; 32];
        assert!(store.get_peer(&id).is_some());
    }

    #[test]
    fn find_by_addr() {
        let mut store = PeerStore::new(100, Duration::from_secs(300));
        let addr: SocketAddr = "192.168.1.1:51820".parse().unwrap();
        let peer = create_test_peer(1, addr);

        store.add_peer(peer);
        assert!(store._get_peer_by_addr(&addr).is_some());
    }

    #[test]
    fn max_peers_eviction() {
        let mut store = PeerStore::new(2, Duration::from_secs(300));

        let mut peer1 = create_test_peer(1, "192.168.1.1:51820".parse().unwrap());
        peer1.quality_score = 50;
        store.add_peer(peer1);

        let mut peer2 = create_test_peer(2, "192.168.1.2:51820".parse().unwrap());
        peer2.quality_score = 60;
        store.add_peer(peer2);

        let mut peer3 = create_test_peer(3, "192.168.1.3:51820".parse().unwrap());
        peer3.quality_score = 90;
        store.add_peer(peer3);

        // 应该淘汰最低评分的peer1
        assert_eq!(store.len(), 2);
        assert!(store.get_peer(&[1u8; 32]).is_none());
    }

    #[test]
    fn cleanup_expired() {
        let mut store = PeerStore::new(100, Duration::from_millis(1));

        let mut peer = create_test_peer(1, "192.168.1.1:51820".parse().unwrap());
        peer.last_seen = Instant::now() - Duration::from_millis(10);
        store.add_peer(peer);

        std::thread::sleep(Duration::from_millis(5));
        let expired = store.cleanup_expired();
        assert_eq!(expired.len(), 1);
    }
}
