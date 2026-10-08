//! C1: Noise IKpsk2 会话协议引擎 —— 零 IO 纯状态机
//!
//! 本模块是 [`crate::crypto::noise_handshake`] 的**唯一生产消费者**，把 C0 的
//! Noise IKpsk2 握手（原语层，无 IO）装配成 C1 的 SANS-IO 协议引擎：
//! 输入事件 → 状态迁移 → 待发送字节 / 产出事件，全程不碰 socket。
//!
//! ## 为什么是 IKpsk2 而不是 IK
//!
//! `IKpsk2` 在握手末尾用 PSK 混入链式密钥（`MixKeyAndHash`）。PSK 只能来自
//! 带外（预共享配置 / 已建立的信控通道），**绝不能由数据通道协商** —— 否则
//! 首次连接的中间人只需自举一个 PSK 即可。`_NoiseIkSession` 因此把 PSK 做成
//! 构造期参数，不提供任何「从网络学 PSK」的入口。
//!
//! ## 分层
//!
//! C1 依赖 C0，反向不依赖 —— 符合 `ztnet/mod.rs` 声明的六层单向依赖。
//! C2 `packet` 负责把 `poll_output()` 产出的字节封成 IP 包；C1 不感知 IP。
//!
//! ## 用法
//!
//! ```ignore
//! let mut ep = _NoiseIkSession::initiator(local_static, peer_static, psk)?;
//! let evs = ep.handle_input(_SessionInput::Start, now);      // 产出 Initiated
//! let tx = ep.poll_output();                                 // 取 msg1 (96B)
//! // …真实网络收发…
//! let evs = ep.handle_input(_SessionInput::Wire(tx_bytes), now); // 产出 msg2
//! let tx2 = ep.poll_output();
//! let evs = ep.handle_input(_SessionInput::Wire(msg2_bytes), now); // 产出 Established
//! ```

use std::collections::VecDeque;
use std::time::Instant;

use bytes::Bytes;

use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::aead::{AeadKey, Nonce};
use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::keys::{PrivateKey, PublicKey};
use crate::l3_embodiment::nt_shield::nt_shield_ztnet::crypto::noise_handshake::{
    _HandshakeResult, _NoiseError, _NoiseHandshake,
};
use crate::l3_embodiment::nt_shield::nt_shield_ztnet::protocol::traits::{
    NtProtocol, _ProtocolState, _Transmit,
};

/// 传输轮 nonce 上限。Noise 传输态要求每条方向的计数器从 0 单调递增，
/// 且**绝不可复用**（复用 ⇒ ChaCha20-Poly1305 keystream 复用 ⇒ 明文关系泄露）。
///
/// 上限取 u64::MAX/2 而非 u64::MAX：留出足够余量后主动耗尽会话，
/// 强制重新握手 —— 这比在计数回绕后继续用更安全，也比 `u64` 溢出可定义。
const _MAX_TRANSPORT_NONCE: u64 = u64::MAX / 2;

/// 单帧明文上限。与 `_MAX_TRANSPORT_NONCE` 同为防御性上限：AEAD 单次操作
/// 对超大明文的内存占用是线性增长的，本引擎是 SANS-IO 纯状态机，
/// 不应被一个畸形帧撑爆。
const _MAX_PLAINTEXT_LEN: usize = 64 * 1024;

// ─────────────────────────── 输入 / 事件 ───────────────────────────

/// 协议引擎输入
#[derive(Debug, Clone)]
pub enum _SessionInput {
    /// 请求开始握手。responder 侧忽略（由 `Wire` 首包自动触发）。
    Start,
    /// 从网络收到的字节。握手期视作握手消息，建立后视作密文帧。
    Wire(Bytes),
    /// 关闭会话。
    Close,
}

/// 协议引擎产出事件
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _SessionEvent {
    /// 握手消息已就绪，等待上层发往对端
    HandshakeMessageReady { msg: Vec<u8>, is_first: bool },
    /// 会话建立完成，`peer_static_public` 为**已认证**的对端静态公钥
    /// （IK 模式下对端静态公钥在 msg1 阶段即被确认，不是握手后才学到）
    Established { peer_static_public: [u8; 32] },
    /// 收到并成功解密的明文
    DataReceived { plaintext: Vec<u8> },
    /// 失败。`reason` 为面向日志的静态串，不含密钥材料。
    Failed { reason: String },
    /// 会话已关闭（主动或被动）
    Closed,
}

