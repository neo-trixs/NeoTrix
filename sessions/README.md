# sessions/ 交接文档索引

> 本文件是**导航层**，不存快照正文。2026-10-06 建：此前 70 份 handoff **无任何索引**，
> 只能靠文件名猜。⛔ 索引里的「状态」列只反映**建表当时**的判断，未核实项已标注。

## 怎么用这个目录

| 我要做的 | 读哪几份 |
|---|---|
| 开新窗口，先对齐现状 | `handoff-20260928-new-window-opening.md` → `handoff-20260928-consolidated.md`（⛔ 后者 §2 勘误表必读） |
| 提交纪律 / 覆盖事故 | `handoff-commit-only-2026-09-29.md` · `handoff-20261003-my-overwrite-incident.md` |
| 能力市场与涌现 | `handoff-2026-10-06-capability-invoke.md`（最新）→ `handoff-2026-10-06-agent-loop-wiring.md` → `handoff-2026-10-06-output-distill.md` |
| 找遗留缺陷 | `OPEN-DEFECTS.md`（本目录，**单一入口**） |
| 抄方法论 | `../docs/architecture/LESSONS-*.md` 按主题挑读，勿只读最新 |

## ⛔ 慎读 / 已被取代

- `handoff-20260928-consolidated.md` 的 **§2 勘误表**：4.1/2.1/2.2/5.2/4.4 的台账前提
  **已被实测证伪**，照原文做会重造已存在的东西。
- `handoff-2026-09-29-*` 系列中的计数与行号多已漂移（指针守恒：⛔ 不用行号锚点）。
- `handoff-loop-20250925.md` / `handoff-loop-20260927.md`：AgentLoop 已被拆解，
  有效部分迁入 neobot 真实执行环（见 10-06 三份），**这两份只作历史取证**。
- `HANDOFF-TEMPLATE.md` 是模板，不是交接记录。

## 全量清单（70 份）


### 窗口协同与纪律（先读这组）（16 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `HANDOFF-TEMPLATE.md` | Handoff 交接模板（复制改名用） | ? | 3K |
| `handoff-20260927-final.md` | 交接 · 卡死/内存专项 + 长尾修复（2026-09-27 收口） | 20260927 | 7K |
| `handoff-20260928-consolidated.md` | Handoff — 剩余任务汇总（供单窗口统一修复，2026-09-28） | 20260928 | 10K |
| `handoff-20260928-new-window-opening.md` | …在这里跑真实命令，最后 git worktree remove --force "$W" | 20260928 | 7K |
| `handoff-20260928-ratchet-final.md` | Handoff — 2026-09-28 分层门棘轮收口（101 → 8） | 20260928 | 3K |
| `handoff-20260929-b2-noise-final.md` | 交接：B-2 Noise IKpsk2 重写收口（2026-09-29） | 20260929 | 20K |
| `handoff-20260930-second-tree-final.md` | Handoff — 第二棵树 B 方案收官（2026-09-30） | 20260930 | 8K |
| `handoff-20260930-silentfix-imstop.md` | Handoff — 静默失败契约收尾 + IM outbox 毒行修复（2026-09-30）  <br/>*静默修复 / IM 停止（质量纪律）* | 20260930 | 7K |
| `handoff-20261003-my-overwrite-incident.md` | 事故记录：批量机械改写 + `git checkout --` 覆盖他窗 WIP（2026-10-03） | 20261003 | 4K |
| `handoff-commit-only-2026-09-29.md` | 共享 index 下的提交纪律：`git commit --only`（2026-09-29 两次实测事故） | 2026-09-29 | 3K |
| `handoff-consolidate-all-windows-20260928.md` | Handoff · 单窗口汇总（2026-09-28 收口）— 请由**单一窗口**统一修复 | 20260928 | 10K |
| `handoff-crystal-consolidate-20260928.md` | Handoff — 晶体核心统一收口（2026-09-28 第 5 次会话） | 20260928 | 10K |
| `handoff-decision-20260927.md` | 需人工决策项 · 技术决策书 (2026-09-27) | 20260927 | 5K |
| `handoff-s-audit0927.md` | Handoff — 结构性审计线（cycle `audit0927` / `audit0927b`） | 2026-09-2x | 6K |
| `handoff-s000.md` | Handoff s000 — main.rs 战斗簇拆分进行中（3 个重复定义待删） | ? | 9K |
| `handoff-to-browser-window.md` | 喊话 browser 窗口：三处调用点缺参，请补齐（owner 明确不让我代修） | ? | 1K |

