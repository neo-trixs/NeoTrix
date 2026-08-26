# NeoTrix TODO 列表
> 智能同步生成，最后更新：2026-08-18

## 🔴 High 优先级

### ✅ task-awakening-e1: 纯 Rust 觉醒管线执行完成
**状态**: done
**更新**: 2026-08-25
**描述**: `examples/kb_awakening.rs` — E1 Embedding(7948×256维) + E2 语义边(+5820) + E3 图遍历(连通分量=1) + E4 GWT。总边 31568, 全图连通, 写入门禁通过 (26MB/2048MB)。

## 🔴 P0: KB 语料分库 + 写入门禁

### ✅ task-kb-p01: KB 语料分库（方案 B 表换名重建）
**状态**: done
**更新**: 2026-08-25
**描述**: 64GB 膨胀库 → 精简 1.6MB。方案 B 执行完成：nodes/edges swap → FTS 重建 → 孤儿清理。归档回滚点 `~/.neotrix/knowledge-archive-corpus-20260825.db` (64GB)。精简后 5077 节点 / 85 treats 边 / FTS 5077 行 / 1.6MB。

### ✅ task-kb-p03: 部署 KB 写入门禁
**状态**: done
**更新**: 2026-08-25
**描述**: 写入门禁已部署 `~/.neotrix/kb_write_gate.sh`。规则：KB >2MB 拒绝写入、批量 >10000 告警、活跃连接告警。门禁配置存于 kb_gate_config 表。


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

### ⬜ task-wave3-s1: 激活 nt_act 幽灵模块使 S1 三分态语义生效

**状态**: blocked (类型冲突需专门重构)
**更新**: 2026-08-25
**描述**: `src/neotrix/nt_act/` 模块存在但未在 neotrix/mod.rs 声明。添加声明暴露 209 errors（与 nt_act_autonomy/nt_act_code 类型冲突）。需专门 session 解决：合并重复类型定义、更新 import 路径、解决 E0119 conflicting implementations。相关测试 (nt_act_autonomy 等) 已全部通过 568/568。接续步骤见 notes/absorption-20260825-wave3-dshim-grokbot.md §五。

### ⬜ task-wave3-roadmap: dsh-im/grok-bot 路线图批次

**状态**: pending
**更新**: 2026-08-25
**描述**: 14 项机制路线图，映射目标+消费者已登记: Office 租约协议→nt_core_conn (消费者 orchestrator+approval)、能力探测 im_bridge+节流编辑流→nt_io_web、per-bot 策略→ConfirmationPolicy、per-turn 推理路由+CLI 认证复用→provider_pool、沙箱 owned-label 门禁→nt_shield_sandbox、usage schemaVersion→cost.rs、SHA-256 pinning→构建基础设施 (D9 机制化)。全表: notes/absorption-20260825-wave3-dshim-grokbot.md §四。