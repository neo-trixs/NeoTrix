# 特性进化列表 — 吸收产出对标 (2026-08-14)

> 基准: 12 唯一源吸收 (`notes/absorption-20260814-unified.md`) + seabatch 57 源 (G1-G30/P0-P2)。
> 现状列 = 代码 grep 接地 (evidence-first); 进化路径 = R-P42 强化既有节点。
> 标记: 🔴 缺失 → 🟡 已有但缺关键机制 → 🟢 已有且成熟。

## A. Provider 可靠性 (OpenRouter 能力栈 / ds4-8gb-cpu / web-search-benchmark)

| 特性 | 外部模式 | NeoTrix 现状 (代码接地) | 进化目标 | Tier |
|------|----------|------------------------|----------|------|
| 模型路由 | 市场智慧路由 + Auto Exacto 每 5min 重评 (吞吐/工具遥测/基准) | 🟡 `nt_io_provider/gateway.rs` 无 route 函数命中 | 路由策略加市场信号 + 周期重评 | T3 |
| 响应缓存 | Response Caching: 相同请求零成本 | 🟡 Obsidian rune 概念, 无请求级缓存实现 | hash 键请求缓存 (provider gateway) | T2 |
| JSON 修复 | Response Healing: 畸形 JSON 缺陷 -80% | 🟡 结构化输出无自动修复 | 输出后畸形 JSON 修复通道 | T2 |
| 委托拓扑 | Subagent (frontier→小模型) / Advisor (便宜→咨询强) | 🟡 `nt_core_gwt` 谐振路由, 无委托双工具 | GWT 加 subagent/advisor 委托工具 | T2/T3 |
| Guardrails | 预算/零保留/注入防御/DLP | 🟡 NT-SHIELD 审计 | provider guardrails 参数集 | T2 |
| 本地推理 | CPU demand-paging GGUF (78GiB 在 7.7GiB RAM) | 🟡 provider 有 ollama 概念 (README), 无 demand-paging | provider 加 demand-paging 本地变体 | T2 |
| 搜索配置 | 搜索预算/引擎/深度/模型三维对照基准 | 🟡 `nt_mind_eval_harness.rs` 有 budget_grid (model×budget 点) | benchmark 加引擎/深度维度 + 预算/成本/速度列 | T2 |

## B. 会话/扩展工程 (pi coding-agent)

| 特性 | 外部模式 | NeoTrix 现状 (代码接地) | 进化目标 | Tier |
|------|----------|------------------------|----------|------|
| session 树 | JSONL id/parentId 原地分支 + /tree 跳转续跑 | 🟡 `nt_io_neocodex.rs` 有 side-chat 分支不污染主上下文 (L1779) | 完整 session 树 (分支/跳转/续跑) + JSONL 持久 | T3 |
| compaction | 有损摘要, 全历史 JSONL 留底可回看 | 🟡 `nt_io_provider/compaction.rs` 存在 | 有损摘要 + 留底可回看双轨 | T2 |
| 包分发 | extensions/skills/prompts/themes 经 npm/git 分发 | 🟡 `nt_mind_skill_engine` skill 打包 | skill/prompt/theme 包分发面 | T2 |
| 项目信任 | trust.json 分层信任模型 | 🟡 `nt_io_agents_md` 上下文加载 | 项目信任决策 + 重启生效 | T2 |

## C. 媒体生产 agent 化 (Insightforge / shuohao-skills)

| 特性 | 外部模式 | NeoTrix 现状 (代码接地) | 进化目标 | Tier |
|------|----------|------------------------|----------|------|
| 视频流水线 | 15-agent 协作: 叙事→角色→分镜→镜头→关键帧→片段→成片 | 🟡 `nt_world_video_pipeline.rs` 有 extraction/transcode/subtitle (帧描述符/phash 去重) | 加 agent 编排链 (脚本→分镜→镜头→成片) + checkpoint 断点续跑 | T3 |
| 文档摄取 | 101 格式/371 语言 | 🟡 `nt_world_absorber` + `nt_world_code_search` (符号索引) | absorber 统一格式摄取; code_search 语言表扩展 | T2/T3 |
| Office 工具 | docx/xlsx/pptx 单二进制读改写 | 🟡 `nt_agent_mcp_tools` | mcp_tools 加 office 工具面 | T2 |
| 制作 skill | 短剧: 角色小传/改编大纲/美术设定/分镜 | 🟡 `nt_mind_skill_engine` (UCN 星辰表) | 多媒体制作 skill 包收编 | T2 |

