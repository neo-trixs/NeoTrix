# 吸收迭代任务清单（2026-10-08，可直接实施）

> **前置**：本清单的源判定与许可表见 `docs/architecture/ABSORPTION-BATCH-2026-10-08-NEOBOT-34.md`。
> **纪律**：每条任务落地三件套 = 代码改动 + 测试（min 3-5 条/模块）+ `cargo check -p neotrix --lib` 零错误。
> 未通过三件套的条目不得打勾。未接线裁决走 Cycle 1201 三选一（✅ 落地 / 📋 路线图 / ❌ 拒）。
> 硬闸：`code_map` 类改动不碰层级依赖；跨层引用走消费方本层 facade；`rg` 禁用 `-E`。

---

## A. neobot 能力补齐（MIT/Apache 源，可熔炼）

### A1. quota 感知模型路由（freellmapi + magpie）
- **源**：`tashfeenahmed/freellmapi`（MIT，31,810★）router+fallback+加密 key；
  `yetone/magpie`（MIT，6,238★）额度耗尽切账号
- **现状**：`crates/neotrix-neobot/src/nt_store/nt_store_quota.rs` 已有
  `QuotaWindow/QuotaLimit/snapshot_quota_windows/quota_used_vs_limit`；
  `nt_routing.rs` 已有 `RoutingEngine/RouteMode/RouteGroup/RouteMember/build_engine_by_name`
- **改**：在 `nt_routing.rs` 新增 `RouteMode::QuotaAwareFallback` variant（仿
  `RouteMode::parse/as_str`），在 `RoutingEngine::execute` 路径里消费
  `QuotaWindow` 余量——耗尽即切到 members 下一个后端
- **验**：`cargo test -p neotrix-neobot --lib nt_routing` 新增 3 条（额度耗尽切换、
  全部耗尽返回 RetryError、fallback 保留 mode=X）+ metrics 字段

### A2. IM 多渠道适配树（dsh-im）
- **源**：`xmanrui/dsh-im`（MIT，1,719★）飞书/微信/钉钉/企微/QQ/Slack/TG/Discord/WhatsApp
- **现状**：`nt_channel.rs` 已有 `ChannelRegistry/register(Box<dyn ChannelAdapter>)/ChannelHealth`；
  `nt_channel_telegram.rs` 是现成参照
- **改**：新增 `nt_channel_wecom.rs`/`nt_channel_feishu.rs`（先 1 个试点），实现
  `ChannelAdapter`，注册进 `ChannelRegistry`；健康心跳复用 `ChannelHealth`
- **验**：`cargo test -p neotrix-neobot --lib nt_channel` 新增：注册不重复、
  health 失败降级、适配器 round-trip 零 panic

### A3. 本地推理加速档（Edge0 + Atomic-Chat）
- **源**：`Edge0-AI/Edge0`（Apache，3,645★）SSD 流式 MoE 35B@2.5GB、Recover-LoRA、prerouter；
  `AtomicBot-ai/Atomic-Chat`（Apache，1,709★）本地 agent 推理
- **现状**：`crates/neotrix-neobot/src/nt_llama.rs`、`LOCAL-LLAMA-2026-09-28.md`
- **改**：在 `docs/standards` 或 `nt_llama.rs` 的启动参数构造处增加
  `--override-kv tokenizer.ggml.prefill-size`/`--mlock` 之外的 SSD 流式档位
  配置 struct（字段：`ssd_offload: bool`、`router_predict: bool`），默认关闭
- **验**：单元测试覆盖参数构造；端到端不跑（需真实模型文件）

### A4. System-1 快速判定路径（laya）
- **源**：`NandhaKishorM/laya`（Apache，31,520★）非自回归 System-1 决策引擎 typed choice/score
- **现状**：`neotrix-core/src/l0_substrate/nt_judge.rs` 已有
  `JudgeVerdict::{admit,archive,skip}` + `ChunkAdmission::judge`
- **改**：在 `ChunkAdmission` 同级新增 `FastDecision`（typed yes/no/score 三分支），
  不自回归、纯规则打分（沿用 judge 的 max_chars/sensitive_terms 过滤）；
  接口对齐 `JudgeVerdict` 语义
- **验**：`cargo test -p neotrix --lib nt_judge` 新增 ≥3 条；判定收敛且可解释

### A5. HTML→markdown 转换器接入（mdream）
- **源**：`harlan-zw/mdream`（MIT，987★）LLM 优化的流式 html→md
- **现状**：`nt_file_ability` doc-parse 分支（`skills/nt_file_ability/branches/doc-parse`）
- **改**：若现有 converter 慢或产出缺 table/strikethrough，评估在
  `nt_file_ability` 注册 mdream 适配器（走适配器注册表，不内联）
- **验**：对 5 个样例 HTML 跑 golden diff；不回归现有测试

### A6. 代理池健康检查（mubeng）
- **源**：`mubeng/mubeng`（Apache，2,729★）fast proxy checker + IP rotator
- **现状实测**：仓内 `ProxyPool` 仅以 imported asset（`import_proxy_pool` 导入 KB）存在，
  **无活的健康检查/轮换循环消费者**；`proxy_daemon` 只是 env 查询的薄壳。
- **裁决 2026-10-08**：**📋 intake（无消费者）**——与 R-P79 红线一致：
  不为了「吸收」而给不存在的轮换循环加代码；待某组件真正按 pool 轮询时再落实 checker。
- **验**：本条不验代码，只在 §3 code map 任务清单中保持 intake 记录。