### 外部吸收（19 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `handoff-2026-10-05-game-source-absorption.md` | 交接 —— Rust 游戏源码吸收窗口（2026-10-05 15:5x ~ 18:52） | 2026-10-05 | 9K |
| `handoff-2026-10-05-miu2d-ra2.md` | handoff — 2026-10-05 — miu2d / rust-alert 吸收轮（确定性纪律 + 涌现指纹） | 2026-10-05 | 11K |
| `handoff-20260929-absorption-round2.md` | Handoff — 第二轮外部吸收 + 幻影门拆除（2026-09-29） | 20260929 | 8K |
| `handoff-EVO04-browse-20260926.md` | EVO-04 高速浏览器环 Spec（抄 browser-use / jev-ultrafast） | 20260926 | 4K |
| `handoff-S39-20260926.md` | §39 交接提示词（2026-09-26，补齐 handoff-generative-20260924 预告） | 20260926 | 2K |
| `handoff-absorption-round24-20260930.md` | handoff — ABSORPTION-ROUND24 / 调用图 G4 + G6 + 两个 P0（一修一待签）（2026-09-30） | 20260930 | 8K |
| `handoff-cleanup-puremac-absorb.md` | Handoff — cleanup-puremac-absorb | ? | 3K |
| `handoff-evo-20260926.md` | NeoTrix 统一进化迭代（2026-09-26，34 源吸收 → 路线 → 版本 → 归档 → 清单） | 20260926 | 16K |
| `handoff-evolution-20260929.md` | 收尾交接 —— 进化实验接线 + L1 备件清理（2026-09-29） | 20260929 | 5K |
| `handoff-generative-20260924.md` | Handoff — 生成式觉醒窗全量交接（2026-09-24 10:50，新对话从此接） | 20260924 | 66K |
| `handoff-generative-awaken-kev-ab.md` | Handoff — generative awaken / kev A/B / verify / arch3 | ? | 9K |
| `handoff-hf-batch-absorb-20260928.md` | Handoff — HF 12 候选数据集吸收（2026-09-28） | 20260928 | 5K |
| `handoff-medical-absorb-20260928.md` | Handoff — 医患对话疾病集吸收进晶体核心（2026-09-28） | 20260928 | 7K |
| `handoff-mimo-rlenv-absorb-20260928.md` | Handoff — MiMo-V2.6-RL-oss 训练环境吸收进晶体核心（2026-09-28） | 20260928 | 8K |
| `handoff-neobot-absorption-20260928.md` | Handoff · NeoBot 侧边栏/IM 吸收轮（单窗口汇总收口） | 20260928 | 12K |
| `handoff-ntbrowse-20260923.md` | Handoff：自研浏览器内核 + Lingee 收割（2026-09-23 早） | 20260923 | 2K |
| `handoff-ntcode-tauri-api.md` | ntcode 桌面 App 构建交接 | ? | 7K |
| `handoff-social-access-20261003.md` | handoff — social_access 缺陷修复与生产接线（2026-10-03）  <br/>*社交平台访问* | 20261003 | 21K |
| `handoff-spire-restore-20260923.md` | 跨会话字条：neotrix-spire 恢复（2026-09-23 夜） | 20260923 | 1K |

### UI / 桌面 / 交互（10 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `handoff-20261001-msg-copy-and-theme-fix.md` | 交接 — neobot-ui 补缺陷窗口（2026-10-01 续） | 20261001 | 12K |
| `handoff-20261001-ui-layout-goal-correction.md` | 交接：NeoBot 交付路径 UI 重构（2026-10-01） | 20261001 | 8K |
| `handoff-dialog-spec-20260927.md` | 对话面修复＋格式 Spec（2026-09-27，D/Z/W 三 lane） | 20260927 | 2K |
| `handoff-fiveentity-20260923.md` | Handoff：五实体蓝图窗（fiveentity，2026-09-23） | 20260923 | 6K |
| `handoff-layout-spec-20260927.md` | 对话流＋多模态最优解排版格式（2026-09-27，调研收敛版） | 20260927 | 4K |
| `handoff-neobot-desktop-20260930.md` | handoff — neobot-desktop 重建会话（2026-09-30） | 20260930 | 6K |
| `handoff-neobot-selfhosted-ui-20261001.md` | Handoff — 商用自持前端（neobot-ui）重构收尾 | 20261001 | 8K |
| `handoff-nt-pet-unwrap-notice.md` | 知会：`nt_pet.rs` 2 处 `.unwrap()` 待修（属另一窗口在途特性，我未改） | ? | 3K |
| `handoff-qwen-mm-20260928.md` | Handoff — Qwen-MM-Plugins 吸收（A+B+C 全做）· 2026-09-28  <br/>*Qwen-MM 视觉能力接入* | 20260928 | 29K |
| `handoff-ui-opt-20260927.md` | 通用 UI 优化建议全集（2026-09-27，调研＋对标＋落地状态） | 20260927 | 1K |

