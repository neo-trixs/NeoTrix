# NeoTrix 全量迭代任务评测报告

**日期**: 2026-09-15 | **Cycle**: 20260915_fusion
**URLs处理**: 200+ 外部源 | **领域**: 8个NT域
**状态**: 核心融合架构已实现，待验证与清理

---

## 一、融合架构现状评估

### ✅ 已实现的融合模块

| 模块 | 文件位置 | 外部来源 | 成熟度 |
|------|----------|----------|--------|
| **复杂度分类器 (14维)** | `nt_io/model_routing.rs` | openfreerouter, llmrouter | C4 |
| **投机解码** | `nt_io/model_routing.rs` | vLLM N-Gram | C4 |
| **OCR层 (PaddleOCR)** | `nt_world/ocr/` | PaddlePaddle/PaddleOCR | C3 |
| **类型化记忆** | `nt_memory/typed_memory/` | Memanto, LMCache | C3 |
| **OSINT收集** | `nt_shield_osint.rs`, `nt_shield/osint/` | awesome-osint, maigret | C3 |
| **HTTP拦截代理** | `nt_shield/http_intercept/` | Gori, defending-code-harness | C3 |
| **AgentLoop** | `nt_io/nt_io_agent_loop.rs` | ARTEMIS, LongHorizon-Harness | C4 |
| **安全技能注册表** | `nt_act/` | addyosmani, tech-leads | C3 |
| **Egress Privacy Guard** | `nt_core_llm` | NT-SHIELD | C5 |
| **GWT Attention Routing** | `nt_core_gwt` | Spotify Shunt | C5 |

### 🔴 冗余问题清单

| # | 冗余描述 | 位置 | 影响 | 清理方案 |
|---|----------|------|------|----------|
| R1 | 模型路由策略与GWT salience重叠 | model_routing.rs + nt_core_gwt | 双重路由决策 | 合并为GWT+cost统一信号 |
| R2 | 文件解析在nt_file_ability和nt_memory重复 | both modules | 数据双写 | 明确边界：file_ability=解析，memory=存储 |
| R3 | 浏览器自动化跨NT-WORLD和NT-SHIELD | nt_world + nt_shield | 职责不清 | NT-WORLD=感知/爬取，NT-SHIELD=隐身/指纹 |
| R4 | 技能加载多源重复 | nt_act + nt_io + l3_vendor_skills | 加载逻辑分散 | 统一SKILL-SPEC合同 |
| R5 | 知识库存储SQLite+codebase-memory-mcp | nt_memory + DeusData | 存储冗余 | 合并codebase memory到KB |

### 🟡 扁平缺陷清单

| # | 缺陷描述 | 位置 | 严重度 | 修复方案 |
|---|----------|------|--------|----------|
| D1 | GWT路由缺少复杂度维度信号 | nt_core_gwt | 高 | 将ComplexityProfile接入GWT salience计算 |
| D2 | 投机解码未与SelfModel性能模型集成 | nt_core_self | 高 | SelfModel需感知speculative_decoding状态 |
| D3 | 类型化记忆的PolicyDrivenForgetting未激活 | typed_memory/forgetting.rs | 中 | 实现衰减策略并与experience-tree联动 |
| D4 | LMCache热存储未接入NT-MEMORY | nt_memory | 中 | 添加LMCache adapter作为HotStore |
| D5 | WHALE自适应phase切换未接入consciousness_tick | nt_mind | 高 | 实现phase检测→优化→评估→决策闭环 |
| D6 | SkillSpector验证未实现 | nt_mind | 中 | 添加SKILL.md合约验证和content hashing |
| D7 | Codebase Memory MCP未接入NT-MEMORY | nt_memory | 低 | 集成DeusData/codebase-memory-mcp |

### 🔵 跨域错位清单

