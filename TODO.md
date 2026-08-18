# NeoTrix TODO 列表
> 智能同步生成，最后更新：2026-13-16T09:28:00

## 🔴 High 优先级

### ⬜ task-easytier-1: 验证 secure discovery 测试通过

**状态**: pending
**更新**: 2026-08-17T20:00:00
**描述**: `cargo test -p neotrix --lib -- nt_agent_protocol::discovery` 全绿 (roundtrip/replay/tamper/wrong-secret)。当前因 test 二进制编译超时未跑完，需在干净窗口复跑 (工作树被后台循环并发编辑)。

### ⬜ task-easytier-2: 能力树注册 (R-P100)

**状态**: pending
**更新**: 2026-08-17T20:00:00
**描述**: secure agent discovery 落地后 `neotrix-capability bud` 注册节点 (NT-ACT/agent_protocol), 再 `mature` 晋升 C1。

### ⬜ task-easytier-3: 后续吸收候选 (EasyTier)

**状态**: pending
**更新**: 2026-08-17T20:00:00
**描述**: 后续 session 可选吸收: NAT traversal/UDP hole-punch (NT-SHIELD)、OSPF 链路路由→GWT、per-session 棘轮加密 (SecureDatagramSession)、zero-copy ZCPacket→NT-MEMORY。见 notes/absorption-20260817-easytier*.md。

### 🔄 task-2: parent

**状态**: in_progress
**子代理**: ses_1786699677_4
**依赖**: task-1
**更新**: 2026-13-16T09:28:00
**效率分数**: 32.0

### ⬜ task-1: blockable

**状态**: in_progress
**更新**: 2026-13-16T09:28:00
**效率分数**: 30.0

### ⬜ task-3: child

**状态**: pending
**更新**: 2026-13-16T09:28:00
**效率分数**: 30.0

### ⬜ task-4: test

**状态**: pending
**更新**: 2026-13-16T09:28:00
**效率分数**: 30.0

### ⬜ task-5: json_test

**状态**: pending
**更新**: 2026-13-16T09:28:00
**效率分数**: 30.0

### ⬜ task-6: move_test

**状态**: pending
**更新**: 2026-13-16T09:28:00
**效率分数**: 30.0

## 🟡 Medium 优先级

### ✅ S-T1: First

**状态**: done
**更新**: 2026-13-16T09:28:00
**效率分数**: 20.0

### ⬜ S-T2: Second

**状态**: pending
**更新**: 2026-13-16T09:28:00
**效率分数**: 20.0

