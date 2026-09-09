//! C3: ICE Agent (SANS-IO)
//!
//! ICE候选收集、连通性检查、优先级计算。
//! 零IO纯状态机，由外部驱动网络IO。

use std::net::SocketAddr;
use std::time::Instant;
use bytes::Bytes;

/// ICE候选类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CandidateType {
    /// Host候选 (本地接口)
    Host,
    /// Server Reflexive候选 (STUN)
    Srflx,
    /// Peer Reflexive候选 (连通性检查发现)
    Prflx,
    /// Relay候选 (TURN)
    Relay,
}

/// ICE候选
#[derive(Debug, Clone)]
pub struct IceCandidate {
    /// 候选类型
    pub candidate_type: CandidateType,
    /// 地址
    pub address: SocketAddr,
    /// 优先级
    pub priority: u32,
    /// 基础协议
    pub foundation: String,
    /// 组件ID (1=RTP, 2=RTCP)
    pub component: u16,
    /// 传输协议
    pub transport: String,
}

/// ICE对端候选
#[derive(Debug, Clone)]
pub struct RemoteCandidate {
    pub candidate_type: CandidateType,
    pub address: SocketAddr,
    pub priority: u32,
    pub foundation: String,
    pub component: u16,
    pub transport: String,
}

/// ICE连通性检查
#[derive(Debug, Clone)]
pub struct ConnectivityCheck {
    /// 本地候选
    pub local: IceCandidate,
    /// 远端候选
    pub remote: RemoteCandidate,
    /// 绑定请求数据
    pub binding_request: Bytes,
    /// 发送时间
    pub sent_at: Instant,
    /// 重试次数
    pub retries: u32,
}

/// ICE检查状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckState {
    /// 等待发送
    Waiting,
    /// 已发送，等待响应
    InProgress,
    /// 收到响应
    Succeeded,
    /// 超时
    Failed,
    /// 被取消
    Cancelled,
}

/// ICE对端
#[derive(Debug, Clone)]
pub struct IcePeer {
    /// 本地候选
    pub local: IceCandidate,
    /// 远端候选
    pub remote: RemoteCandidate,
    /// 检查状态
    pub state: CheckState,
    /// 最后一次检查
    pub last_check: Option<Instant>,
    /// 连通性分数 (0-100)
    pub score: u32,
}

/// ICE状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IceState {
    /// 初始
    Initial,
    /// 收集候选中
    Gathering,
    /// 连通性检查中
    Checking,
    /// 已连接
    Connected,
    /// 已完成 (最佳候选确定)
    Completed,
    /// 失败
    Failed,
    /// 已断开
    Disconnected,
}

/// ICE Agent 状态机
pub struct IceAgent {
    /// 状态
    pub state: IceState,
    /// 本地候选
    pub local_candidates: Vec<IceCandidate>,
    /// 远端候选
    pub remote_candidates: Vec<RemoteCandidate>,
    /// 活跃对端
    pub peers: Vec<IcePeer>,
    /// 最佳对端索引
    pub best_peer: Option<usize>,
    /// 检查超时时间
    pub check_timeout: std::time::Duration,
    /// 最大重试次数
    pub max_retries: u32,
}

impl IceAgent {
    /// 创建新ICE Agent
    pub fn new() -> Self {
        Self {
            state: IceState::Initial,
            local_candidates: Vec::new(),
            remote_candidates: Vec::new(),
            peers: Vec::new(),
            best_peer: None,
            check_timeout: std::time::Duration::from_secs(5),
            max_retries: 3,
        }
    }

    /// 添加本地候选
    pub fn add_local_candidate(&mut self, candidate: IceCandidate) {
        self.local_candidates.push(candidate);
        if self.state == IceState::Initial {
            self.state = IceState::Gathering;
        }
    }

    /// 添加远端候选
    pub fn add_remote_candidate(&mut self, candidate: RemoteCandidate) {
        self.remote_candidates.push(candidate);
    }

    /// 开始连通性检查
    pub fn start_checking(&mut self, stun_client: &mut crate::connectivity::stun::StunClient) -> Vec<ConnectivityCheck> {
        self.state = IceState::Checking;
        let mut checks = Vec::new();

        for local in &self.local_candidates {
            for remote in &self.remote_candidates {
                if local.component == remote.component {
                    if let Some(req) = self.create_binding_request(local, remote, stun_client) {
                        self.peers.push(IcePeer {
                            local: local.clone(),
                            remote: remote.clone(),
                            state: CheckState::InProgress,
                            last_check: Some(Instant::now()),
                            score: self.calculate_priority(local, remote),
                        });

                        checks.push(req);
                    }
                }
            }
        }

        checks
    }