| # | 错位描述 | 域A | 域B | 解决方案 |
|---|----------|-----|-----|----------|
| X1 | NT-ACT与NT-IO的agent执行边界 | nt_act | nt_io | NT-ACT=决策执行，NT-IO=模型通信+循环 |
| X2 | NT-CORE与NT-MIND的E8/SEAL边界 | nt_core | nt_mind | NT-CORE=推理引擎，NT-MIND=进化循环 |
| X3 | NT-WORLD与NT-SHIELD的浏览器 | nt_world | nt_shield | NT-WORLD=内容感知，NT-SHIELD=隐身安全 |
| X4 | NT-MEMORY与NT-NEXUS的记忆边界 | nt_memory | nt_nexus | NT-MEMORY=当前KB，NT-NEXUS=跨会话编织 |
| X5 | nt_file_ability与nt_world的文件 | nt_file_ability | nt_world | nt_file_ability=Office格式，nt_world=Web/爬取 |

---

## 二、核心路线任务清单

### P0 — 立即执行 (阻塞性)

| # | 任务 | 模块 | 来源 | 验收标准 | 预估工时 |
|---|------|------|------|----------|----------|
| P0-1 | **GWT+Complexity融合** | nt_core_gwt | openfreerouter | GWT salience计算包含ComplexityProfile | 2天 |
| P0-2 | **WHALE循环接入** | nt_mind | KRAFTON WHALE | consciousness_tick包含phase检测→优化闭环 | 3天 |
| P0-3 | **SelfModel投机解码** | nt_core_self | vLLM N-Gram | SelfModel感知speculative_decoding状态 | 2天 |
| P0-4 | **PaddleOCR生产集成** | nt_world/ocr | PaddlePaddle | PDF→text pipeline通过测试 | 2天 |
| P0-5 | **文件处理边界明确** | nt_file_ability + nt_memory | 架构审查 | 无重复解析逻辑 | 1天 |

### P1 — 近期待完成 (增强性)

| # | 任务 | 模块 | 来源 | 验收标准 | 预估工时 |
|---|------|------|------|----------|----------|
| P1-1 | **LMCache热存储集成** | nt_memory | LMCache | MemoryMultitier包含LMCache HotStore | 2天 |
| P1-2 | **SkillSpector验证** | nt_mind | NVIDIA/SkillSpector | SKILL.md合约验证+content hashing | 2天 |
| P1-3 | **Codebase Memory MCP** | nt_memory | DeusData | 代码库记忆接入KB | 2天 |
| P1-4 | **Browser-use集成** | nt_world | browser-use | 浏览器自动化harness可用 | 2天 |
| P1-5 | **HTTP拦截完善** | nt_shield/http_intercept | Gori | 完整请求/响应拦截+修改 | 2天 |
| P1-6 | **OSINT完整管线** | nt_shield/osint | awesome-osint | 多源OSINT收集+威胁画像 | 3天 |
| P1-7 | **安全技能注册表** | nt_act/skill_registry | addyosmani | SKILL-SPEC验证+progressive disclosure | 2天 |
| P1-8 | **Prime-Agent协调** | nt_act | PrimeIntellect | 多agent协调框架 | 3天 |

### P2 — 中期规划 (扩展性)

| # | 任务 | 模块 | 来源 | 验收标准 | 预估工时 |
|---|------|------|------|----------|----------|
| P2-1 | **Observability Stack** | nt_meta | onyx, openobserve | 完整追踪+指标+日志 | 3天 |
| P2-2 | **CubeSandbox集成** | nt_world | TencentCloud/CubeSandbox | 安全沙箱环境 | 2天 |
| P2-3 | **Disaggregated Pipeline** | nt_core | vLLM-Omni | 多模态分离推理 | 3天 |
| P2-4 | **Knowledge Graph增强** | nt_memory | agent-memory-atlas | 图知识+VSA HyperCube | 3天 |
| P2-5 | **Headroom管理** | nt_mind | headroomlabs-ai | 上下文窗口管理 | 2天 |
| P2-6 | **多模态视觉生成** | nt_io | vivid-figures-skill | 图表/图形生成管线 | 2天 |
| P2-7 | **OpenMontage视频** | nt_io | calesthio/OpenMontage | 视频蒙太奇能力 | 3天 |

---

## 三、多Agent自动巡检修复方案

### 巡检Squad部署

