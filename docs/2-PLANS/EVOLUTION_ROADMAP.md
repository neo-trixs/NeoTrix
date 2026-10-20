# NeoTrix Evolution Roadmap — 完整路线图

**Sources**: MAPPA, BES, AdaCoM, agentmemory, Scrapling, V-SPLADE, Fastest-RAG, WeSight, GenAI Book, EFC
**Owner**: 多 session 协同
**核心约束**: 所有进化迭代必须在规则系统下执行

---

## 本次 Session (已执行)

| # | 任务 | 状态 |
|---|---|---|
| ✅ | 预存在测试错误修复 (engine_core 括号 — 1325-1335 孤儿代码) | ✅ 完成 |

---

## 统一重构 Session (另一 session 执行)

| Phase | 任务 | 优先级 |
|---|---|---|
| 0 | git 分支 + 映射清单 + cargo check 基线 | HIGH |
| 1 | NT-CORE 重命名 core/ → nt_core_* | HIGH |
| 2 | NT-MIND 重命名 reasoning_brain/ → l8_autonomic_impl/nt_mind/ | HIGH |
| 3 | NT-MEMORY 重命名 knowledge_base/ → nt_memory_kb/ | HIGH |
| 4 | NT-SHIELD 重命名 security/ → nt_shield/ + CLI sandbox | MEDIUM |
| 5 | NT-ACT 重命名 crypto_agent/earn_agent/file_sync/social_media/spear/neogram/voice | MEDIUM |
| 6 | NT-WORLD 重命名 browser/crawler/world_model/jepa_world_model/sensory + 新增 nt_world_parse | MEDIUM |
| 7 | NT-IO 重命名 cli/server/entry/web_ui/notification + 前端 nt_ui_* | MEDIUM |
| 8 | crates/neotrix-types re-export + Cargo.toml path 更新 | MEDIUM |
| 9 | 测试修复 — #[path] + use 语句 | MEDIUM |
| 10 | 全量 cargo check + cargo test --lib + npm test 验证 | HIGH |

---

## 规则完善 (新 Phase — 重命名后、进化前执行)

**详见 `RULES_SYSTEM.md`**

| Phase | 任务 | 文件 | 阻断点 |
|---|---|---|---|
| R1 | 创建 `ShieldEnforcer` 统一入口 | `cli/shield_enforcer.rs` (新) | 所有命令派发 |
| R2 | 创建 `ProjectLaws` 代码化法则 | `cli/laws.rs` (新) | pre-commit + CI |
| R3 | Wire into CLI dispatch | `types.rs` execute() | 命令执行前 |
| R4 | Wire into file operations | `file_cmds.rs` | 文件写入前 |
| R5 | Wire into SEAL pipeline | `pipeline.rs` | 每 stage 执行前 |
| R6 | Wire into LLM/Provider | provider 调用链 | LLM 调用前 |
| R7 | Wire into MCP tools | `mcp_tools.rs` | 工具调用前 |
| R8 | CI pre-commit hook | 新脚本 | git commit 前 |
| R9 | E2E test: shield 阻断验证 | 新测试 | CI |

**规则**: 未通过的 ProjectLaws + ShieldEnforcer 不允许 P0 任何代码合并。

---

## 其他 Session 任务 (重命名 + 规则完善后执行)

| # | 任务 | 优先级 |
|---|---|---|
| 1 | 新模块测试套件 | MEDIUM |
| 2 | 跨模块集成测试设计 (12维同步) | MEDIUM |
| 3 | 文档同步 (AGENTS.md 11维架构章节) | MEDIUM |
| 4 | 守护进程集成 GeometrySync::cycle() | MEDIUM |
| 5 | clippy 警告清理 | MEDIUM |

---

## P0 进化任务 (规则完善后执行 — 在规则下执行)

### P0-0: nt_world_parse — 统一文档解析网关

**来源**: olmOCR (AI2) + Marker (Datalab) + Docling (IBM) + pdfmux + Unstructured.io + Surya
**缺口**: NeoTrix 无 PDF→结构化数据（Markdown/JSON/Memory Nodes）的文档解析能力，知识库只能从 HTML/ArXiv/GitHub 摄取
**核心理念**: 不自研 OCR/VLM，做 **Router + Audit** — 把现有最佳解析引擎编排成统一管道
**最高优先级**: P0（文档是知识库最大的输入来源缺口）

