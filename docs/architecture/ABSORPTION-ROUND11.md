# NeoTrix Absorption Round 11: Meta-skill / Orchestration / MCP / Desktop

> **Absorption Date**: 2026-09-22 | **SIM**: SIM-45 | **Status**: Landed
> **Upstream**: R10 triage A–G 余项，6 读封顶（5 实读＋1 传输失败诚实记）。
> **This round: 0 新条款**（无 crisp 增量；NT-STD 停 1.0.8，显式记录非遗漏）。

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-74 | super-hermes 478★ (自写 thinking instructions；finding＋盲点 constraint footer；conservation law 非 bug 即结构不可能性；adversarial multi-pass；`.prism-history.md` 增长环＋诚实三局限；WHERE/WHEN/WHY 正交；cost inversion) | Meta-skill | verdict 格式注＋经验树注 |
| S-75 | ARES 401★ (fail-closed ScopeGuard＋双层 scope firewall；goal-directed DAG planner；30+ Pydantic 类型化模块＋dry-run；role gating；AES-256-GCM vault；MCP 单次 60s HMAC＋taint 隔离；dry-run-before-traffic；deterministic proof 非 CVE 匹配) | Platform | SEAL 注（金） |
| S-76 | jev-mcp 243★ (10 类型化工具；逐 claim fail-closed；invalid_response≠不确定性；阈值即初值；screen 只建议不阻断；extract 逐字非模型写；jev_gate claim 对 evidence；classify 风险轴 read_only/reversible/destructive→decide；模糊失败永不重试；error 脱敏；凭据卫生) | MCP tools | B03/ADR-0004 注（金） |
| S-77 | Exegol 3.1k★ | — | 正文传输失败，仅目录可机验；记未读（SIM-39 promptfoo 先例），R12 候选，不算吸收 |
| S-78 | agent-desktop 1.4k★ Rust (OS 无障碍树非像素；稳定 ref＋安全重试；STALE_REF/AMBIGUOUS_TARGET 永不擅选；actionability preflight；headless-by-default 阻 silent 副作用；skeleton 78–96% 降 token；session 命名空间＋JSONL trace；multi-agent ID；保留名 fail-closed；CDP 锁 127.0.0.1) | CLI | EQ-18 审计对照组 |
| S-79 | PI-Desktop 5k★ LGPL-3.0 (local-first 工作区；core 聚焦＋插件一切；权限层 Allow/Ask/Deny；Subagents＋Worker Sessions 独立上下文可检视；Project→Session→Agent→Work 非一次性 chat；模型即换件；凭据 OS Keychain；零遥测；"不该做成插件吗" core 纪律问) | Desktop | EQ-18 审计对照组＋工作区注 |

Rejected: 云 judge 依赖；新 SaaS；H 类（折叠维持）；Exegol 内容（未读不判）。

---

## 2. Pattern → NeoTrix Mapping（全 bake note）

- Verdict 格式（S-74）：finding＋constraint footer（maximized/sacrificed＋指路他 skill）；
  conservation law 思维并入 rev-officer；增长环局限三问（pruning/索引/验证）并入经验树设计。
- SEAL 输入（S-75，金）：scope firewall（fail-closed CIDR＋双层拦截）／dry-run-before-traffic／
  单次 HMAC token／taint 隔离／role gating／evidence redaction——六件即 SEAL 威胁模型骨架，
  P-task L5 取用。
- Completion 门（S-76，金）：jev_gate（patch review＋claim 对 evidence，双真才 auto，
  自信矛盾即 escalate）即 ADR-0004 Stage 门的生产形态，Stage 验收引用；
  reversible/destructive 分级即 NTS-B03 不可逆检查点的外部同构；
  模糊失败永不重试（无幂等键）并入 outbox 纪律。
- 桌面审计对照（S-78/S-79 → EQ-18 checklist 增补）：ref 稳定＋STALE/AMBIGUOUS 永不擅选；
  headless-by-default；权限对象显式 granted/denied/unknown；CDP/端点锁 loopback；
  敏感操作过权限层；凭据 OS Keychain；零遥测声明可验。
  PI 的 "Would this be better as a Plugin?" 收为 core 聚焦纪律问（NTS-B11 旁注）。
- 阈值诚实（S-76）：cookbook 初值＋自调声明并入 SDB 评分惯例（AERS/R9：宣称未校准即诚实）。

---

## 3. Landings

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R11-1 | SIM-45 record | row + §44 present |
| L-R11-2 | BLUEPRINT v1.6.10 changelog | line present |
| L-R11-3 | EQ-18 checklist 增补项（§2 桌面段） | rows referenced |

NT-STD 停 1.0.8（本轮零条款是决议，不是拖延）。

Deferred: Exegol 正文（R12）；本地 judge（P-task 延续）；SEAL 骨架取用（P-task L5）；
H 类（维持折叠）；R12 A–G 余项。

---

*End of Absorption Round 11*