// ─────────────────────────── 会话引擎 ───────────────────────────

/// Noise IKpsk2 单会话协议引擎（SANS-IO）
pub struct _NoiseIkSession {
    role: _Role,
    /// 握手期持有的引擎；握手完成后置 None（密钥已提取到 send/recv）
    hs: Option<_NoiseHandshake>,
    /// 传输期发送密钥（含 nonce 计数）
    send: Option<_TxCipher>,
    /// 传输期接收密钥（含 nonce 计数）
    recv: Option<_RxCipher>,
    /// 待发送队列（握手消息 / 加密帧）
    txq: VecDeque<Vec<u8>>,
    /// 对端地址。SANS-IO 侧不发包，但 `NtProtocol::poll_output` 须给出
    /// `_Transmit.dst`；IKpsk2 是单对端会话，故地址在构造期确定。
    peer: std::net::SocketAddr,
    state: _ProtocolState,
    /// 本端静态公钥（事件里回显用）
    local_static: PublicKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum _Role {
    Initiator,
    Responder,
}

struct _TxCipher {
    key: AeadKey,
    nonce: u64,
}

struct _RxCipher {
    key: AeadKey,
    nonce: u64,
}

impl _NoiseIkSession {
    /// 创建 Initiator 侧会话。
    ///
    /// `remote_static` 是预配置的**带外可信**对端静态公钥 —— IK 模式的安全
    /// 前提正是「initiator 预先知道 responder 的静态公钥」，否则该模式退化为
    /// 无身份认证的匿名握手。
    pub fn initiator(
        local_static: PrivateKey,
        remote_static: PublicKey,
        psk: Option<[u8; 32]>,
        peer: std::net::SocketAddr,
    ) -> Result<Self, _NoiseError> {
        let local_pub = local_static.public();
        Ok(Self::assemble(
            _Role::Initiator,
            local_pub,
            peer,
            _NoiseHandshake::_initiator(local_static, remote_static, psk),
        ))
    }

    /// 创建 Responder 侧会话。
    pub fn responder(
        local_static: PrivateKey,
        psk: Option<[u8; 32]>,
        peer: std::net::SocketAddr,
    ) -> Result<Self, _NoiseError> {
        let local_pub = local_static.public();
        Ok(Self::assemble(
            _Role::Responder,
            local_pub,
            peer,
            _NoiseHandshake::_responder(local_static, psk),
        ))
    }

    fn assemble(
        role: _Role,
        local_static: PublicKey,
        peer: std::net::SocketAddr,
        hs: _NoiseHandshake,
    ) -> Self {
        Self {
            role,
            hs: Some(hs),
            send: None,
            recv: None,
            txq: VecDeque::new(),
            state: _ProtocolState::Initial,
            local_static,
            peer,
        }
    }

    /// 发 Start 并取回 msg1（便捷方法，供上层编排与测试使用）
    pub fn poll_output_after_start(&mut self) -> Bytes {
        let _ = self.handle_input(_SessionInput::Start, Instant::now());
        self.poll_output().map(|t| t.payload).unwrap_or_default()
    }

    /// 本端静态公钥（IK 模式下用于上层做对端身份比对）
    pub fn local_static(&self) -> PublicKey {
        self.local_static.clone()
    }

    /// 提取握手结果为传输密钥。
    fn promote(&mut self, r: _HandshakeResult) {
        self.send = Some(_TxCipher { key: r.send_key, nonce: 0 });
        self.recv = Some(_RxCipher { key: r.recv_key, nonce: 0 });
        self.hs = None;
        self.state = _ProtocolState::Established;
    }