```
                    ┌─────────────────────────────────────┐
                    │         ParseGateway                 │
                    │  (router + confidence audit + retry) │
                    ├─────────────────────────────────────┤
                    │  1. 按文档类型/页内容路由到最佳后端     │
                    │  2. 4-signal 置信度审计（pdfmux 模式） │
                    │  3. 低置信度自动回退到更强后端           │
                    │  4. 缓存 = (文件hash + 参数) → 结果    │
                    └──────────────────┬──────────────────┘
                                       │
          ┌────────────────────────────┼────────────────────────────┐
          │                            │                            │
          ▼                            ▼                            ▼
 ┌─────────────────┐   ┌────────────────────────┐   ┌──────────────────────┐
 │  Tier 0: Fast   │   │  Tier 1: Hybrid        │   │  Tier 2: VLM/Cloud   │
 │ (0.01s/page)    │   │ (0.1-1s/page, CPU)     │   │ (1-10s/page, GPU)    │
 ├─────────────────┤   ├────────────────────────┤   ├──────────────────────┤
 │ PyMuPDF 直提取   │   │ surya OCR + 布局       │   │ olmOCR VLM (--server) │
 │ 数字 PDF 文本    │   │ + reading order        │   │ Marker Python 子进程  │
 │ pdfminer 回退   │   │ + texify 公式          │   │ Docling VLM (remote)  │
 │ text-only LLM   │   │ + 表格检测             │   │ Gemini/GPT-4V Vision  │
 │ 重排            │   │ + 页眉页脚清洗          │   │ (复用 GatewayV2)      │
 └─────────────────┘   └────────────────────────┘   └──────────────────────┘
```

#### 架构文件布局

```
nt_world_parse/
├── mod.rs                   — 模块注册 + 公开类型
├── doc_parser.rs            — DocParser trait + ParsedDocument
├── parse_gateway.rs         — ParseGateway (router + audit + retry + cache)
├── confidence.rs            — 4-signal 置信度评分 (文本密度/字符分布/结构完整性/OCR 对比)
├── backends/
│   ├── text_only.rs         — Tier 0: 免费 LLM text→Markdown (pollinations/groq)
│   ├── pymupdf.rs           — Tier 0: PyMuPDF 直接文本提取 (纯 Rust, 最快路径)
│   ├── surya.rs             — Tier 1: surya OCR+布局 Python 子进程
│   ├── marker.rs            — Tier 1: Marker 完整管线 (OCR→布局→处理器→LLM)
│   ├── docling.rs           — Tier 1: IBM Docling (MIT, TableFormer, 表精度最高)
│   ├── olmocr.rs            — Tier 2: olmOCR VLM (--server remote 模式)
│   └── vision_api.rs        — Tier 2: Gemini/GPT-4V Vision (复用 GatewayV2)
├── processors/              ← Marker 式模块化处理器
│   ├── table.rs             — 表格检测 + markdown/JSON 格式化
│   ├── math.rs              — 公式检测 + LaTeX 转换 (texify 或 mathpix)
│   ├── cleanup.rs           — 页眉页脚移除 + 阅读顺序恢复
│   └── code.rs              — 代码块检测 + 语法高亮
├── renderers/               ← Docling 式多格式渲染器
│   ├── markdown.rs          — Markdown 渲染
│   ├── json.rs              — JSON 树渲染 (映射到 nt_memory_kb 节点/关系)
│   └── chunks.rs            — RAG 分块渲染 (pdfmux 式 flatten)
├── render/
│   └── pdf.rs               — PDF→image 渲染 (poppler/mutool)
├── ingest.rs                — ingest_pdf() → nt_memory_kb 管道
├── mcp.rs                   — MCP server (pdfmux 式: convert_pdf/analyze_pdf/batch_convert)
└── cli.rs                   — `neotrix doc parse/analyze/batch` CLI
```

#### 任务清单

