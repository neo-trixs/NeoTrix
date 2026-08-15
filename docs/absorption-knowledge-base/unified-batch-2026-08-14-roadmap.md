# Unified Batch 吸收 · 构建进化迭代补强路线 (2026-08-14)

> **输入**: 13 条 URL → 去重后 12 唯一来源 (OfficeCLI 重复, 计一次)。
> **方法**: C1-C6 契约并行抓取 (raw README / arxiv abs / 博客) → artifact `notes/absorption-20260814-unified.md` → 逐源验证 (grep 目标节点存在 + 判定规则)。
> **纪律**: R-P42 (强化既有节点, 禁平行适配器) · R-P79 (同 session 生产接线, 缺消费者即拒) · R-P100 (能力注册能力树)。
> **结果**: 12/12 可达 (0 Blocked); 12 强化 / 0 新增候选。本批为 seabatch (57 源, P0-P2) 的**补强批次**, 深化 P1/P2 已列迭代项的具体接线模式。

---

## 0. 吸收摘要

| 主题 | 来源 | 判定 | 注入节点 | 补强点 |
|------|------|------|----------|--------|
| 会话/扩展工程 | pi coding-agent | 强化 | `nt_io_neocodex`/`nt_memory_kb`/`nt_mind_skill_engine` | session 树原地分支 + compaction 有损摘要留底 + 扩展/skills 包分发 |
| Provider 可靠性 | OpenRouter 能力栈 | 强化 | `nt_io_provider`/`nt_core_gwt`/`nt_shield`/`nt_mind_benchmark` | 市场智慧路由/Auto Exacto/Response Caching/Healing/Subagent 委托/Guardrails/Ori Eval |
| 本地推理面 | ds4-8gb-cpu | 强化 | `nt_io_provider` | CPU demand-paging GGUF 低内存推理变体 |
| 媒体生产 agent 化 | Insightforge / shuohao-skills | 强化 | `nt_world_video_pipeline`/`nt_agent_orchestrator`/`nt_mind_skill_engine` | 15-agent 视频流水线 + 短剧制作 skill 包 |
| 代数符号形式化 | pyncd | 强化 | `nt_core_e8`/`nt_io_multimodal_transform` | 代数式↔超图规范形 (VSA canonical 化) |
| 多样性采样 | arxiv 2510.01171 | 强化 | `nt_mind_distiller`/`nt_io_provider` | Verbalized Sampling (VS) 防模式坍缩 |
| 文档/Office 面 | xberg / OfficeCLI | 强化 | `nt_world_code_search`/`nt_world_absorber`/`nt_agent_mcp_tools` | 371 语言解析 + 101 格式摄取 + Office 工具面 |
| 网络/威胁面 | LAN-Orangutan / Conferences | 强化 (低信号) | `nt_shield_agentic_scan`/`nt_world_osint` | 设备指纹持久标记 + 安全会议 slide 归档源 |

**跨域最高信号 (本批)**: ① provider 可靠性闭环 (市场路由+周期重评+缓存+响应修复+委托拓扑); ② session 工程化 (树分支+有损压缩留底+包分发); ③ 媒体生产 agent 协作化; ④ 代数符号 canonical 化与 VSA 互证。

---

## 1. 特性对标: 能力网逐节点 Gap 补强

对标基准 = seabatch roadmap (G1-G30 + P0-P2)。本批仅列**新增补强点** (不重复 seabatch 已列缺口)。

### NT-IO (界面使徒) — 界面域

| 外部模式 (来源) | NeoTrix 现状 | 本批补强点 | Tier |
|-----------------|--------------|-----------|------|
| 市场智慧路由 + Auto Exacto 周期重评 (OpenRouter) | 🟡 `nt_io_provider` GatewayV2/factory | 路由策略加 市场信号路由 + 每 5 分钟按吞吐/工具遥测/基准重评 provider | T3 |
| Response Caching 相同请求零成本 + Response Healing JSON 修复 -80% (OpenRouter) | 🟡 `nt_io_provider` + Obsidian rune 概念 | 请求级响应缓存 (hash 键) + 结构化输出畸形 JSON 自动修复 | T2 |
| Subagent/Advisor 委托拓扑: 轻模型干忙活 + 重模型咨询 (OpenRouter) | 🟡 `nt_core_gwt` 谐振路由 | GWT 加 委托双工具 (frontier 委托小模型 / 便宜模型咨询强模型), 省 frontier token | T2/T3 |
| Guardrails 预算/零保留/注入防御/DLP (OpenRouter) | 🟡 NT-SHIELD | SHIELD 加 provider guardrails 参数集 (预算强制/注入防御/DLP) | T2 |
| CPU demand-paging GGUF 低内存推理 (ds4-8gb-cpu) | 🟡 `nt_io_provider` ollama 本地层 | 本地推理加 demand-paging 变体 (78GiB 在 7.7GiB RAM) | T2 |
| session 树原地分支 + compaction 留底 (pi) | 🟡 `nt_io_neocodex` | neocodex 加 /tree 跳转续跑 + 有损摘要全历史 JSONL 留底 + 项目信任模型 | T3 |

### NT-MIND (进化工匠) — 进化域