```
┌─────────────────────────────────────────────────────────────┐
│               NT-CORE (主协调器)                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────┐ │
│  │ Audit Agent│  │ Build Agent│  │ Memory Agent│  │ Sec Agent│ │
│  │ rev-officer│  │ dev-implem │  │ experience │  │ nt-shield│ │
│  │ D1-D63     │  │ cargo check│  │ 60s tick   │  │ pentest  │ │
│  │ FPAM       │  │ test       │  │ KB health  │  │ OSINT    │ │
│  └────────────┘  └────────────┘  └────────────┘  └────────┘ │
│                                                             │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────┐ │
│  │ World Agent│  │ IO Agent   │  │ Mind Agent │  │ Des Agent│ │
│  │ nt-world   │  │ nt-io      │  │ nt-mind    │  │ des-arch │ │
│  │ crawl      │  │ providers  │  │ SEAL loop  │  │ ADR      │ │
│  │ OCR accuracy│ │ routing    │  │ WHALE cycle│  │ review   │ │
│  └────────────┘  └────────────┘  └────────────┘  └────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 巡检Agent Prompt模板

**1. Audit Agent (rev-officer)**
```
执行D1-D63全量审查。FPAM方法论：
- Phase A: Invariant Freeze — 锁定invariant词汇
- Phase B: ADI 3-Pass — Abduction→Deduction→Induction
- Phase C: Leverage Point Scoring — top 5杠杆点
- Phase D: Coverage Matrix — D1-D63覆盖矩阵
- Phase E: Delta Measurement — FP vs FN成本