| # | 任务 | 文件 | 依赖 | 参考实现 |
|---|------|------|------|---------|
| P0-1 | `DocParser` trait + `ParsedDocument` + `PageResult` 类型 | `doc_parser.rs` | — | Docling `DoclingDocument` / pdfmux `PageResult` |
| P0-2 | `ParseGateway` router (best-per-page routing) | `parse_gateway.rs` | P0-1 | pdfmux router (per-page backend selection) |
| P0-3 | 4-signal 置信度评分 (text_density/char_distribution/structure/ocr_probe) | `confidence.rs` | P0-1 | pdfmux confidence scoring |
| P0-4 | Auto-retry: 低置信度页 → 更强后端重提取 | `parse_gateway.rs` | P0-2, P0-3 | pdfmux self-healing pipeline |
| P0-5 | LRU 结果缓存 (key=file_hash+params, 30d TTL, 1GB limit) | `parse_gateway.rs` | — | pdfmux result cache |
| P0-6 | `PyMuPDFBackend` — 纯 Rust PDF 文本提取 (最快路径, 0.01s/page) | `backends/pymupdf.rs` | `lopdf` crate | pdfmux PyMuPDF |
| P0-7 | `TextOnlyBackend` — 免费 LLM text→Markdown (已验证: Pollinations 可行) | `backends/text_only.rs` | GatewayV2 | 本 session curl 验证 |
| P0-8 | `SuryaBackend` — surya OCR+布局 Python 子进程包装 | `backends/surya.rs` | Python + surya | Marker surya wrapper |
| P0-9 | `MarkerBackend` — Marker 完整管线 (OCR→布局→处理器→LLM) | `backends/marker.rs` | Python + marker-pdf | marker CLI |
| P0-10 | `DoclingBackend` — IBM Docling (MIT, 最佳表格精度 97.9% TEDS) | `backends/docling.rs` | Python + docling | docling CLI |
| P0-11 | `OlmocrBackend` — olmOCR VLM remote --server 模式 | `backends/olmocr.rs` | olmOCR server/vLLM | olmOCR `--server` |
| P0-12 | `VisionApiBackend` — Gemini/GPT-4V vision (复用 GatewayV2) | `backends/vision_api.rs` | GatewayV2 | marker `--use_llm` |
| P0-13 | 表格处理器 (block type detect + markdown/JSON/HTML 渲染) | `processors/table.rs` | P0-8/9/10 | Marker TableConverter |
| P0-14 | 公式处理器 (surya→texify LaTeX) | `processors/math.rs` | P0-8 | Marker texify |
| P0-15 | 页眉页脚 + 阅读顺序恢复 | `processors/cleanup.rs` | P0-8 | Marker reading order |
| P0-16 | Markdown 渲染器 | `renderers/markdown.rs` | P0-1 | 所有工具 |
| P0-17 | JSON 树渲染器 (映射到 nt_memory_kb 的 22 节点类型/19 关系) | `renderers/json.rs` | P0-1, KB types | Docling JSON |
| P0-18 | PDF→image 渲染 (poppler/mutool/sips) | `render/pdf.rs` | poppler | 所有工具 |
| P0-19 | `ingest_pdf()` — 解析结果入库 nt_memory_kb | `ingest.rs` | P0-17, KB | — |
| P0-20 | CLI: `neotrix doc parse/analyze/batch` | `cli.rs` | ParseGateway | pdfmux CLI |
| P0-21 | MCP server: `convert_pdf`/`analyze_pdf`/`batch_convert` tool | `mcp.rs` | ParseGateway | pdfmux MCP server |
| P0-22 | 集成测试: (a) PDF→Markdown (b) PDF→JSON→KB (c) MCP 工具调用 | `tests/` | P0-16/17/21 | — |
| P0-23 | 基准测试: 各后端在 olmOCR-Bench 子集上的精度/速度对比 | `benches/` | 全部后端 | olmOCR-Bench |

**关键设计决策**:

| 决策 | 选择 | 理由 |
|------|------|------|
| router, not extractor | **Route** 模式 (pdfmux) | 不自研 OCR/VLM，做编排。每页路由到最强后端。比单一后端更灵活 |
| 后端调用方式 | **子进程 Python** (surya/marker/docling) | 成熟生态在 Python，Rust 重写成本极高。子进程隔离好、易升级 |
| VLM 调用 | **复用 GatewayV2** (olmocr/vision_api) | 已有断路器+回退+限流，零额外工作 |
| 置信度 | **4-signal 评分** (pdfmux) | 文本密度+字符分布+结构完整性+OCR 探测 → 0-1 分数。可做质量门控 |
| 输出格式 | **DoclingDocument 式 JSON 树** | 保留层次结构（Page→Section→Table/Text/Figure→Line/Span），直接映射 nt_memory_kb |
| MCP 协议 | **原生 MCP server** | NeoTrix 已有 MCP 注册中心，pdfmux 验证了模式可行 |
| LLM after processor | Marker `--use_llm` 模式 | 先启发式处理 95%，LLM 只精修表格/公式/复杂布局。兼顾成本与质量 |
| PDF 渲染 | **sips (macOS) + poppler (Linux) + mutool (cross)** | 多平台后备，不依赖单一渲染引擎 |

#### 学习矩阵 (从各项目吸收的设计模式)

| 项目 | 吸收的模式 | nt_world_parse 对应 |
|------|-----------|-------------------|
| **pdfmux** | 每页路由 + 4-signal 置信度 + 自愈 + MCP server | ParseGateway router + confidence.rs + auto-retry + mcp.rs |
| **Marker** | 混合管线 + 模块化 processors + `--use_llm` | processors/ (table/math/cleanup/code) |
| **Docling** | 统一 DoclingDocument + 多格式导出 + MIT 协议 | renderers/json.rs (JSON 树→KB 节点) |
| **olmOCR** | VLM + GRPO RL + Unit Test Rewards + `--server` 模式 | backends/olmocr.rs (remote VLM) |
| **Unstructured.io** | 策略选择 (AUTO/FAST/HI_RES/OCR_ONLY) + 回退链 | ParseGateway 路由策略枚举 |
| **Surya** | 单 VLM 做 OCR+布局+表格+阅读顺序 (650M, 83% olmOCR-bench) | backends/surya.rs (Python 子进程) |

