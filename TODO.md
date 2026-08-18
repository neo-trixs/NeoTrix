# NeoTrix TODO 列表
> 智能同步生成，最后更新：2026-08-18

## 🔴 High 优先级

### ⬜ task-easytier-3: 后续吸收候选 (EasyTier)

**状态**: pending
**更新**: 2026-08-17T20:00:00
**描述**: 后续 session 可选吸收: NAT traversal/UDP hole-punch (NT-SHIELD)、OSPF 链路路由→GWT、per-session 棘轮加密 (SecureDatagramSession)、zero-copy ZCPacket→NT-MEMORY。见 notes/absorption-20260817-easytier*.md。

### ✅ task-easytier-1: 验证 secure discovery 测试通过

**状态**: done
**更新**: 2026-08-18
**描述**: `cargo test -p neotrix --lib -- nt_agent_protocol::discovery` 11/11 全绿 (roundtrip/replay/tamper/wrong-secret/plaintext-reject)。

### ✅ task-easytier-2: 能力树注册 (R-P100)

**状态**: done
**更新**: 2026-08-18
**描述**: `nt_agent_protocol::secure_discovery` bud + mature C1 + wiring_evidence (agent_cmds.rs:240-241)。