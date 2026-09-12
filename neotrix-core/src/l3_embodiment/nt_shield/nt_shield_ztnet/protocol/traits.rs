//! C1: SANS-IO Protocol Engine — 零IO纯状态机协议引擎层
//!
//! 所有协议引擎实现统一 `NtProtocol` trait，由外部事件循环驱动。
//! 参考: Firezone connlib, boringtun, KVMem

use std::time::Instant;
use bytes::Bytes;

/// _Transmit 抽象 — 待发送的网络数据
#[derive(Debug, Clone)]
pub struct _Transmit {
    /// 目标地址
    pub dst: std::net::SocketAddr,
    /// 载荷数据
    pub payload: Bytes,
    /// 源地址 (可选)
    pub src: Option<std::net::SocketAddr>,
}

/// Protocol 事件 — 从状态机产出的事件
#[derive(Debug, Clone)]
pub enum _ProtocolEvent {
    /// 需要发送数据
    _Transmit(_Transmit),
    /// 隧道就绪
    TunnelReady {
        /// 发送方向索引
        send_idx: u32,
        /// 接收方向索引
        recv_idx: u32,
    },
    /// 数据包就绪 (待解密)
    PacketReady {
        /// 数据包内容
        data: Bytes,
        /// 源地址
        src: std::net::SocketAddr,
    },
    /// 连接关闭
    ConnectionClosed {
        /// 关闭原因
        reason: String,
    },
    /// 需要重试
    Retry {
        /// 重试延迟
        delay: std::time::Duration,
    },
}

/// SANS-IO 协议引擎 trait
///
/// 所有方法均为纯函数，不执行任何网络IO。
/// 外部事件循环负责:
/// 1. 调用 `handle_input()` 处理收到的数据
/// 2. 调用 `poll_output()` 获取待发送数据
/// 3. 调用 `handle_timeout()` 推进定时器
/// 4. 调用 `poll_timeout()` 获取下次唤醒时间
pub trait NtProtocol {
    /// 输入类型 (收到的网络数据)
    type Input;
    /// 产出类型 (事件)
    type Event;

    /// 处理输入数据
    ///
    /// 返回: 产出的事件列表
    fn handle_input(&mut self, input: Self::Input, now: Instant) -> Vec<Self::Event>;

    /// 获取待发送数据
    ///
    /// 返回: _Transmit 或 None
    fn poll_output(&mut self) -> Option<_Transmit>;

    /// 处理超时
    ///
    /// 返回: 产出的事件列表
    fn handle_timeout(&mut self, now: Instant) -> Vec<Self::Event>;

    /// 获取下次唤醒时间
    ///
    /// 返回: Instant 或 None (无定时器)
    fn poll_timeout(&self) -> Option<Instant>;

    /// 获取当前状态摘要 (用于调试/监控)
    fn state_summary(&self) -> _ProtocolState;

    /// 重置状态机
    fn reset(&mut self);
}

/// 协议状态摘要
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _ProtocolState {
    /// 初始状态
    Initial,
    /// 握手中
    Handshaking,
    /// 已建立
    Established,
    /// 正在关闭
    Closing,
    /// 已关闭
    Closed,
    /// 错误状态
    Error { message: String },
}

/// 事件循环驱动器
///
/// 将 NtProtocol 与 tokio 运行时集成
pub struct _EventLoop<P: NtProtocol> {
    /// 协议引擎
    protocol: P,
    /// 发送通道
    tx: Option<tokio::sync::mpsc::Sender<_Transmit>>,
    /// 事件通道
    event_tx: tokio::sync::mpsc::Sender<P::Event>,
}

impl<P: NtProtocol> _EventLoop<P> {
    /// 创建新的事件循环
    pub fn new(
        protocol: P,
        tx: tokio::sync::mpsc::Sender<_Transmit>,
        event_tx: tokio::sync::mpsc::Sender<P::Event>,
    ) -> Self {
        Self { protocol, tx: Some(tx), event_tx }
    }

    /// 运行事件循环
    pub async fn run(&mut self) {
        loop {
            // 检查是否有待发送数据
            while let Some(transmit) = self.protocol.poll_output() {
                if let Some(tx) = &self.tx {
                    if tx.send(transmit).await.is_err() {
                        return; // 发送通道关闭
                    }
                }
            }

            // 检查超时
            if let Some(timeout) = self.protocol.poll_timeout() {
                let now = Instant::now();
                if timeout <= now {
                    let events = self.protocol.handle_timeout(now);
                    for event in events {
                        if self.event_tx.send(event).await.is_err() {
                            return;
                        }
                    }
                } else {
                    // 等待超时
                    tokio::time::sleep_until(timeout.into()).await;
                }
            } else {
                // 无定时器，短暂休眠
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }
}

/// Peer 解复用器 — 根据来源地址分发到对应的状态机
pub struct _PeerDemux<P: NtProtocol> {
    /// 地址→状态机映射
    peers: std::collections::HashMap<std::net::SocketAddr, P>,
    /// 默认状态机工厂
    factory: Box<dyn Fn() -> P>,
}

impl<P: NtProtocol> _PeerDemux<P> {
    /// 创建新的解复用器
    pub fn new(factory: impl Fn() -> P + 'static) -> Self {
        Self {
            peers: std::collections::HashMap::new(),
            factory: Box::new(factory),
        }
    }

    /// 处理输入，路由到对应Peer
    pub fn handle_input(
        &mut self,
        src: std::net::SocketAddr,
        input: P::Input,
        now: Instant,
    ) -> Vec<P::Event> {
        let peer = self.peers.entry(src).or_insert_with(|| (self.factory)());
        peer.handle_input(input, now)
    }

    /// 获取活跃Peer数量
    pub fn _peer_count(&self) -> usize {
        self.peers.len()
    }

    /// 移除不活跃的Peer
    pub fn _prune_inactive(&mut self, _threshold: Instant) {
        self.peers.retain(|_, peer| {
            matches!(peer.state_summary(), _ProtocolState::Established | _ProtocolState::Handshaking)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transmit_creation() {
        let transmit = _Transmit {
            dst: "127.0.0.1:8080".parse().unwrap(),
            payload: Bytes::from_static(b"hello"),
            src: None,
        };
        assert_eq!(transmit.dst.port(), 8080);
    }

    #[test]
    fn protocol_state_variants() {
        let states = vec![
            _ProtocolState::Initial,
            _ProtocolState::Handshaking,
            _ProtocolState::Established,
            _ProtocolState::Closing,
            _ProtocolState::Closed,
            _ProtocolState::Error { message: "test".into() },
        ];
        assert_eq!(states.len(), 6);
    }

    #[test]
    fn peer_demux_creation() {
        let _demux: _PeerDemux<DummyProtocol> = _PeerDemux::new(|| DummyProtocol);
        // 编译通过即成功
    }

    struct DummyProtocol;

    impl NtProtocol for DummyProtocol {
        type Input = Bytes;
        type Event = ();

        fn handle_input(&mut self, _input: Self::Input, _now: Instant) -> Vec<Self::Event> {
            vec![]
        }

        fn poll_output(&mut self) -> Option<_Transmit> {
            None
        }

        fn handle_timeout(&mut self, _now: Instant) -> Vec<Self::Event> {
            vec![]
        }

        fn poll_timeout(&self) -> Option<Instant> {
            None
        }

        fn state_summary(&self) -> _ProtocolState {
            _ProtocolState::Initial
        }

        fn reset(&mut self) {}
    }
}
