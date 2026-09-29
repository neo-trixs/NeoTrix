---
name: self-health
description: Self health monitoring protocol for the agent — check system vitality across compile/test/guard/KB/node dimensions without depending on the neotrix TUI (which blocks on provider wizard). Use when asked to check system health, run diagnostics, monitor self, 健康检测, 自我健康, 状态检查, or before/after significant work.
version: "1.0.0"
author: "NeoTrix NT-SHIELD"
triggers: health, self-health, 健康, 自我健康, 监控, diagnostics, 诊断, 自检, status
---

# Self-Health — 自我健康监控协议

Agent 自助健康检查。**不依赖 `./target/release/neotrix` TUI** — 因为 `main.rs:265` 在非 ops 命令启动时强制 provider wizard,阻塞自动化。本协议全部用 `cargo`/`rg`/`sqlite3`/`neotrix-experience` 直连,agent 可直接执行。

## 背景:现有机制与缺口

系统内在健康数据链路完整,但 agent 无自助入口:

| 已有机制 | 位置 | agent 可用性 |
|----------|------|--------------|
| `neotrix status` ops | `entry/status.rs` · `entry/sysops.rs` | ⚠️ 绕 provider 但只读 brain.json,不含节点健康 |
| SelfTest pass rate → branch health | `l5_cognition/nt_mind/` | ⚠️ 内存中,仅日志 |
| `neotrix-experience hub/query` | `~/.local/bin/neotrix-experience` | ✅ 可用 |
| 门禁脚本（本协议 D3 的主力） | `scripts/check-*.sh` · `scripts/ops/*` | ✅ 可用 |

> 2026-09-29 订正：原表列的 `/self-audit`、`/doctor`（`cli/commands/*.rs`）与
> `nt_core_consciousness_tree.rs` **均已随 `neotrix-core/src/cli/` 一并删除**。
> 本表的「缺口」判断不变 —— TUI 仍会在非 ops 命令启动时强制 provider wizard
> 阻塞自动化，故下面 D1–D6 全部走 `cargo`/`rg`/`sqlite3` 直连。

## 六维健康检查 (MUST run 每次会话收尾或用户请求)

### D1 编译健康
```sh
cargo check --all-targets -p neotrix 2>&1 | tail -5
# 期望: Finished; 记录 error 数, 与上次基线对比
```

### D2 测试健康 (定向, 避开网络 hang)
```sh
cargo test -p neotrix --lib -- nt_memory_crawl nt_world_absorber nt_mind_knowledge_pipeline nt_http 2>&1 | tail -5
# 全量 cargo test --lib 会因前序网络测试挂起(>300s), 必须定向
# 期望: N passed; 记录 pass/fail 数
```

### D3 门禁健康 (AGENTS.md 指针守恒)
```sh
node -e '
const { readFileSync } = require("fs");
const c = readFileSync("AGENTS.md", "utf8");
const lines = c.replace(/\n$/, "").split("\n");
const h2 = lines.filter((l) => l.startsWith("## ")).map((l) => l.replace(/^##\s+/, "").trim());
const allowed = ["0. 决策树：任务 → 动作（先查这里，别全文读）","1. 硬规则","2. 并行公约（2026-09-21 事故复盘）","3. 微操作公约（skill: nt-locate）","4. 三道闸（2026-09-27 事故后置入，违反即阻塞）","5. 扫描器告警 ≠ 缺陷（2026-09-27 差点把 bug 修进正确代码）","6. 正典索引","7. 本地模型"];
const bad = [];
if (lines.length > 130) bad.push(`total ${lines.length}>130 lines`);
if (Buffer.byteLength(c, "utf8") > 22000) bad.push(`total ${Buffer.byteLength(c, "utf8")}>22000 bytes`);
const unknown = h2.filter((s) => !allowed.includes(s));
if (unknown.length) bad.push(`non-whitelisted sections: ${unknown.join(", ")}`);
if (c.includes("## Experience Index") || c.includes("| Cycle | Domain | Summary |") || c.includes("| Cycle | Date |") || c.includes("| Cycle | Session |")) bad.push("inline Experience Index table (pointers live in KB only)");
if (bad.length) { console.log("❌ " + bad.join("; ")); process.exit(1); } else console.log("✅ AGENTS.md pointer-conservation passed");
'
```

### D4 KB 健康
```sh
neotrix-experience hub 2>&1 | rg -c '"cycles"|"branches"'   # hub 索引存在
sqlite3 ~/.neotrix/knowledge.db "PRAGMA integrity_check;"    # 期望 ok
sqlite3 ~/.neotrix/knowledge.db "SELECT COUNT(*) FROM kv_store WHERE namespace='experience';"
# 记录 cycle 数/branch 数 与上次基线对比; 异常增长或 0 → 告警
```

### D5 节点健康 (能力 map 概览)
```sh
# 生产接线: 谁在构造/注册 Capability (动态节点, 非静态 count)
echo "Capability 构造点: $(rg -l 'Capability \{' neotrix-core/src --type rust | rg -v 'registry.rs|mod.rs' | wc -l | tr -d ' ') 文件"
# SelfTest 生产注册点 (外部接线)
echo "SelfTest 接线: $(rg -l 'register\(Box::new' neotrix-core/src --type rust | rg -v 'self_test.rs' | wc -l | tr -d ' ') 文件"
# 各层模块数 (2026-09-29 订正: 旧版的 l1_body_impl…l9_transcendent_impl 九层
# 早已不是当前分层 —— 现为 l0_substrate → l6_meta 七层, 见 AGENTS.md §1)
for d in l0_substrate l1_action l2_perception l3_embodiment l4_emotion l5_cognition l6_meta; do
  echo "$d: $(ls neotrix-core/src/$d/ 2>/dev/null | wc -l | tr -d ' ') 模块"
done
# 第二棵逃逸树 (不参与 L0-L6, 见 DIR-REMEDY-2026-09-28.md §2.5)
echo "neotrix/(逃逸树): $(ls neotrix-core/src/neotrix/ 2>/dev/null | wc -l | tr -d ' ') 项"
```

### D6 依赖/死代码健康
```sh
# 未使用依赖 (cargo machete 若有) / 大文件警戒 (单个 .rs > 3000 行)
find neotrix-core/src -name "*.rs" -exec wc -l {} + | sort -rn | head -5
# 孤儿 bin (scripts 一次性的) — R-P79 检查
ls neotrix-core/src/bin/ | wc -l
```

## 健康判定

每个维度产出 `✅ / ⚠️ / ❌` + 一条证据。全绿 → `HEALTH: NORMAL`。任一 ❌ → 先修复再继续工作;⚠️ → 记录到经验吸收,不阻塞。

## 与经验树联动 (R-P 协议)

检查发现的新缺陷/模式 → 走 `experience-tree` 吸收,不进 AGENTS.md。健康基线存 KB `experience` namespace,检索: `neotrix-experience query --kw "health baseline"`。

## Agent 派发

本 skill 提供 `self-health-agent.md` 子 agent 定义(见同目录),可在 `task` 中派发执行完整六维检查。