    /// 创建绑定请求
    fn create_binding_request(
        &self,
        local: &IceCandidate,
        remote: &RemoteCandidate,
        stun_client: &mut crate::connectivity::stun::StunClient,
    ) -> Option<ConnectivityCheck> {
        let binding_request = stun_client.poll_request()?;

        Some(ConnectivityCheck {
            local: local.clone(),
            remote: remote.clone(),
            binding_request,
            sent_at: Instant::now(),
            retries: 0,
        })
    }

    /// 处理绑定响应
    pub fn handle_binding_response(
        &mut self,
        remote_addr: SocketAddr,
        response: &[u8],
        stun_client: &mut crate::connectivity::stun::StunClient,
    ) -> bool {
        if let Some(mapped_addr) = stun_client.handle_response(response) {
            // 更新对应peer的状态
            for peer in &mut self.peers {
                if peer.remote.address == remote_addr {
                    peer.state = CheckState::Succeeded;
                    peer.score = self.calculate_priority(&peer.local, &peer.remote);

                    // 更新最佳对端
                    self.update_best_peer();
                    return true;
                }
            }
        }
        false
    }

    /// 更新最佳对端
    fn update_best_peer(&mut self) {
        let mut best_idx = None;
        let mut best_score = 0;

        for (i, peer) in self.peers.iter().enumerate() {
            if peer.state == CheckState::Succeeded && peer.score > best_score {
                best_score = peer.score;
                best_idx = Some(i);
            }
        }

        self.best_peer = best_idx;

        if self.best_peer.is_some() {
            self.state = IceState::Completed;
        }
    }

    /// 计算优先级 (简化版)
    fn calculate_priority(&self, local: &IceCandidate, _remote: &RemoteCandidate) -> u32 {
        let type_pref = match local.candidate_type {
            CandidateType::Host => 126,
            CandidateType::Srflx => 100,
            CandidateType::Prflx => 110,
            CandidateType::Relay => 0,
        };

        let local_pref = type_pref * 65535 + (65535 - local.address.port() as u32);
        local_pref
    }

    /// 获取最佳对端
    pub fn get_best_peer(&self) -> Option<&IcePeer> {
        self.best_peer.map(|i| &self.peers[i])
    }

    /// 检查超时
    pub fn check_timeouts(&mut self, now: Instant) -> Vec<ConnectivityCheck> {
        let retries = Vec::new();

        for peer in &mut self.peers {
            if peer.state == CheckState::InProgress {
                if let Some(sent_at) = peer.last_check {
                    if now.duration_since(sent_at) > self.check_timeout {
                        if peer.local.candidate_type == CandidateType::Host {
                            // Host候选不重试
                            peer.state = CheckState::Failed;
                        } else if peer.remote.candidate_type != CandidateType::Relay {
                            // 非Relay候选重试
                            peer.state = CheckState::Waiting;
                        }
                    }
                }
            }
        }

        retries
    }

    /// 重置
    pub fn reset(&mut self) {
        self.state = IceState::Initial;
        self.local_candidates.clear();
        self.remote_candidates.clear();
        self.peers.clear();
        self.best_peer = None;
    }
}

impl Default for IceAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn ice_agent_creation() {
        let agent = IceAgent::new();
        assert_eq!(agent.state, IceState::Initial);
    }

    #[test]
    fn add_candidates() {
        let mut agent = IceAgent::new();

        agent.add_local_candidate(IceCandidate {
            candidate_type: CandidateType::Host,
            address: SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 5000),
            priority: 2130706431,
            foundation: "1".into(),
            component: 1,
            transport: "udp".into(),
        });

        agent.add_remote_candidate(RemoteCandidate {
            candidate_type: CandidateType::Host,
            address: SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 5000),
            priority: 2130706431,
            foundation: "1".into(),
            component: 1,
            transport: "udp".into(),
        });

        assert_eq!(agent.state, IceState::Gathering);
        assert_eq!(agent.local_candidates.len(), 1);
        assert_eq!(agent.remote_candidates.len(), 1);
    }

    #[test]
    fn priority_calculation() {
        let agent = IceAgent::new();

        let host = IceCandidate {
            candidate_type: CandidateType::Host,
            address: SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 5000),
            priority: 0,
            foundation: "1".into(),
            component: 1,
            transport: "udp".into(),
        };

        let srflx = IceCandidate {
            candidate_type: CandidateType::Srflx,
            address: SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)), 5000),
            priority: 0,
            foundation: "2".into(),
            component: 1,
            transport: "udp".into(),
        };

        let remote = RemoteCandidate {
            candidate_type: CandidateType::Host,
            address: SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 5000),
            priority: 0,
            foundation: "1".into(),
            component: 1,
            transport: "udp".into(),
        };

        let host_score = agent.calculate_priority(&host, &remote);
        let srflx_score = agent.calculate_priority(&srflx, &remote);

        assert!(host_score > srflx_score); // Host优先级高于Srflx
    }
}
