//! 联机房间核心 — multiplayer-game（服务端权威）+ higgsfield rooms（6 纯函数精神）.
//!
//! 公理：
//! - G1：客户端只发 intent，服务端 `validate` 后 `apply`，再广播结果（绝不信任客户端）
//! - G2：只传纯数据（id/字符串/数字），不传对象与回调
//! - G3：`view_for` 即信息隐藏——返回的是该玩家**唯一**能看到的东西（卡牌手牌/他人牌数）
//! - 传输：本模块提供 `LoopbackBus`（内存环回，真跑通双人房间）；真实 socket 为未来项，
//!   协议层（Action/Event/view）不变即可替换。
//!
//! 确定性：房间逻辑纯内存、无 wall-clock；回放 = 重放 action 序列。

use std::collections::{HashMap, VecDeque};

/// 玩家 id（服务端分配，房间内唯一；higgsfield 式：房主=1 亦可，但此处只要求唯一）
pub type PlayerId = u32;

/// 意图动作：纯数据（G2）。`kind` 如 "move"/"play"/"inc"，`payload` 为扁平字符串。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetAction {
    pub player: PlayerId,
    pub kind: String,
    pub payload: String,
}

/// 服务端广播事件：`to=None` 为全播，`Some(id)` 为单播（私密信息走单播）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetEvent {
    pub to: Option<PlayerId>,
    pub kind: String,
    pub payload: String,
}

impl NetEvent {
    pub fn broadcast(kind: &str, payload: &str) -> Self {
        Self { to: None, kind: kind.to_string(), payload: payload.to_string() }
    }

    pub fn unicast(to: PlayerId, kind: &str, payload: &str) -> Self {
        Self { to: Some(to), kind: kind.to_string(), payload: payload.to_string() }
    }

    /// 投递可见性：全播人人可见，单播仅目标可见（viewFor 在传输层的对应物）
    pub fn visible_to(&self, player: PlayerId) -> bool {
        self.to.map_or(true, |t| t == player)
    }
}

/// 权威逻辑契约（higgsfield 6 函数的 validate/apply/view 子集，Rust trait 版）
pub trait RoomLogic {
    /// 校验：轮到谁、距离、目标合法性——全在这里（G1 唯一防线）
    fn validate(&self, action: &NetAction) -> bool;
    /// 生效：返回待广播事件序列
    fn apply(&mut self, action: &NetAction) -> Vec<NetEvent>;
    /// 该玩家视角（G3：隐藏信息在此裁剪）
    fn view_for(&self, player: PlayerId) -> String;
}

/// 权威房间：成员表 + 逻辑 + 审计日志（`log` 即回放源）
pub struct Room<L: RoomLogic> {
    pub id: String,
    pub players: Vec<PlayerId>,
    pub logic: L,
    pub log: Vec<String>,
}

impl<L: RoomLogic> Room<L> {
    pub fn new(id: &str, logic: L) -> Self {
        Self { id: id.to_string(), players: Vec::new(), logic, log: Vec::new() }
    }

    pub fn join(&mut self, player: PlayerId) -> bool {
        if self.players.contains(&player) {
            return false;
        }
        self.players.push(player);
        self.log.push(format!("join:{}", player));
        true
    }

    pub fn leave(&mut self, player: PlayerId) -> bool {
        let n = self.players.len();
        self.players.retain(|p| p != &player);
        let left = self.players.len() != n;
        if left {
            self.log.push(format!("leave:{}", player));
        }
        left
    }

    /// 提交意图：非成员直接拒收；成员动作走 validate→apply，主动作进审计日志
    pub fn submit(&mut self, action: &NetAction) -> Vec<NetEvent> {
        if !self.players.contains(&action.player) {
            return vec![NetEvent::unicast(action.player, "reject", "not-member")];
        }
        if !self.logic.validate(action) {
            return vec![NetEvent::unicast(action.player, "reject", "invalid-action")];
        }
        self.log.push(format!("{}:{}:{}", action.player, action.kind, action.payload));
        self.logic.apply(action)
    }

    pub fn view_for(&self, player: PlayerId) -> String {
        self.logic.view_for(player)
    }
}

/// 环回传输：每玩家一个入队（`send` 投递、`drain` 取走），单测/本地双开即生产可用
#[derive(Debug, Default)]
pub struct LoopbackBus {
    queues: HashMap<PlayerId, VecDeque<NetEvent>>,
}

impl LoopbackBus {
    pub fn new() -> Self {
        Self { queues: HashMap::new() }
    }

    /// 路由一条事件：全播进所有已知玩家队列，单播只进目标队列
    pub fn send(&mut self, ev: NetEvent) {
        match ev.to {
            Some(t) => {
                self.queues.entry(t).or_default().push_back(ev);
            }
            None => {
                for q in self.queues.values_mut() {
                    q.push_back(ev.clone());
                }
            }
        }
    }

    pub fn register(&mut self, player: PlayerId) {
        self.queues.entry(player).or_default();
    }

    pub fn drain(&mut self, player: PlayerId) -> Vec<NetEvent> {
        self.queues.get_mut(&player).map_or(Vec::new(), |q| q.drain(..).collect())
    }
}

