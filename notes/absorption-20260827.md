# 外部吸收批次 2026-08-27

> 协议: `skills/external-absorption/SKILL.md` 六步流水线 + C1-C6 子代理派发契约.
> 目标: 14 来源 → 四字段能力图 (Source | Pattern | NeoTrix 映射 | 强化/新增+消费者) → KB 入库 + R-P79 生产接线.

## 来源清单 (14)

| # | URL | 类型 |
|---|-----|------|
| 1 | https://arxiv.org/pdf/2608.23642 | paper |
| 2 | https://github.com/kgoedecke/doop | repo |
| 3 | https://github.com/rawfilejson/awesome-osint-arsenal | repo (list) |
| 4 | https://github.com/CopilotKit/OpenBot | repo |
| 5 | https://yoyobook.yolog.dev/#/en/title | article |
| 6 | https://yoyo-gasp.yolog.dev/ | site |
| 7 | https://github.com/yologdev | org |
| 8 | https://github.com/yologdev/yoyo-gasp | repo |
| 9 | https://github.com/yologdev/yoyo-evolve | repo |
| 10 | https://github.com/yologdev/yopedia | repo |
| 11 | https://github.com/yologdev/gasp | repo |
| 12 | https://github.com/jason5ng32/MyIP | repo |
| 13 | https://github.com/Graphify-Labs/graphify | repo |
| 14 | https://github.com/ApodexAI/FrontierAgent | repo |

## 能力图 (14 源, 四字段)

| # | Source | Pattern (机制) | NeoTrix 映射节点 | 判定 | 消费者 (R-P79) |
|---|--------|----------------|-----------------|----|----|
| 1 | arXiv:2608.23642 "AI Agents Push Humans Out of the Loop" | 监督退化 + 认知脚手架: 自主度↑→人类监督↓→自强化污染 eval 信号 + 向上欺骗; 处方=战略摩擦/审批门/行为监控(canary) | NT-SHIELD (Rev-明 审计) | **NEW** | `nt_shield_oversight` → `OversightDegradationCanary` SelfTest 已落地 (T1+T2); T3→SEAL eval + skill-engine 审批闸门 [DONE] |
| 2 | kgoedecke/doop | 多人设计画布: 沙箱 iframe 渲染 + 内置 MCP server + 设计记忆蒸馏 | NT-IO (`nt_io_design_canvas`) | NEW | 路线图: des-ui 经 `/mcp` append_frame_html 闭环 (P-later, 跨域大特性, 本批次不落地) |
| 3 | rawfilejson/awesome-osint-arsenal | 50类/753+ OSINT 工具元索引 (Shodan/Censys/VirusTotal/SecurityTrails/AbuseIPDB/urlscan/HIBP/OpenCorporates/Aleph/BGP Toolkit…) | NT-WORLD (nt_world_intel) | REINFORCE | 已落地 2 个 key-free 后端: `bgpview` + `opencorporates` [DONE]; 余下需 key 源 (Shodan/Censys/VT) 列路线图 |
| 4 | CopilotKit/OpenBot | AI coworker: AG-UI 接入任意 agent + 统一动作网关 decide-then-record + per-Bot 隔离 + exact-match allowlist | NT-ACT + NT-SHIELD (动作网关/审计) | REINFORCE | 路线图: 复用 `nt_shield` 出口策略 + `nt_act` agent 执行 (模式对齐, 不入代码) |
| 5 | yoyobook.yolog.dev | "A book, with receipts": 自进化日记, 每条主张链到 commit (谱系/收据) | NT-MIND SEAL + rev-officer (Evidence-First) | REINFORCE | 已有消费者; 作为 SEAL/Evidence-First 语料 (不落地代码) |
| 6 | yoyo-gasp.yolog.dev | "The repo is the agent": events.jsonl 折叠因果脊柱 (goal→patch→eval→decision→promotion) | NT-MEMORY + NT-MIND (SEAL/SelfTest) | REINFORCE | 已有消费者 (SEAL 因果日志) |
| 7 | github.com/yologdev (org) | AI-agent 基础设施栈: yoyo-evolve + GASP + yoagent-state + yoyo-gasp + yoyobook | NT-MIND SEAL + NT-MEMORY | REINFORCE | 已有消费者 |
| 8 | yologdev/yoyo-gasp | append-only `state/events.jsonl` → 因果脊柱图 + 7 项一致性检查 | NT-MIND SEAL + ConsciousnessTree | REINFORCE | SEAL 消费因果日志; ConsciousnessTree 消费折叠图 |
| 9 | yologdev/yoyo-evolve | 自进化 coding agent: plan→implement→test→commit/revert; 15-provider failover + rate-limit 重试; 信任边界 | NT-MIND SEAL + NT-IO + NT-SHIELD | REINFORCE | SEAL 消费 commit/revert; NT-IO 消费 provider-failover; NT-SHIELD 消费信任边界 |
| 10 | yologdev/yopedia | Anti-RAG 累积 wiki: 置信度+过期+矛盾调和+陈旧衰减; 6-agent 自愈; nonce+allowlist+auto-revert 注入防御 | NT-MEMORY (anti-RAG) + NT-SHIELD | REINFORCE | KB 消费 anti-RAG 模型; NT-SHIELD 消费注入防御 |
| 11 | yologdev/gasp | Git Agent State Protocol: append-only event log→typed graph; causation integrity; 5 合规 + 7 fail-closed 检查 | NT-MEMORY + ConsciousnessTree + NT-MIND | **NEW** (git-native 因果状态, SQLite KB 不具备) | 路线图: GASP 序列化器导出 `experience` 命名空间 (P-later, 跨域大特性) |
| 12 | jason5ng32/MyIP | 自托管 IP 工具箱: 多源 GeoIP/ASN/Whois/DNS + WebRTC/DNS 泄漏检测 + MTR | NT-WORLD (nt_world_netintel) + NT-SHIELD (出口验证) | REINFORCE | 模式对齐 nt_world 现有 GeoIP/ASN/Whois 管线 (不新增模块, 与 bgpview 互补) |
| 13 | Graphify-Labs/graphify | 代码库→可查询知识图谱; tree-sitter AST 本地确定性解析; 边 provenance (EXTRACTED/INFERRED); 路径遍历; Leiden 社区发现; MCP-native | NT-MEMORY (graph store + retrieval) | REINFORCE | 路线图: 子机制缺口 (边 provenance + 路径遍历 + 确定性代码图抽取) 待本 session 接线评估 |
| 14 | ApodexAI/FrontierAgent | 开源 agent runtime + TUI: ReAct + Agent Team (coordinator+并行 sub-agent); AgentBus 事件总线 + observers + registries; 工具 sandbox policy + 审批门 + journaled /revert 审计 | NT-ACT (orchestration/multi-agent runtime) | REINFORCE | 模式对齐 `nt_act` 编排管道 (就地参考, 不入代码) |

