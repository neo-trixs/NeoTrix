# NeoTrix 有序执行清单 (SIM-32)

> **原则**: 按序执行，跳序需 SIM 记录理由。`完成定义`是唯一的 done 标准，感觉不算。
> 状态机：⬜待办 → 🟨进行 → 🟩完成 / 🟥阻塞(注明阻塞物)。更新状态只改本表，不另开文档。

## T0. 收尾确认

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-01 | B1 三单测执行 | `cargo test --lib nt_core_cross_layer` 3/3 绿 | P1 | 无（机器独占窗口） | 🟩 (SIM-34 本地实跑) |
| EQ-02 | ADR-0002 关闭 | B1 首绿后，pending-CI 转 closed | P1 | EQ-01 | 🟩 (Resolution 已记) |

## T1. 守卫可信度（先修裁判，再踢比赛）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-03 | 死守卫修退役 SIM＋ADR | Layer/Core 二守卫去向书面化（二选一：重定向＋allowlist／退役） | Architect | 无 | 🟩 (ADR-0003: 退役) |
| EQ-04 | 执行 EQ-03 决议 | 代码落地＋对应单测绿 | L5 | EQ-03 | 🟩 (13/13 实跑绿，注册表 8→6) | ⬜ |

## T2. P1-02 越层消减（主战场，197 行／12 组）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-05 | H-08 math 改直引 L0（首血；原下沉方案已证伪；L1×l5计数26→23已验，全门待CI） | `check-layer-deps` L1×L5 组计数下降＋全量门绿 | L1 | EQ-04 | 🟨 (代码完，待门禁) |
| EQ-06 | H-02 改指正典＋H-05 递延（原下沉方案证伪过半） | 同上，两组计数下降 | L1 | EQ-05 | 🟩 (L1×l2 34→27＋lib门绿；playback递延有据) | ⬜ |
| EQ-07 | Facade 类（H-03/04/06）走门面 | 同上，三组计数下降 | L1/L3 | EQ-06 | 🟨 (机械 5 处完，server/hotreload/lead 递延有据) |
| EQ-08 | dispatcher policy 类（H-01，最后） | L1×L2/L5 计数下降 | L1/L5 | EQ-07 | 🟨 (SIM-42：实结构引用，改指/下沉皆证伪；B06 时限 allowlist 至 2026-10-31＋P1-02 构造注入递延) |

## T3. SDB 烘焙设计（与 T2 并行，不早于此）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-09 | dispatcher 三点 verifier 谓词＋阈值＋log-only 方案 | 设计文档＋SDB-REGISTRY 评分列首填 | L5 | SDB-06/07 定级（已有） | 🟩 (SDB v0.6：VP-1~3＋τ=0.65＋log 格式＋首填 1/2/2/1/1；SIM-42) |
| EQ-10 | SDB-06/07 接线对象确定 | 登记表 C 列填实 | L5 | EQ-09 | 🟩 (06→SDB-05 分解步；07→代码修改动作 SDB-08+；C=log-only 结构化日志，outbox 推广 P-task；SIM-42) |

## T4. P1 收尾

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-11 | 其余 4 fitness 阈值拍板 | L5 书面签字（数字＋理由） | L5 | 无（可并行） | 🟨 (提案备齐待签：NoCycle 0 环／Capability 0 重边／TreeSingleton ≤1 实例点／DeadCode 0 警告＋禁 crate 级 allow；PanicDensity 3000·12.0 与 B2 已有数；SIM-42 §41) |
| EQ-12 | pre-commit 增强落地 | doc-drift／confidence 本地可跑 | Infra | EQ-11 | 🟨 (doc-drift advisory 已接（bash -n 过，随本轮提交实跑）；confidence 系 cargo 侧，待全量门；SIM-42) |
| EQ-13 | P0 独占重验（check --tests 全绿） | 门绿＋新基线入库 | QA | 独占窗口 | 🟨 (工单备齐待跑：本机无 gh 且 CI 无 dispatch，触发＝向 main 建 PR；CI 自带 check×3OS＋test＋layer-deps；步骤：建 PR→读全部门→基线入库→关 EQ-05/07/12 尾巴；待执行) |

## T5. P2/P3（条件触发，提前开工即错）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-14 | 覆盖率 70→80 烘焙 | fail-under-lines 提到 80 且门绿 | QA | P2 启动 | ⬜ |
| EQ-15 | bench 基线＋翻转 | gh-pages 基线满 1 周 → fail-on | Perf | P3 启动 | ⬜ |
| EQ-16 | vet／Scorecard／SLSA／dist | 烘焙计划逐项关 | Security | P3 启动 | ⬜ |
| EQ-17 | fuzz／toxics／AgentHarm 首跑 | 首 harness＋首 toxic＋首对子 | SRE/L3 | P3 启动 | ⬜ |
| EQ-18 | Tauri 独立审计 | 审计报告＋问题清单 | Desktop | M2 | 🟨 (范围已定：main 8 行/conf 82 行/capabilities default.json/permissions neotrix_commands.toml/plugins 2＋src ~30（vault/ipc/commands/browser_host/anthropic/db_pool/agent_identity/autostart…）；checklist＝IPC 暴露面/capability 最小权限/vault 密钥/browser_host 外联/错误外泄；待主＋报告) |
| EQ-19 | FULL-ARCHITECTURE 认领 | 有主（人＋日期） | Architect | M2 | 🟨 (scope 已定：801 行 v1.0.0(09-20) vs 蓝图 v1.6.7 差 6 代；提案：Architect 09-30 前认领并三选一 同步/归档/重写；待批) |
| EQ-20 | R-P112–115 重议＋ARCHITECTURE.md 重写 | 会议纪要＋新版／归档 | Architect | NT-STD-1.1 | ⬜ |

---

*维护：状态变更随代码同 commit（改状态不改代码也值得一次小提交）；新增任务号顺序分配（EQ-21 起）。*