### A7. 逆向工程 MCP 能力（morluto/rea）
- **源**：`morluto/rea`（MIT，17,126★）`rea-agents` MCP
- **现状**：`NT-WORLD/retrieve` 分支；`nt_crawl_sources.rs` 有 crawl 队列
- **改**：📋 路线图——评估把 rea 作为 WORLD 的外部 MCP server 接入；
  不新增模块，只在 `skills/` 加一份配置模板
- **验**：设计文档评审 + 1 个 spike（真实二进制跑 1 次 MCP 调用）

### A8. persona/工作区对照（openhanako + row-bot + paseo-bots）
- **源**：`liliMozi/openhanako`（Apache，6,718★）、`siddsachar/row-bot`（Apache，1,572★）、
  `oliexe/paseo-bots`（MIT）
- **现状**：`nt_pet.rs`/`nt_workspace.rs`/`nt_side_chat.rs`
- **改**：对照 row-bot「local-first sovereignty」与 openhanako「Agent=文件夹」两条原则，
  把 neobot 的 agent 持久化结构落成「一个 agent = 一个目录（AGENTS.md+memory+skills）」
- **验**：文档 + 一个 export→import 往返测试

## B. 评测与长程 harness（论文 + Apache 源）

### B1. eval 评测台（harness-evals + HarnessEval-W）
- **源**：`harness/harness-evals`（Apache）、arxiv 2608.16859
- **现状**：`benches/memory_bench.rs` 仅 Criterion 吞吐；评测缺口记录在
  `ABSORPTION-BATCH-18-MEMORY-2026-10-07.md` 第 1 条 intake
- **改**：新增 `benches/agent_eval.rs`，引入 Normalized Score(0-1)+threshold pass/fail
  的极简等价物（不依赖外部 Python 包）：`struct EvalCase { prompt, gold, scorer }`
- **验**：`cargo bench --bench agent_eval -- --test` 冒烟 1 用例

### B2. 长程 loop checkpoint/recover（LongHorizon-Harness + MIRA）
- **源**：`AMAP-ML/LongHorizon-Harness`（MIT）、arxiv 2610.02525（MIRA 外层 meta-reasoner）
- **现状**：`neotrix-core/src/l6_meta/nt_nexus/checkpoint.rs` 已有 checkpoint；
  `nt-io-agent-loop` 的 loop step 可在 `nt_loop_step.rs` 承载
- **改**：在 loop step 边界显式写入 checkpoint（已对半做，补「resume 判据」：
  与 `nt_crawl_sources` 的 `claim_hf_pending_url` resume 同型——pending/completed 位）
- **验**：注入中断后 resume 测试；不重复执行已完成 step（at-least-once）

### B3. 并行调度重探索成本度量（SquidAgent）
- **源**：arxiv 2610.08647
- **现状**：多 agent 并行入口未明；先 `rg "parallel|spawn" neotrix-core/src/l6_meta`
- **改**：给并行 worker 加 `re-exploration tokens` 计量字段（orchestrator 已有上下文，
  worker 重建的子集计为浪费），输出 metric
- **验**：3 条单测：浪费计数、0 浪费路径、序列化稳定

## C. 设计/文档侧（无代码或低代码）

### C1. open-design 设计工作区接入评估
- **源**：`nexu-io/open-design`（Apache，99,918★）DeepSeek Harness 设计插件
- **改**：只写评估文档：neobot 设计能力从哪进（Tauri 侧 vs CLI 侧），
  是否值得作为 P2 能力

### C2. 桌面发布双通道（Captain_Who + Herald-OS）
- **源**：`Tiga001/Captain_Who`（Apache，Rust）、`iamlukethedev/Herald-OS`（MIT）
- **改**：在 `docs/` 写 neobot 桌面发布规格：稳定版/预览版双通道、
  问后再装策略、签名公证步骤（源只借鉴形状，不取代码）

### C3. 能力树 bud/strengthen 同步
- 每完成 A1/A2/A4/B1/B2 任一条，立即在能力注册（`.neotrix/capability_registry.json`
  的 schema 对齐）里 bud 或 strengthen 对应节点，并把 `absorbed_capability`
  四元组补进对应 KB 节点 metadata（用 `update-node-metadata`）

## D. 不吸收（记录即结论）

| 源 | 原因 |
|---|---|
| Ai-jailbreak（无 LICENSE） | 红队集；只取设计 ⇒ 把「无 LICENSE ⇒ 只取设计」写进记录 |
| massCode（AGPL-3.0） | 仅取设计；不取代码 |
| The-Last-Math-Competition（GPL-3.0） | 题集非能力 |
| OpenCE（CC0） | Halo 移植；非核心 |
| v（MIT） | V 语言非能力缺口 |
| airgorah（MIT） | 已有 NT-SHIELD/audit 覆盖 |
| wails（MIT） | Tauri 已覆盖定位 |
| karotte（MIT） | RL 环境缺口 ⇒ 记入 eval 台 B1 的依赖面 |

## E. 验证顺序（每日第一轮）

```sh
sh scripts/ops/nt_mem_gate.sh; echo $?           # 非 0 禁构建
python3 scripts/ops/nt_lock_audit.py neotrix-core/src   # 0 条才继续
cargo check --all-targets -p neotrix              # 零错误门
cargo test -p neotrix --lib <触及模块>            # 改动模块单测
python3 scripts/ops/nt_map_reconcile.py           # 文档断言不死
bash scripts/check-feature-gates.sh --quick       # feature 门控
```