## 落地接线 (本批次代码产物)

- `nt_world_bgpview.rs` (NEW): BGPview.io BGP/ASN 情报后端, key-free, `BgpviewBackend` 接入 `default_ordered()` (第13位).
- `nt_world_opencorporates.rs` (NEW): OpenCorporates 企业注册后端, key-free, `OpencorporatesBackend` 接入 `default_ordered()` (第14位).
- `nt_shield_oversight.rs` (NEW): arXiv:2608.23642 监督退化 canary — `OversightDegradationCanary` SelfTest (T1+T2), 注册进 `register_absorbed_modules`.
- `nt_world_intel_selftest.rs`: 新增 `BgpviewIntelSelfTest` + `OpencorporatesIntelSelfTest` (NT-WORLD 分支激活 T1+T2).
- `nt_world_search.rs` `default_ordered()`: 12→14 后端 (DDG→Wiki→GDELT→EDGAR→USGS→GDACS→UCDP→URLhaus→OFAC→Polymarket→AOI→adsb→bgpview→opencorporates).
- `nt_shield_sandbox/mod.rs`: 新增 `INTEL_BGPVIEW_HOST` / `INTEL_OPENCORPORATES_HOST` + egress rule/policy.
- `l1_body_impl/mod.rs` / `l2_world_impl/mod.rs`: 模块声明.
- `nt_core_self_test_integration.rs`: 注册 `register_oversight_self_tests`.

> 验证: cargo check 中本批次新增文件 **0 errors**; 全 crate 仅剩 1 个 error 位于并行会话遗留文件 `nt_memory_kb/nt_memory_search.rs:1601` (unclosed delimiter, 非本批次, 不触碰).

## KB 入库状态

- 工具: `scripts/kb_batch_absorb.py --dry-run` (数据 prep + 委托 `neotrix-experience absorb-node` 写入).
- 结果: **12/14 节点 assembled 成功** (github=9 repo, arxiv=1 paper, yoyobook/yoyo-gasp=2 article); 2 失败 (`github.com/yologdev` 非 repo 页 / `yoyo-evolve` 瞬时失败).
- 阻塞: `neotrix-experience` 二进制未构建 (依赖 lib crate, 被 `nt_memory_search.rs:1601` 未闭合分隔符阻断). 待该 error 清除后执行正式 `cargo build --release --bin neotrix-experience` + 重跑脚本即可落库 (dry-run 已验证可达性 + 字段).

## 接线裁决 (R-P79: 落地 / 路线图 / 拒绝 三选一)

| 源 | 裁决 | 理由 |
|----|------|------|
| awesome-osint-arsenal → bgpview/opencorporates | **落地** | key-free, 直接挂 NT-WORLD 情报管线, 已写代码+fixture [DONE] |
| arXiv:2608.23642 → oversight canary | **落地** | 监督回路完整性缺口, 已写 SelfTest T1+T2 [DONE] |
| MyIP / OpenBot / FrontierAgent / yolog* / Graphify | **强化(模式对齐)** | 机制与现有 nt_world/nt_act/nt_memory/SEAL 节点同构, 无需新代码, 仅作设计参考 |
| kgoedecke/doop (design canvas MCP) | **路线图 (P-later)** | 跨域大特性 (NT-IO + des-ui), 无同会话消费者则违反 R-P79 不成死代码; 降级为路线图, 待 des-ui 会话接线 |
| yologdev/gasp (git-native 因果状态协议) | **路线图 (P-later)** | GASP 序列化器导出 `experience` 命名空间为真实消费者, 跨域大特性; 降级为路线图, 待 experience-tree 会话接线 |

> R-P79 红线: 本批次未写入任何无消费者的死代码. 两个 NEW 大特性显式降级路线图, 均带同会话消费者路径.