    /// 传输期加密一帧。`salt` 供上层的会话密钥混淆使用时透传为 nonce 的一部分。
    fn seal_frame(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, _NoiseError> {
        let tx = self.send.as_mut().ok_or(_NoiseError::InvalidState)?;
        if tx.nonce >= _MAX_TRANSPORT_NONCE {
            return Err(_NoiseError::InvalidState);
        }
        let ct = tx
            .key
            .seal_with_ad(&_nonce_of(tx.nonce), &[], plaintext)
            .map_err(|_| _NoiseError::EncryptionFailed)?;
        tx.nonce += 1;
        Ok(ct)
    }

    fn open_frame(&mut self, ct: &[u8]) -> Result<Vec<u8>, _NoiseError> {
        let rx = self.recv.as_mut().ok_or(_NoiseError::InvalidState)?;
        if rx.nonce >= _MAX_TRANSPORT_NONCE {
            return Err(_NoiseError::InvalidState);
        }
        let mut buf = ct.to_vec();
        let pt = rx
            .key
            .open_with_ad(&_nonce_of(rx.nonce), &[], &mut buf)
            .map_err(|_| _NoiseError::DecryptionFailed)?
            .to_vec();
        // 只在解密成功后才自增。但注意：**这不是「跳过坏帧继续」**——
        // Noise 传输态收发共用对称 nonce 递增，发送端每发一帧都会推进。
        // 一旦某帧解密失败，接收端计数器就落后于发送端，流**永久失步**，
        // 没有任何办法恢复（无法推断发送端计数到了哪）。故上层收到
        // `DecryptionFailed` 必须**作废整个会话重新握手**，而不是继续收。
        // 这不是可修的 DoS，而是该设计的固有性质。
        rx.nonce += 1;
        Ok(pt)
    }

    /// 发送应用层明文（仅 Established 态可用）
    pub fn send_data(&mut self, plaintext: &[u8]) -> Result<(), _NoiseError> {
        if self.state != _ProtocolState::Established {
            return Err(_NoiseError::InvalidState);
        }
        if plaintext.len() > _MAX_PLAINTEXT_LEN {
            return Err(_NoiseError::InvalidState);
        }
        let ct = self.seal_frame(plaintext)?;
        self.txq.push_back(ct);
        Ok(())
    }
}

/// 传输轮 nonce：4 字节零 || LE64(counter)（Noise 规范，与 WireGuard 相反）
fn _nonce_of(n: u64) -> Nonce {
    let mut b = [0u8; 12];
    b[4..12].copy_from_slice(&n.to_le_bytes());
    Nonce::from_bytes(&b)
}

// ─────────────────────────── NtProtocol 实现 ───────────────────────────

impl NtProtocol for _NoiseIkSession {
    type Input = _SessionInput;
    type Event = _SessionEvent;

    fn handle_input(&mut self, input: Self::Input, _now: Instant) -> Vec<Self::Event> {
        let mut evs = Vec::new();
        match input {
            _SessionInput::Start => {
                if self.role == _Role::Initiator && self.state == _ProtocolState::Initial {
                    if let Some(h) = self.hs.as_mut() { match h._create_message1() {
                        Ok(msg) => {
                            self.state = _ProtocolState::Handshaking;
                            evs.push(_SessionEvent::HandshakeMessageReady {
                                msg: msg.clone(),
                                is_first: true,
                            });
                            self.txq.push_back(msg);
                        }
                        Err(e) => {
                            self.state = _ProtocolState::Error {
                                message: e.to_string(),
                            };
                            evs.push(_SessionEvent::Failed { reason: e.to_string() });
                        }
                    } }
                }
            }
            _SessionInput::Wire(bytes) => match self.state {
                _ProtocolState::Initial | _ProtocolState::Handshaking => {
                    self.on_wire(&bytes, &mut evs);
                }
                _ProtocolState::Established => match self.open_frame(&bytes) {
                    Ok(pt) => evs.push(_SessionEvent::DataReceived { plaintext: pt }),
                    Err(e) => {
                        // 流已失步且不可恢复 ⇒ 作废会话，强制重新握手
                        let reason = format!("{} (session desynchronized, must re-handshake)", e);
                        self.send = None;
                        self.recv = None;
                        self.hs = None;
                        self.state = _ProtocolState::Error {
                            message: reason.clone(),
                        };
                        evs.push(_SessionEvent::Failed { reason });
                    }
                },
                ref other => {
                    let msg = format!("unexpected input in state {other:?}");
                    self.state = _ProtocolState::Error { message: msg.clone() };
                    evs.push(_SessionEvent::Failed { reason: msg });
                }
            },
            _SessionInput::Close => {
                self.state = _ProtocolState::Closed;
                evs.push(_SessionEvent::Closed);
            }
        }
        evs
    }

    fn poll_output(&mut self) -> Option<_Transmit> {
        let dst = self.peer;
        self.txq.pop_front().map(|payload| _Transmit {
            dst,
            payload: Bytes::from(payload),
            src: None,
        })
    }