### 能力市场 / 涌现 / 金丝雀（8 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `handoff-2026-10-05-decor-noise-and-canary-rewiring.md` | handoff — 2026-10-05 星号噪声清理 + 金丝雀改接真实派发路径 | 2026-10-05 | 8K |
| `handoff-2026-10-05-tui-wiring-and-six-defects.md` | handoff — TUI 接线 + 六个真缺陷（2026-10-05） | 2026-10-05 | 40K |
| `handoff-2026-10-06-agent-loop-wiring.md` | handoff — 2026-10-06 把 AgentLoop 的独有能力接到真实执行环 | 2026-10-06 | 8K |
| `handoff-2026-10-06-capability-invoke.md` | handoff — 2026-10-06 capability_invoke 接线 + 星号/编码清理 | 2026-10-06 | 6K |
| `handoff-2026-10-06-output-distill.md` | handoff — 2026-10-06 工具输出 errors-first 蒸馏（AgentLoop 最后一项能力） | 2026-10-06 | 7K |
| `handoff-2026-10-06-security-wiring-and-shell-guard.md` | 交接 —— 游戏源码吸收 · 第二批（修「尺子」+ 补安全洞） | 2026-10-06 | 13K |
| `handoff-loop-20250925.md` | Handoff — 采矿循环窗 + 抖音任务（2026-09-25 晚，新对话从此接）  <br/>*AgentLoop 早期轮次（2025-09，已被后续取代）* | 20250925 | 4K |
| `handoff-loop-20260927.md` | Handoff Loop 20260927 — 长循环执行证据（owner 直令循环到全绿）  <br/>*AgentLoop 主线（被 10-06 agent-loop-wiring 取代）* | 20260927 | 25K |

### 目录 / 分层 / 孤儿治理（6 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `handoff-2026-10-05-orphan-governance-and-gate-repair.md` | 交接：孤儿治理收官 + 三道门修复（2026-10-05） | 2026-10-05 | 12K |
| `handoff-20260928-algo-extraction.md` | handoff — 算法萃取 + 目录归档（2026-09-28） | 20260928 | 35K |
| `handoff-20260928-merge-readiness.md` | MERGE-READINESS — `fix/bitemporal-and-layer-ratchet` → 主干 | 20260928 | 3K |
| `handoff-codemap-20260927.md` | NeoTrix 全域 Code Map（2026-09-27，只读侦察＋本窗裁决） | 20260927 | 8K |
| `handoff-dir-arch-20260927.md` | Handoff — 2026-09-27 目录架构统一轮 | 20260927 | 9K |
| `handoff-secondtree-reflow-20260930.md` | handoff — 第二棵树 B 方案 第 2/8、3/8 模块回流（2026-09-30） | 20260930 | 10K |

### 缺陷分诊 / 债务（6 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `handoff-20260929-salvage-archive.md` | 交接件 — 抢救性归档窗口（2026-09-29） | 20260929 | 11K |
| `handoff-cocoons-health-20260928.md` | Handoff — 晶体核心基础健康修复（2026-09-28 第 3 次会话）  <br/>*晶体核心健康：1645 个重号 M-id 是 id 碰撞吃掉不同记忆（非冗余）* | 20260928 | 12K |
| `handoff-disease-list-20260927.md` | 待修复清单 (2026-09-27) — 卡死/内存爆炸专项 | 20260927 | 36K |
| `handoff-global-todo-20260926.md` | NeoTrix 全域待解决任务清单（2026-09-26，只列清单不动） | 20260926 | 12K |
| `handoff-s001.md` | Handoff s001 最终版：吸收＋执行＋M1–M7（2026-09-22 关窗）  <br/>*S001 任务交接* | ? | 1K |
| `handoff-test-debt-20260928.md` | Handoff — 剩余三笔债（含精确诊断，2026-09-28） | 20260928 | 7K |

### 融合 / 一致性 / 分歧（4 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `handoff-20260930-tier1-struct-pilot.md` | handoff — tier1 struct 归并试点（15 个） | 20260930 | 9K |
| `handoff-20261001-unified-reconcile.md` | 统一归并：两份交接的冲突裁决与统一队列（2026-09-30） | 20261001 | 4K |
| `handoff-decompose-parity-20260930.md` | 交接：原子拆解 + 行为对位（建议 2 关闭）— 2026-09-30 | 20260930 | 21K |
| `handoff-type-fusion-20260929.md` | Handoff — 类型自动融合器 nt_fuse_types.py（2026-09-29） | 20260929 | 2K |

### 安全 / 授权 / 凭据 / 沙箱（1 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|
| `handoff-2026-10-05-authorization-audit-and-inert-defenses.md` | Handoff — 授权栈审计与「声称存在但实际不生效」防线（本窗口） | 2026-10-05 | 16K |

### 未归类（0 份）

| 文件 | 标题 | 日期 | 体积 |
|---|---|---|---:|

## 维护纪律

- 新增 handoff 时**必须**在对应主题表里加一行，否则索引立刻失真（这正是它当初缺失的原因）。
- 文件名沿用 `handoff-<日期>-<主题>.md`；日期用 `YYYY-MM-DD`。
- 交接里的**待修缺陷**不要只写在正文里 —— 同步登记到 `OPEN-DEFECTS.md`，
  否则缺陷会随窗口关闭而沉底（`handoff-disease-list-20260927.md` 的教训）。