### P0-A: MAPPA — Per-Action Process Rewards

| # | 任务 | 文件 | 检查点 |
|---|---|---|---|
| A1 | `CoachScore` struct | `coach.rs` | L009 (浮点约束) |
| A2 | `AICoach` trait | `coach.rs` | L001+L002 (命名+注册) |
| A3 | `LlmCoach` impl | `coach.rs` | R6 (LLM 调用走 Shield) |
| A4 | `NullCoach` impl | `coach.rs` | — |
| A5 | 注册 coach 模块 | `mod.rs` | L002 (必须注册) |
| A6 | `coach_scores` → `StageResult` | `pipeline.rs` | — |
| A7 | `coach` → `SelfIteratingBrain` | `loop_impl/core.rs` | — |
| A8 | `with_coach()` builder | `loop_impl/core.rs` | — |
| A9 | `BrainStage::process` 签名变更 | `pipeline.rs` trait | — |
| A10 | 更新 27 个 stage | 各 stage 文件 | — |
| A11 | pipeline.execute() 传递 coach | `pipeline.rs` | R5 (SEAL 走 Shield) |
| A12 | 每 stage 后 coach.score() | `pipeline.rs` | R6 (coach LLM 调用走 Shield) |
| A13 | REINFORCE++ 梯度 | `pipeline.rs` | — |
| A14 | baseline EMA | `pipeline.rs` | L009 (浮点约束) |
| A15 | E8 用 avg_coach_score | `e8_experiment.rs` | — |
| A16 | --coach-model CLI | CLI config | R3 (CLI 命令走 Shield) |
| A17-A19 | 3 个测试 | 对应文件 tests | R9 (Shield E2E) |

### P0-B: BES — Bidirectional Evolutionary Search

| # | 任务 | 文件 | 检查点 |
|---|---|---|---|
| B1 | `E8Trajectory` struct | `e8_reasoning.rs` | L001 (nt_ 前缀) |
| B2 | `crossover()` | `e8_reasoning.rs` | — |
| B3 | `mutate()` | `e8_reasoning.rs` | — |
| B4 | `population` + `generation` | `e8_experiment.rs` | — |
| B5 | `evolution_step()` | `e8_experiment.rs` | — |
| B6 | `backward_decompose()` | `e8_experiment.rs` | R6 (LLM 调用走 Shield) |
| B7 | `verify_subgoals()` | `e8_experiment.rs` | — |
| B8 | `core_review()` subgoal | `engine_core.rs` | R6 |
| B9 | BES 替代 best-of-N | `e8_experiment.rs` | — |
| B10-B13 | 3 测试 + 1 bench | 对应文件 | R9 |

---

## P1-P3 进化任务 (P0 完成后执行)

| Phase | 组 | 条目 | 前置 |
|---|---|---|---|
| P1-A | AdaCoM 上下文管理 | C1-C9 | P0 + R1-R9 |
| P1-B | agentmemory KB 后端 | D1-D6 | P0 + R1-R9 |
| P2-A | Scrapling 反机器人 | E1-E7 | P0 + R1-R9 |
| P2-B | V-SPLADE 稀疏检索 | F1-F4 | P0 + R1-R9 |
| P3 | 桌面 UI + GenAI 数学 | G1-G6 | P0 + R1-R9 |

---

## 执行时间线

```
重命名 (Phase 0-10)
  │
  ├──→ 规则完善 (Phase R1-R9) — 创建 ShieldEnforcer + ProjectLaws
  │       │
  │       ├──→ nt_world_parse P0-0 (文档解析网关)
  │       │      │── P0-1~P0-5: 核心框架 (DocParser + ParseGateway + confidence)
  │       │      │── P0-6~P0-12: 后端集成 (PyMuPDF / Marker / Docling / olmOCR / VLM)
  │       │      │── P0-13~P0-15: 处理器 (table / math / cleanup)
  │       │      │── P0-16~P0-18: 渲染器 (markdown / JSON / PDF→image)
  │       │      │── P0-19~P0-21: 管道 (KB ingest / CLI / MCP)
  │       │      └── P0-22~P0-23: 测试 + 基准
  │       │
  │       ├──→ P0 进化: MAPPA (A1-A19) + BES (B1-B13)
  │       │
  │       └──→ P1-P3 进化
  │
  └── 每个阶段都经过 ShieldEnforcer 检查
```

**核心**: 在执行 `types.rs:120 execute()` 之前，先过 `ShieldEnforcer` 链。任何违反 ProjectLaws 的操作被阻断，报 `ExitCode::PermissionDenied(3)`。