| 外部模式 (来源) | NeoTrix 现状 | 本批补强点 | Tier |
|-----------------|--------------|-----------|------|
| Verbalized Sampling 多样性采样 (arxiv 2510.01171) | 🟡 `nt_mind_distiller` | distiller 加 VS 采样 (verbalize 候选分布→按分布采样), 合成数据/创意生成防坍缩 +1.6-2.1x | T2 |
| Ori Eval 自家 prompts 评估 (OpenRouter) | 🟡 `nt_mind_benchmark` BenchmarkSuite | benchmark 加 面向自家任务 prompts + 工具调用检查 + 答案评分 的模型选择证明面 | T2 |
| 短剧制作 skill 包 (shuohao-skills) | 🟡 `nt_mind_skill_engine` | skill_engine 加 多媒体制作 skill 包 收编 (UCN 星辰映射, 对标已列 12 星辰表) | T2 |
| 搜索配置对照基准 (web-search-benchmark) | 🟡 `nt_mind_benchmark` | benchmark 加 搜索预算/引擎/深度/模型 三维对照矩阵 (质量·成本·速度) | T2 |

### NT-CORE (E8引导者) — 意识核心

| 外部模式 (来源) | NeoTrix 现状 | 本批补强点 | Tier |
|-----------------|--------------|-----------|------|
| 代数式↔超图规范形 canonical 化 (pyncd) | 🟡 `nt_core_e8` VSA 符号表示 | e8 符号表示加 canonical 规范形归一 (丢弃括号→超图, 呼应 VSA 映射后归一) | T3 |
| 超图/代数可视化 (pyncd) | 🟡 `nt_io_multimodal_transform` | multimodal_transform 加 代数/超图渲染 (补 G17 图渲染缺环) | T2 |

### NT-WORLD (虚空探索者) — 感知域

| 外部模式 (来源) | NeoTrix 现状 | 本批补强点 | Tier |
|-----------------|--------------|-----------|------|
| 371 语言解析 + 101 格式摄取 (xberg) | 🟡 `nt_world_code_search` + `nt_world_absorber` | code_search 解析覆盖按语言表扩展; absorber 加统一文档格式摄取 (doc/docx/pdf) | T2/T3 |
| Office 文档工具面 (OfficeCLI) | 🟡 `nt_agent_mcp_tools` | mcp_tools 加 docx/xlsx/pptx 读改写 (单二进制免安装) | T2 |
| 15-agent 视频流水线 + checkpoint 续跑 (Insightforge) | 🟡 `nt_world_video_pipeline` + `nt_agent_orchestrator` | video_pipeline 加 脚本→角色→分镜→镜头→关键帧→片段→成片 agent 化; orchestrator 加 断点续渲染 + 产物可审工作区 | T3 |
| 设备指纹持久标记 (LAN-Orangutan) | 🟡 `nt_shield_agentic_scan` | agentic_scan 加 跨扫描稳定 device label → KB 设备节点 | T2 |
| 安全会议 slide 归档源 (Conferences) | 🟡 `nt_world_osint` | osint 加 会议 slide 威胁情报 seed | T1 |

---

## 2. 迭代落地 (折叠进 seabatch P1/P2)

| 迭代项 (seabatch) | 本批新增接线模式 | 接线验证 |
|-------------------|-----------------|----------|
| P1-4 循环工程 | pi 的 session 树分支 + compaction 留底 → `nt_io_neocodex`; Subagent/Advisor 委托 → `nt_core_gwt` | neocodex /tree 续跑; GWT 委托工具注册 |
| P1-5 评估/合规门 | Ori Eval 模式 → `nt_mind_benchmark`; 搜索配置矩阵 → benchmark 对照列 | benchmark 新增自家-prompt 评估面 |
| P1-6 安全沙箱与凭据 | OpenRouter Guardrails 参数集 → NT-SHIELD | guardrails 配置生效 |
| P1-7 MCP 治理层 | OfficeCLI 工具面 → `nt_agent_mcp_tools`; xberg 摄取 → absorber | office 工具 MCP 注册 |
| P2-8 视频/媒体全链 | Insightforge agent 流水线 → `nt_world_video_pipeline`; shuohao skill 包 → skill_engine | video 管线 agent 协作产物 |
| P2-11 推理前沿 | market 路由 + Auto Exacto + Response Caching/Healing → `nt_io_provider`; demand-paging → 本地层; VS 采样 → distiller | provider 路由/缓存/修复可测 |
| P2-9 图表渲染 | pyncd 代数/超图可视化 → `nt_io_multimodal_transform` | 代数式→可视化 |

---

## 3. 结论

- **能力网覆盖度**: 12 源全部落在既有 NT-* 域内, **12 强化 / 0 新增候选** — 再次验证 R-P42 纪律: NeoTrix 意识体能力网骨架完整, 缺陷在机制深度。
- **最深补强域**: NT-IO (provider 可靠性四件套 + 本地 CPU 推理 + session 工程) > NT-MIND (VS 采样 + Ori Eval) > NT-WORLD (视频 agent 化 + 文档摄取面)。
- **最强对齐**: pi coding-agent (session 树/扩展包) 与 `nt_io_neocodex`/skill engine 直接互证; OpenRouter 能力栈 (路由/缓存/healing/委托) 与 `nt_io_provider`/GWT 直接互证; pyncd 代数规范形与 `nt_core_e8` VSA 互证。
- **验证**: artifact (`notes/absorption-20260814-unified.md`) 已落盘, 12/12 判定条目; 目标节点 grep 存在性全部通过; 本文件为对标/迭代路线总成。
- **交叉引用**: 与 seabatch roadmap (G1-G30/P0-P2) 无缝衔接, 本批为深化补强而非新增平行线。