/// 演示逻辑：计数器房（每玩家独立计数 + 私密 secret，演示 view 隐藏）
#[derive(Debug, Default)]
pub struct CounterRoom {
    counters: HashMap<PlayerId, i32>,
    secrets: HashMap<PlayerId, String>,
}

impl RoomLogic for CounterRoom {
    fn validate(&self, action: &NetAction) -> bool {
        matches!(action.kind.as_str(), "inc" | "set_secret")
    }

    fn apply(&mut self, action: &NetAction) -> Vec<NetEvent> {
        match action.kind.as_str() {
            "inc" => {
                let c = self.counters.entry(action.player).or_insert(0);
                *c += 1;
                vec![NetEvent::broadcast("count", &format!("{}:{}", action.player, c))]
            }
            "set_secret" => {
                self.secrets.insert(action.player, action.payload.clone());
                vec![NetEvent::unicast(action.player, "secret_ok", "stored")]
            }
            _ => vec![NetEvent::unicast(action.player, "reject", "unknown-kind")],
        }
    }

    fn view_for(&self, player: PlayerId) -> String {
        // 只给：自己的 secret + 所有人的计数（他人 secret 永不出现——higgsfield 卡牌例）
        let mine = self.secrets.get(&player).map_or("-", String::as_str);
        let mut counts: Vec<String> = self.counters.iter()
            .map(|(p, c)| format!("{}:{}", p, c)).collect();
        counts.sort();
        format!("me:{} secret:{} counts:[{}]", player, mine, counts.join(","))
    }
}

/// 快照序列 guard（G2：位置等不可靠通道 latest-wins，旧包/重包丢弃）
#[derive(Debug, Default)]
pub struct SeqTracker {
    last: HashMap<PlayerId, u64>,
}

impl SeqTracker {
    pub fn new() -> Self {
        Self { last: HashMap::new() }
    }

    /// 新序列号才接受（首包恒接受；≤last 丢弃）
    pub fn accept(&mut self, player: PlayerId, seq: u64) -> bool {
        match self.last.get(&player) {
            None => {
                self.last.insert(player, seq);
                true
            }
            Some(&l) if seq > l => {
                self.last.insert(player, seq);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_player_room() -> Room<CounterRoom> {
        let mut r = Room::new("t1", CounterRoom::default());
        assert!(r.join(1));
        assert!(r.join(2));
        assert!(!r.join(1)); // 重复加入拒绝
        r
    }

    #[test]
    fn member_action_applies_and_broadcasts() {
        let mut r = two_player_room();
        let evs = r.submit(&NetAction { player: 1, kind: "inc".into(), payload: String::new() });
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].to, None);
        assert!(evs[0].payload.contains("1:1"));
    }

    #[test]
    fn seq_tracker_latest_wins() {
        let mut t = SeqTracker::new();
        assert!(t.accept(1, 0)); // 首包恒接受
        assert!(t.accept(1, 1));
        assert!(!t.accept(1, 1)); // 重包丢弃
        assert!(!t.accept(1, 0)); // 旧包丢弃
        assert!(t.accept(1, 5));
        assert!(t.accept(2, 0)); // 玩家独立
        assert!(!t.accept(2, 0));
    }

    #[test]
    fn outsider_rejected() {
        let mut r = two_player_room();
        let evs = r.submit(&NetAction { player: 9, kind: "inc".into(), payload: String::new() });
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].kind, "reject");
        assert_eq!(evs[0].to, Some(9));
        // 审计日志无此外来动作
        assert!(!r.log.iter().any(|l| l.starts_with("9:")));
    }

    #[test]
    fn view_hides_other_secrets() {
        let mut r = two_player_room();
        r.submit(&NetAction { player: 1, kind: "set_secret".into(), payload: "aaa".into() });
        r.submit(&NetAction { player: 2, kind: "set_secret".into(), payload: "bbb".into() });
        let v1 = r.view_for(1);
        let v2 = r.view_for(2);
        assert!(v1.contains("aaa") && !v1.contains("bbb"));
        assert!(v2.contains("bbb") && !v2.contains("aaa"));
    }

    #[test]
    fn loopback_routes_broadcast_and_unicast() {
        let mut bus = LoopbackBus::new();
        bus.register(1);
        bus.register(2);
        bus.send(NetEvent::broadcast("tick", "1"));
        bus.send(NetEvent::unicast(2, "secret_ok", "stored"));
        let d1 = bus.drain(1);
        let d2 = bus.drain(2);
        assert_eq!(d1.len(), 1); // 1 只收到全播
        assert_eq!(d2.len(), 2); // 2 收到全播+单播
        assert!(d1[0].visible_to(1) && !d1[0].visible_to(0) == false);
        assert!(NetEvent::unicast(2, "x", "y").visible_to(2));
        assert!(!NetEvent::unicast(2, "x", "y").visible_to(1));
    }

    #[test]
    fn leave_and_replay_log() {
        let mut r = two_player_room();
        r.submit(&NetAction { player: 1, kind: "inc".into(), payload: String::new() });
        assert!(r.leave(2));
        assert!(!r.players.contains(&2));
        // 日志即回放源：join/动作/leave 全记录
        assert!(r.log.iter().any(|l| l == "join:1"));
        assert!(r.log.iter().any(|l| l == "1:inc:"));
        assert!(r.log.iter().any(|l| l == "leave:2"));
    }
}