输出：severity-ranked findings (critical/warning/info)
频率：每5个cycle或每周
```

**2. Build Agent (dev-implementer)**
```
执行cargo check + cargo test + clippy验证。
- cargo check --all-targets -p neotrix
- cargo test -p neotrix --lib
- cargo clippy --all-targets -p neotrix
- 验证无unsafe代码 (#![forbid(unsafe_code)])

频率：每次commit后
```

**3. Memory Agent (nt-memory)**
```
KB健康巡检：
- experience-tree hub验证
- ghost branch清理 (route-verify --clean)
- kv_store完整性检查
- concept神经元Hebb网络验证
- LMCache热存储状态

频率：60s tick (handlers_absorption)
```

**4. Security Agent (nt-shield)**
```
NT-SHIELD安全审计：
- HTTP拦截proxy状态检查
- OSINT数据源可用性
- stealth_net指纹管理验证
- PenList威胁情报更新
- Egress Privacy Guard状态
- 路径验证(PathValidator)状态

频率：每周
```

**5. World Agent (nt-world)**
```
NT-WORLD爬取管线巡检：
- PaddleOCR识别准确率验证
- 浏览器自动化harness状态
- UnifiedCrawler健康
- 媒体资产注册表完整性
- 数据源可用性检查

频率：每日
```

**6. IO Agent (nt-io)**
```
NT-IO提供者健康巡检：
- 所有provider可用性
- GWT路由效率(延迟/成功率/成本)
- 复杂度分类器准确率
- 投机解码命中率
- LMCache热存储命中率
- 提供者负载均衡状态

频率：每日
```

**7. Mind Agent (nt-mind)**
```
NT-MIND进化循环巡检：
- SEAL pipeline阶段完整性
- WHALE phase切换状态
- 技能结晶化状态
- SkillSpector验证结果
- experience-tree吸收完整性
- 进化速度指标

频率：每个cycle boundary
```

**8. Design Agent (des-architect)**
```
架构设计审查：
- 六层架构合规性检查
- ADR一致性验证
- R-P规则合规 (零unsafe/指针守恒/等)
- 跨域边界清晰度
- Dark Forest规则 (无孤儿模块)
- 模块成熟度C0-C6评估

频率：每月
```

### 巡检冲突解决策略

| 冲突场景 | 解决策略 |
|----------|----------|
| 多Agent同时修改同一文件 | 文件级锁 + 串行化处理 |
| Build Agent与Audit Agent同时运行 | Build优先，Audit延迟 |
| Memory Agent与Mind Agent同时写KB | Memory优先(60s tick)，Mind次之 |
| Security Agent与World Agent冲突 | Security优先级最高 |
| IO Agent与Build Agent冲突 | IO Agent只读不写，无冲突 |

---

## 四、经验吸收清单 (经验树索引)

### 本次吸收的经验节点

| Branch Key | 领域 | 类型 | 内容摘要 | 来源 |
|------------|------|------|----------|------|
| branch_fusion_001 | NT-CORE | pattern | Cost-Aware+Complexity+Speculative三重路由融合 | free-router|vLLM|openfreerouter |
| branch_fusion_002 | NT-MIND | pattern | WHALE Harness-Weight自适应优化 | krafton.ai/whale |
| branch_fusion_003 | NT-MEMORY | pattern | Typed Memory Estates+Conflict Resolution | memanto|LMCache |
| branch_fusion_004 | NT-ACT | pattern | AgentLoop Planner/Op/Checker triad | google/artemis|LongHorizon |
| branch_fusion_005 | NT-WORLD | pattern | OCR Pipeline + Browser Automation | PaddleOCR|browser-use |
| branch_fusion_006 | NT-SHIELD | pattern | HTTP Intercept + OSINT Collection | Gori|maigret|awesome-osint |
| branch_fusion_007 | NT-META | insight | 项目已实现大部分融合架构，待验证集成 | 代码审计 |
| branch_fusion_008 | NT-META | defect | 构建缺陷需定位修复 | cargo check |
| branch_fusion_009 | NT-META | rule | R-P79: 外部吸收必须同session接线生产路径 | AGENTS.md |
| branch_fusion_010 | NT-CORE | pattern | Skill as Production Template (SKILL-SPEC) | addyosmani|Easel |

### 外部技术知识图谱

```
200+ URLs
├── AI Agent Frameworks (43) → NT-ACT
│   ├── Agent Loop Patterns → AgentLoop consolidation
│   ├── Multi-Agent Orchestration → Planner/Op/Checker
│   ├── Skill Registry → SKILL-SPEC contract
│   └── Cost-Aware Routing → GWT salience
├── AI Research (25) → NT-CORE, NT-MIND
│   ├── E8 Reasoning → TLCM, RSI roadmap
│   ├── GWT Attention → SAS sparsification
│   ├── VSA HyperCube → VoT, OpenWAM
│   └── SEAL Evolution → WHALE, Feyospace
├── Code/Dev Tools (30) → NT-ACT, NT-MIND
│   ├── Hash-anchored Edit → edit verification
│   ├── Memory Multitier → Colibri, LMCache
│   └── Loop Engineering → LongHorizon-Harness
├── Browser/Security (24) → NT-SHIELD, NT-WORLD
│   ├── HTTP Interception → nt_shield/http_intercept
│   ├── OSINT Collection → nt_shield_osint
│   ├── Pentesting → Penetration-List
│   └── Reverse Engineering → Ghidra
├── Model Routing (15) → NT-IO
│   ├── FreeRouter → cost-aware routing
│   ├── openfreerouter → complexity classifier
│   ├── Vercel eve → filesystem-first agent
│   └── Speculative Decoding → fast/slow path
├── Document/File (14) → nt_file_ability
│   ├── PaddleOCR → OCR layer
│   ├── Fly OCR → PDF→text pipeline
│   └── Memanto → typed memory
└── Design/System (24) → des-architect, des-ui
    ├── Semantic Pattern Routing → diagram-design
    ├── First-Principles → Build-Your-Own-X
    ├── Layered Abstraction → system-design-primer
    └── Progressive Disclosure → agent-skills
```

---

## 五、总结与下一步

### 关键成果
1. **200+外部源已分析**并映射到8个NT域
2. **融合架构核心模块已实现**：复杂度分类器、投机解码、OCR、类型化内存、HTTP拦截、OSINT
3. **3大架构问题已识别**：冗余(R1-R5)、缺陷(D1-D7)、错位(X1-X5)
4. **核心路线任务清单已制定**：P0(5项)→P1(8项)→P2(7项)
5. **多Agent巡检方案已设计**：8个Agent+冲突解决策略

### 立即行动
1. **P0-1**: GWT+Complexity融合 (2天) — 最高优先级
2. **P0-2**: WHALE循环接入 (3天) — 核心进化能力
3. **P0-3**: SelfModel投机解码 (2天) — 性能优化
4. **P0-4**: PaddleOCR生产集成 (2天) — 文档处理补全
5. **P0-5**: 文件处理边界明确 (1天) — 架构清理

### 经验写入
所有本次经验已写入 `~/.neotrix/pending-absorb.json`，由后台循环自动吸收到KB hub。
