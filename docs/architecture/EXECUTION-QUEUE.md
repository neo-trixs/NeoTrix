# NeoTrix 有序执行清单 (SIM-32)

> **原则**: 按序执行，跳序需 SIM 记录理由。`完成定义`是唯一的 done 标准，感觉不算。
> 状态机：⬜待办 → 🟨进行 → 🟩完成 / 🟥阻塞(注明阻塞物)。更新状态只改本表，不另开文档。

## T0. 收尾确认

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-01 | B1 三单测执行 | `cargo test --lib nt_core_cross_layer` 3/3 绿 | P1 | 无（机器独占窗口） | ⬜ |
| EQ-02 | ADR-0002 关闭 | B1 首绿后，pending-CI 转 closed | P1 | EQ-01 | ⬜ |

## T1. 守卫可信度（先修裁判，再踢比赛）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-03 | 死守卫修退役 SIM＋ADR | Layer/Core 二守卫去向书面化（二选一：重定向＋allowlist／退役） | Architect | 无 | ⬜ |
| EQ-04 | 执行 EQ-03 决议 | 代码落地＋对应单测绿 | L5 | EQ-03 | ⬜ |

## T2. P1-02 越层消减（主战场，197 行／12 组）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-05 | H-08 math 下沉 L0（首血） | `check-layer-deps` L1×L5 组计数下降＋全量门绿 | L1 | EQ-04 | ⬜ |
| EQ-06 | H-02 TaskType＋H-05 playback 下沉 | 同上，两组计数下降 | L1 | EQ-05 | ⬜ |
| EQ-07 | Facade 类（H-03/04/06）走门面 | 同上，三组计数下降 | L1/L3 | EQ-06 | ⬜ |
| EQ-08 | dispatcher policy 类（H-01，最后） | L1×L2/L5 计数下降 | L1/L5 | EQ-07 | ⬜ |

## T3. SDB 烘焙设计（与 T2 并行，不早于此）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-09 | dispatcher 三点 verifier 谓词＋阈值＋log-only 方案 | 设计文档＋SDB-REGISTRY 评分列首填 | L5 | SDB-06/07 定级（已有） | ⬜ |
| EQ-10 | SDB-06/07 接线对象确定 | 登记表 C 列填实 | L5 | EQ-09 | ⬜ |

## T4. P1 收尾

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-11 | 其余 4 fitness 阈值拍板 | L5 书面签字（数字＋理由） | L5 | 无（可并行） | ⬜ |
| EQ-12 | pre-commit 增强落地 | doc-drift／confidence 本地可跑 | Infra | EQ-11 | ⬜ |
| EQ-13 | P0 独占重验（check --tests 全绿） | 门绿＋新基线入库 | QA | 独占窗口 | ⬜ |

## T5. P2/P3（条件触发，提前开工即错）

| ID | 任务 | 完成定义 | Owner | 前置 | 状态 |
|----|------|---------|-------|------|------|
| EQ-14 | 覆盖率 70→80 烘焙 | fail-under-lines 提到 80 且门绿 | QA | P2 启动 | ⬜ |
| EQ-15 | bench 基线＋翻转 | gh-pages 基线满 1 周 → fail-on | Perf | P3 启动 | ⬜ |
| EQ-16 | vet／Scorecard／SLSA／dist | 烘焙计划逐项关 | Security | P3 启动 | ⬜ |
| EQ-17 | fuzz／toxics／AgentHarm 首跑 | 首 harness＋首 toxic＋首对子 | SRE/L3 | P3 启动 | ⬜ |
| EQ-18 | Tauri 独立审计 | 审计报告＋问题清单 | Desktop | M2 | ⬜ |
| EQ-19 | FULL-ARCHITECTURE 认领 | 有主（人＋日期） | Architect | M2 | ⬜ |
| EQ-20 | R-P112–115 重议＋ARCHITECTURE.md 重写 | 会议纪要＋新版／归档 | Architect | NT-STD-1.1 | ⬜ |

---

*维护：状态变更随代码同 commit（改状态不改代码也值得一次小提交）；新增任务号顺序分配（EQ-21 起）。*