## D. 意识/符号 (pyncd / Verbalized Sampling)

| 特性 | 外部模式 | NeoTrix 现状 (代码接地) | 进化目标 | Tier |
|------|----------|------------------------|----------|------|
| 符号规范形 | 代数式↔超图规范形 (丢弃括号→canonical 归一) | 🟡 `nt_core_e8` 有 row/column normalize (L1282/1293), 无符号 canonical 化 | e8 加 代数→规范形 归一 | T3 |
| 多样性采样 | Verbalized Sampling: verbalize 候选分布→按分布采样, 创意多样性 +1.6-2.1x | 🟡 `nt_mind_distiller.rs` 无多样性采样 (仅 sample_log 测试) | distiller 加 VS 采样防模式坍缩 | T2 |
| 模型选择证明 | Ori Eval: 自家 prompts + 工具调用检查 + 答案评分 | 🟡 `nt_mind_eval_harness.rs` 有 budget grid | benchmark 加 自家任务评估面 | T2 |

## E. 网络/威胁 (LAN-Orangutan / Conferences)

| 特性 | 外部模式 | NeoTrix 现状 (代码接地) | 进化目标 | Tier |
|------|----------|------------------------|----------|------|
| 设备指纹 | 跨扫描稳定 device label → KB 设备节点 | 🟡 `nt_shield_agentic_scan` | agentic_scan 加持久设备标记 | T2 |
| 威胁 seed | 安全会议 slide 归档 (BlackHat/REcon/Hexacon) | 🟡 `nt_world_osint` | osint 加会议 slide 源 | T1 |

## F. 本批验证通过 (P1-1 循环工程, 已接线)

| 特性 | 来源映射 | 实现 | 测试 |
|------|----------|------|------|
| 独立验证门 G8 | LongHorizon-Harness (seabatch) | `nt_mind_evolution_loop.rs` Auditor (Evidence/Consistency/Governance 三角色) + checkpoint→verify_change→recover | test_auditor_accepts_improvement / rejects_regression_and_recovers / consensus_requires_all_roles / governance_blocks_new_hotspots ✅ |
| 循环就绪评分 G9 | loop-engineering (seabatch) | `nt_mind_background_loop/run.rs` LoopReadyScore (handlers 40/kb 25/no_stall 20/cadence 15) + L1-L3 AutonomyTier | test_loop_ready_score_full_kit / low_is_l1 / no_kb_downgrades_tier / bounds ✅ |
| 机械 denylist G12 | loop-engineering | `nt_mind_background_loop/run.rs` PathDenylist (8 破坏性模式, fail-closed) + 宏 3 臂命名门 | test_denylist_blocks_destructive_patterns / allows_safe_actions / extensible_and_fail_closed ✅ |

## 进化优先级 (本批最高 ROI → 迭代顺序)

1. **Provider 可靠性四件套** (T3, 折叠 P2-11): 市场路由 + Auto Exacto 重评 + Response Caching + Response Healing → `nt_io_provider`。最大外部共鸣 (OpenRouter 已是生产级)。
2. **Video agent 流水线** (T3, 折叠 P2-8): Insightforge 15-agent 链 + checkpoint 续跑 → `nt_world_video_pipeline`。与现有 extraction/transcode/subtitle 无缝衔接。
3. **Session 树工程** (T3, 折叠 P1-4): pi 的 /tree 分支续跑 + compaction 留底 → `nt_io_neocodex`。现有 side-chat 分支 (L1779) 是天然锚点。
4. **VS 多样性采样** (T2, 折叠 P2-11): Verbalized Sampling → `nt_mind_distiller`, 防合成数据/创意生成模式坍缩。
5. **pyncd 符号规范形** (T3, 折叠 P2-9): 代数式↔超图 canonical 化 → `nt_core_e8`, 与 VSA 表示直接互证。