    fn handle_timeout(&mut self, _now: Instant) -> Vec<Self::Event> {
        Vec::new()
    }

    fn poll_timeout(&self) -> Option<Instant> {
        // 重连退避由 C3/C5 的重试策略负责，C1 不持有定时器（保持零 IO）。
        None
    }

    fn state_summary(&self) -> _ProtocolState {
        self.state.clone()
    }

    fn reset(&mut self) {
        self.state = _ProtocolState::Initial;
        self.send = None;
        self.recv = None;
        self.txq.clear();
    }
}

impl _NoiseIkSession {
    /// 握手期收到字节。
    fn on_wire(&mut self, bytes: &[u8], evs: &mut Vec<_SessionEvent>) {
        // responder 收到首包即自动进入握手（无需显式 Start）
        if self.state == _ProtocolState::Initial {
            self.state = _ProtocolState::Handshaking;
        }
        let role = self.role;
        let h = match self.hs.as_mut() {
            Some(h) => h,
            None => {
                evs.push(_SessionEvent::Failed {
                    reason: "handshake already consumed".to_string(),
                });
                return;
            }
        };
        let res = match role {
            _Role::Initiator => h._consume_message2(bytes).map(|r| (None, r)),
            _Role::Responder => match h._consume_message1(bytes) {
                // `_consume_message1` 返回的 initiator 静态公钥是**经认证**的
                // （由 msg1 内加密的静态字段解出），与 `_create_message2` 结果里的
                // `peer_static_public` 同源，故此处无需重复使用。
                Ok(_peer) => match h._create_message2() {
                    Ok((msg, r)) => Ok((Some(msg), r)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            },
        };
        match res {
            Ok((out_msg, r)) => {
                // `PublicKey` 非 Copy：先拷贝字节，再整份交给 promote
                let peer = *r.peer_static_public.as_bytes();
                self.promote(r);
                if let Some(msg) = out_msg {
                    evs.push(_SessionEvent::HandshakeMessageReady {
                        msg: msg.clone(),
                        is_first: false,
                    });
                    self.txq.push_back(msg);
                }
                evs.push(_SessionEvent::Established {
                    peer_static_public: peer,
                });
            }
            Err(e) => {
                self.state = _ProtocolState::Error {
                    message: e.to_string(),
                };
                evs.push(_SessionEvent::Failed { reason: e.to_string() });
            }
        }
    }
}

// ─────────────────────────── 测试 ───────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l3_embodiment::nt_shield::nt_shield_ztnet::protocol::traits::NtProtocol;

    fn hx(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    fn bk32(v: &[u8]) -> [u8; 32] {
        v.try_into().unwrap()
    }

    /// 与 C0 官方向量验收测试同源的一组静态密钥，使 C1 的基线与 C0 一致。
    const INIT_STATIC: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const RESP_STATIC: &str = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
    const PSK_HEX: &str = "2176657279736563726574766572797365637265747665727973656372657421";

    fn si() -> PrivateKey {
        PrivateKey::from_bytes(&bk32(&hx(INIT_STATIC)))
    }
    fn sr() -> PrivateKey {
        PrivateKey::from_bytes(&bk32(&hx(RESP_STATIC)))
    }
    fn psk() -> Option<[u8; 32]> {
        Some(bk32(&hx(PSK_HEX)))
    }

    /// 在内存里把两侧引擎对接（模拟网络），返回 initiator 侧的 Established 事件
    /// 携带的对端静态公钥。
    fn handshake(
        i: &mut _NoiseIkSession,
        r: &mut _NoiseIkSession,
    ) -> (Bytes, Bytes) {
        let e1 = i.handle_input(_SessionInput::Start, Instant::now());
        assert!(matches!(
            e1.as_slice(),
            [_SessionEvent::HandshakeMessageReady { is_first: true, .. }]
        ));
        let msg1 = i.poll_output().map(|t| t.payload).expect("initiator 应产出 msg1");

        let e2 = r.handle_input(_SessionInput::Wire(msg1.clone()), Instant::now());
        assert!(e2.iter().any(|e| matches!(e, _SessionEvent::Established { .. })), "{e2:?}");
        let msg2 = r.poll_output().map(|t| t.payload).expect("responder 应产出 msg2");

        let e3 = i.handle_input(_SessionInput::Wire(msg2.clone()), Instant::now());
        assert!(e3.iter().any(|e| matches!(e, _SessionEvent::Established { .. })), "{e3:?}");
        (msg1, msg2)
    }

    fn addr(port: u16) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn pair() -> (_NoiseIkSession, _NoiseIkSession) {
        let i = _NoiseIkSession::initiator(si(), sr().public(), psk(), addr(51820)).unwrap();
        let r = _NoiseIkSession::responder(sr(), psk(), addr(51821)).unwrap();
        (i, r)
    }

    #[test]
    fn handshake_message_sizes_match_official_vector() {
        let (mut i, mut r) = pair();
        let (msg1, msg2) = handshake(&mut i, &mut r);
        assert_eq!(msg1.len(), 96, "IKpsk2 msg1 = e(32)+enc_s(48)+enc_empty(16)");
        assert_eq!(msg2.len(), 48, "IKpsk2 msg2 = e(32)+enc_empty(16)");
        assert_eq!(i.state_summary(), _ProtocolState::Established);
        assert_eq!(r.state_summary(), _ProtocolState::Established);
    }

    /// IK 的安全要点：responder 侧学到的 initiator 静态公钥来自 msg1 内的
    /// 加密静态字段，是**经认证**的（错的静态私钥解不开 msg1）。
    #[test]
    fn responder_authenticates_initiator_static_from_msg1() {
        let (mut i, mut r) = pair();
        let msg1 = i.poll_output_after_start();
        let e = r.handle_input(_SessionInput::Wire(msg1), Instant::now());
        let peer = e
            .iter()
            .find_map(|ev| match ev {
                _SessionEvent::Established { peer_static_public } => Some(*peer_static_public),
                _ => None,
            })
            .expect("responder 应产出 Established");
        assert_eq!(peer, *si().public().as_bytes(), "responder 认证到的应是 initiator 静态公钥");
    }

    #[test]
    fn wrong_initiator_static_key_cannot_complete_msg1() {
        // responder 配的是**另一把**静态私钥 ⇒ 与 initiator 加密进 msg1 的
        // 静态公钥不匹配 ⇒ msg1 的 s 字段解密失败
        let mut i = _NoiseIkSession::initiator(si(), sr().public(), psk(), addr(1)).unwrap();
        let other = PrivateKey::from_bytes(&[7u8; 32]);
        let mut r = _NoiseIkSession::responder(other, psk(), addr(2)).unwrap();
        let _ = i.handle_input(_SessionInput::Start, Instant::now());
        let msg1 = i.poll_output().map(|t| t.payload).unwrap();
        let e = r.handle_input(_SessionInput::Wire(msg1), Instant::now());
        assert!(
            e.iter().any(|ev| matches!(ev, _SessionEvent::Failed { .. })),
            "静态公钥不匹配必须失败（认证绑定失效）：{e:?}"
        );
        assert_ne!(r.state_summary(), _ProtocolState::Established);
    }

    #[test]
    fn data_flows_both_directions() {
        let (mut i, mut r) = pair();
        handshake(&mut i, &mut r);

        i.send_data(b"ping from initiator").unwrap();
        let f = i.poll_output().expect("应产出密文帧").payload;
        assert_ne!(&f[..], b"ping from initiator", "线上必须是密文");
        assert_eq!(
            r.handle_input(_SessionInput::Wire(f), Instant::now()),
            vec![_SessionEvent::DataReceived { plaintext: b"ping from initiator".to_vec() }]
        );

        r.send_data(b"pong from responder").unwrap();
        let f = r.poll_output().unwrap().payload;
        assert_eq!(
            i.handle_input(_SessionInput::Wire(f), Instant::now()),
            vec![_SessionEvent::DataReceived { plaintext: b"pong from responder".to_vec() }]
        );
    }

    /// 传输态收发共用对称 nonce 递增 ⇒ 任一帧解密失败即**流永久失步**
    /// （发送端已推进，接收端无法推断其计数）⇒ 会话必须作废，不得继续收。
    ///
    /// ⚠️ 本测试初版断言的是「失败帧不推进 nonce ⇒ 后续合法帧仍能解」——
    ///    那个性质**不存在**，是我写死的错误断言。测试跑红才暴露了设计缺陷。
    ///    正确性质是下面这个：失步后引擎作废会话，后续帧不得再当数据交付。
    #[test]
    fn tampered_frame_desynchronizes_and_invalidates_session() {
        let (mut i, mut r) = pair();
        handshake(&mut i, &mut r);

        // 基线：正常帧可解
        i.send_data(b"first").unwrap();
        let e = r.handle_input(_SessionInput::Wire(i.poll_output().unwrap().payload), Instant::now());
        assert_eq!(e, vec![_SessionEvent::DataReceived { plaintext: b"first".to_vec() }]);

        // 篡改一帧 ⇒ 失败 + 会话作废
        i.send_data(b"second").unwrap();
        let mut bad = i.poll_output().unwrap().payload.to_vec();
        bad[0] ^= 0xff;
        let e = r.handle_input(_SessionInput::Wire(Bytes::from(bad)), Instant::now());
        assert!(matches!(e[0], _SessionEvent::Failed { .. }), "{e:?}");
        assert!(
            matches!(r.state_summary(), _ProtocolState::Error { .. }),
            "解密失败后会话必须作废（流不可恢复），实际 {:?}",
            r.state_summary()
        );

        // 后续帧不得再被当作数据交付（也不能凭空「恢复」）
        i.send_data(b"third").unwrap();
        let e = r.handle_input(_SessionInput::Wire(i.poll_output().unwrap().payload), Instant::now());
        assert!(
            !e.iter().any(|ev| matches!(ev, _SessionEvent::DataReceived { .. })),
            "已作废的会话不得再交付数据：{e:?}"
        );
        // 重新握手是唯一的出路
        r.reset();
        assert_eq!(r.state_summary(), _ProtocolState::Initial);
    }

    #[test]
    fn psk_mismatch_does_not_reach_established_on_initiator() {
        let mut i = _NoiseIkSession::initiator(si(), sr().public(), Some([1u8; 32]), addr(1)).unwrap();
        let mut r = _NoiseIkSession::responder(sr(), Some([2u8; 32]), addr(2)).unwrap();
        let _ = i.handle_input(_SessionInput::Start, Instant::now());
        let msg1 = i.poll_output().map(|t| t.payload).unwrap();
        let _ = r.handle_input(_SessionInput::Wire(msg1), Instant::now());
        let msg2 = r.poll_output().map(|t| t.payload).expect("msg1 不含 psk 派生解密，responder 仍会产出 msg2");
        let e = i.handle_input(_SessionInput::Wire(msg2), Instant::now());
        assert!(
            !e.iter().any(|ev| matches!(ev, _SessionEvent::Established { .. })),
            "PSK 不一致时 initiator 绝不可 Established：{e:?}"
        );
    }

    #[test]
    fn short_message_in_handshake_is_rejected() {
        let (mut i, mut r) = pair();
        let _ = i.handle_input(_SessionInput::Start, Instant::now());
        let _ = i.poll_output();
        let e = r.handle_input(_SessionInput::Wire(Bytes::from_static(b"too short")), Instant::now());
        assert!(e.iter().any(|ev| matches!(ev, _SessionEvent::Failed { .. })), "{e:?}");
        assert_ne!(r.state_summary(), _ProtocolState::Established);
    }

    #[test]
    fn data_before_established_is_rejected() {
        let (mut i, _r) = pair();
        assert!(i.send_data(b"x").is_err(), "未建立前 send_data 必须报错");
        let e = i.handle_input(_SessionInput::Wire(Bytes::from_static(b"junkjunkjunk")), Instant::now());
        assert!(!e.iter().any(|ev| matches!(ev, _SessionEvent::Established { .. })));
    }

    #[test]
    fn plaintext_cap_enforced_after_established() {
        let (mut i, mut r) = pair();
        handshake(&mut i, &mut r);
        assert!(i.send_data(&vec![0u8; _MAX_PLAINTEXT_LEN]).is_ok());
        assert!(i.send_data(&vec![0u8; _MAX_PLAINTEXT_LEN + 1]).is_err());
    }

    #[test]
    fn close_and_reset_return_to_initial() {
        let (mut i, _r) = pair();
        let e = i.handle_input(_SessionInput::Close, Instant::now());
        assert_eq!(e, vec![_SessionEvent::Closed]);
        assert_eq!(i.state_summary(), _ProtocolState::Closed);
        i.reset();
        assert_eq!(i.state_summary(), _ProtocolState::Initial);
    }
}
