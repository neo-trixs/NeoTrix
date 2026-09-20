# NeoTrix 设计能力架构融合分析

> 日期: 2026-09-20
> 输入源: 30+ 外部仓库/论文/工具 + NeoTrix 6层架构逆向推理
> 方法: 聚焦冗余 + 扁平缺陷 + 跨域错位 → 冗余清理 + 架构重构

---

## 一、外部技术模式提取 (30 sources → 42 patterns)

### A. 设计/视觉能力模式

| # | 来源 | 模式 | 可融合层级 |
|---|------|------|-----------|
| P01 | Cursor Icons | 光学尺寸分离 (16px/24px 独立网格) | L1_Action (media) |
| P02 | Cursor Icons | 构造法: 0°/90°/45° + 圆角, 无自由曲线 | L2_Perception (sense) |
| P03 | Cursor Icons | `ship it` 一键管线: SVG→码位→字体→站点→提交 | L1_Action (pipeline) |
| P04 | qiaomu-icon-gen | Contact sheet + 32px 可读性检查 + 用户选择 | L1_Action (media) |
| P05 | ip-as-logo | 3语义色约束, 从不调用输出为"logo" | L2_Perception (sense) |
| P06 | ui-ux-pro-max | BM25 排名引擎匹配 style/palette/font | L5_Cognition (reasoning) |
| P07 | ui-ux-pro-max | 192条行业推理规则 + 反模式目录 | L5_Cognition (judgment) |
| P08 | beautifului.dev | 21个 AI-native 组件原语 (thinking/approval/chips) | L1_Action (UI) |
| P09 | visual-taste-lab | VI-First Guardrail + 原型验证 | L5_Cognition (reasoning) |
| P10 | decantr-design | 3层设计上下文 + DNA Guards | L6_Meta (governance) |

### B. Agent/推理架构模式

| # | 来源 | 模式 | 可融合层级 |
|---|------|------|-----------|
| P11 | hermes-jev | 决策模型分离: Jev做分类/routing, LLM做生成 | L5_Cognition (model_routing) |
| P12 | hermes-jev | Fail-open: 每个组件有精确回退等价物 | L3_Embodiment (guard) |
| P13 | hermes-jev | Shadow mode: 先决定+日志, 不行动 | L6_Meta (self_review) |
| P14 | hermes-jev | Batched ranking: 120 skill/批, 960=120延迟 | L1_Action (skill_engine) |
| P15 | hermes-jev | Risk word floor: 生产/删除/迁移强制 medium | L3_Embodiment (shield) |
| P16 | HarnessDev | LLM构建+迭代执行基础设施 | L6_Meta (evolution) |
| P17 | Dream-RSI | 发现历史→回放模拟器→策略优化 | L6_Meta (evolution) |
| P18 | AFT | Tool hoisting: 替换宿主内置工具 | L1_Action (agent_loop) |
| P19 | AFT | Symbol-aware: outline/zoom/edit by symbol | L2_Perception (code_search) |
| P20 | AFT | Multi-tier output compression | L1_Action (IO) |

### C. 知识/记忆架构模式

| # | 来源 | 模式 | 可融合层级 |
|---|------|------|-----------|
| P21 | Graphify | Local AST→知识图谱, 无LLM/embedding | L2_Perception (knowledge) |
| P22 | Graphify | Edge confidence: EXTRACTED vs INFERRED | L4_Memory (KB) |
| P23 | Atlas | Session→commit linkage with patch-id | L4_Memory (historian) |
| P24 | Atlas | Cross-agent shared semantic index | L5_Cognition (federation) |
| P25 | Panel | Module Protocol: typed I/O contracts | L1_Action (pipeline) |
| P26 | CobbleDB | 三组件解耦: Pillar/Lorry/CobbleDB | L1_Action (infra) |

### D. 浏览器/爬虫/安全模式

| # | 来源 | 模式 | 可融合层级 |
|---|------|------|-----------|
| P27 | Scrapling | Adaptive element tracking (重设计后自动定位) | L2_Perception (web) |
| P28 | Scrapling | Multi-session spider routing | L1_Action (IO) |
| P29 | patchright | Protocol-level stealth (非JS注入) | L3_Embodiment (shield) |
| P30 | Agent-Reach | Capability layer: ordered backend fallback | L1_Action (gateway) |
| P31 | Agent-Reach | Real probe health checks | L6_Meta (health) |

### E. 生产力/工作流模式

| # | 来源 | 模式 | 可融合层级 |
|---|------|------|-----------|
| P32 | caveman | Type-aware payload compression | L1_Action (IO) |
| P33 | Plannotator | Hook-based agent integration | L1_Action (agent_loop) |
| P34 | My Brain Full | Dispatcher: skill-first routing | L1_Action (skill_engine) |
| P35 | lieflat-less-ai | Corpus linguistics→反AI腔调 | L4_Feel (writing) |
| P36 | Hono | Web Standard portability | L1_Action (infra) |
| P37 | macIconChanger | Dual GUI/CLI + auto-restore | L1_Action (media) |
| P38 | make-x-great | Community governance + HMAC匿名 | L6_Meta (governance) |
| P39 | video_vip | Multi-fallback API aggregation | L1_Action (IO) |
| P40 | 3d-vibe | Prompt→GLB pipeline | L1_Action (media) |
| P41 | tailcat | Userspace WireGuard (零配置) | L3_Embodiment (network) |
| P42 | microduck | IMU-as-bus-slave 同步传感 | L2_Perception (sense) |

---

## 二、NeoTrix 现有能力架构诊断

### 2.1 聚焦冗余 (Redundancy Cluster)

**簇 R1: 设计技能碎片化** — 5个design skill存在功能重叠

```
ui-skill (5 modes) ──┐
agentic-design-system ─┤→ 重叠: 都有"review/audit"能力
ui-reasoning ─────────┤→ 重叠: 都有"anti-pattern detection"
visual-taste-lab ─────┘→ 重叠: 都有"VI-First"原则
```

- `ui-skill/audit` 与 `agentic-design-system/design-review` 覆盖 70% 相同检查项
- `ui-reasoning` 的 10 个维度与 `ui-skill` 的 5 个 mode 高度耦合
- `visual-taste-lab` 的 VI-First 与 `ui-reasoning` 的 VI-First 维度重复定义

**簇 R2: 文件能力过载** — `nt_file_ability` 注册了 22 个能力节点

- `xlsx_read` vs `xlsx_read_fast` vs `parse_xlsx` → 三种 XLSX 读取路径
- `pdf_edit` + `pdf_icon_enhance` + `pdf_image_extract` + `pdf_merge` → 4个PDF子能力散落
- `doc_parse` (anyparse) 与 `merge_docx` 功能边界模糊
- `format_route` 与 `doc_parse` 路由逻辑重叠

**簇 R3: 记忆/知识系统膨胀** — `nt_memory_kb/` 有 88 个模块

- BM25 + embeddings + graph RAG + adaptive RAG + distillation → 5种检索范式共存
- memory palace + cognitive graph + knowledge storage + dual brain → 4种知识表示
- 无明确的检索策略选择机制(哪个场景用哪个范式)

**簇 R4: Agent-Reach vs neotrix-gateway**

- Agent-Reach 的 ordered backend fallback 与 neotrix-gateway 的 UnifiedModelPool 功能重叠
- 两者都处理: 能力发现、健康检查、后端切换

### 2.2 扁平缺陷 (Flat Defects)

**簇 F1: Design 能力停留在 C2 (bud)** — 5 个 design skill 均无 Rust 原生实现

- `ui-skill` 纯 markdown, 无 `self_test: true` 能力节点
- `agentic-design-system` 无对应 `nt_*` 模块
- `visual-taste-lab` 无向量嵌入或 BM25 检索支持
- 所有设计决策依赖 LLM 上下文推理, 无本地缓存/索引

**簇 F2: Media 处理能力碎片化**

- `nt_media/` 有 13 个模块 (audio, HLS, video audit, VTuber, thumbnail)
- `nt_file_ability` 有 `image_super_resolution` (12模型)
- `nt_file_ability` 有 `pdf_icon_enhance`, `pdf_image_extract`
- 无统一的媒体管线抽象, 每种格式独立处理

**簇 F3: Web 感知缺乏自适应**

- `nt_web_perception/` 存在但无 adaptive element tracking (Scrapling 模式)
- 无 protocol-level stealth (patchright 模式)
- 无 multi-session routing 能力

**簇 F4: 决策路由未分离**

- NeoTrix 的 model routing 在 L5_Cognition 内部
- hermes-jev 证明: 分离决策模型可将路由延迟从 LLM 推理时间降至 ~0.4s
- 当前所有路由决策都走完整 LLM 推理链路

### 2.3 跨域错位 (Cross-Domain Misalignment)

**簇 M1: 视觉理解路由错位在 L1**

- `nt_file_ability::visual::route_prompt` 在 L1_Action
- 但视觉理解本质是 L2_Perception (感知层) 的能力
- 导致 L2 的 `nt_core_sense/` 无法直接调用视觉路由

**簇 M2: 图标/Favicon 能力散落**

- `nt_file_ability::pdf_icon_enhance` 在 L1 (文件能力域)
- `qiaomu-icon-gen` 模式应属于 L1_Action (media) 或 L2_Perception
- 但当前与 PDF 处理耦合, 无法独立服务于图标生成场景

**簇 M3: 写作风格调优错位在 L4_Feel**

- `lieflat-less-ai-tone` 语料库方法论适合 L5_Cognition (reasoning)
- 但写作能力在 L4_Feel (emotion/feel) 域
- 导致 L5 的 `ui-reasoning` 无法直接引用写作反模式

**簇 M4: 安全/stealth 能力未桥接到 web 感知**

- `nt_shield/` 有 39 个安全模块
- 但 `nt_web_perception/` 无 stealth 能力引用
- Scrapling/patchright 的反检测模式无法流入 web 感知管线

---

## 三、架构融合方案

### 3.1 冗余清理 (Redundancy Elimination)

#### R1 合并: 设计技能统一为 3 层

```
Before (5 skills, 70% overlap):
  ui-skill, agentic-design-system, ui-reasoning, visual-taste-lab, decantr-design

After (3 skills, clear separation):
  design-core          ← 合并 ui-skill/build + agentic-design-system/build
  design-audit         ← 合并 ui-skill/audit + agentic-design-system/review + ui-reasoning/anti-pattern
  design-identity      ← 合并 visual-taste-lab + decantr-design + ui-reasoning/VI-First
```

- `design-core`: 组件实现 (architect + build + theme + motion)
- `design-audit`: 质量审查 (3-pass 评估 + 10维度推理 + 7类别评分)
- `design-identity`: 品牌/身份 (VI-First + 设计语言 + 原型验证)

#### R2 合并: XLSX 读取三路径统一

```
Before: xlsx_read (calamine) + xlsx_read_fast (zip+quick_xml) + parse_xlsx (unified)
After:  parse_xlsx (unified entry) → auto-route to calamine or fast based on file size
```

#### R3 合并: PDF 子能力聚合

```
Before: pdf_edit + pdf_icon_enhance + pdf_image_extract + pdf_merge (4 个散落节点)
After:  pdf_pipeline (L1_Composite) → 统一 PDF 处理管线
        提供: edit, enhance, extract, merge
```

#### R4 合并: 检索范式选择器

```
Before: BM25 + embeddings + graph RAG + adaptive RAG + distillation (5 种并列)
After:  nt_memory_kb/router (L2_Orchestrator) → 根据查询特征自动选择检索策略
```

### 3.2 扁平缺陷修复 (Flat Defect Elevation)

#### F1: Design 能力 Rust 化

新增 `nt_design/` 模块 (L1_Action):

```
nt_design/
├── nt_design_token.rs       ← 设计 token 系统 (颜色/字体/间距/圆角)
├── nt_design_bm25.rs        ← BM25 排名引擎 (from ui-ux-pro-max P06)
├── nt_design_rules.rs       ← 192条行业推理规则索引 (P07)
├── nt_design_anti_pattern.rs← 反模式检测器
├── nt_design_icon.rs        ← 图标生成管线 (P01-P04 融合)
│   ├── optical_grid()       ← 光学网格系统
│   ├── contact_sheet()      ← 多候选生成
│   └── readability_check()  ← 32px 可读性验证
├── nt_design_brand.rs       ← 品牌一致性检查 (P09)
└── nt_design_lib.rs         ← 设计资产库索引 (P08 beautifului 组件)
```

#### F2: 媒体管线统一

新增 `nt_media_pipeline/` (L1_Action):

```
nt_media_pipeline/
├── nt_media_route.rs        ← 格式路由 (audio/video/image/3d)
├── nt_media_icon.rs         ← 图标/Favicon 统一处理
├── nt_media_3d.rs           ← Prompt→GLB 管线 (P40)
└── nt_media_thumbnail.rs    ← 缩略图生成 (合并现有 nt_media/thumbnail)
```

#### F3: Web 感知自适应化

增强 `nt_web_perception/`:

```
nt_web_perception/
├── existing modules...
├── nt_web_adaptive.rs       ← Adaptive element tracking (P27)
├── nt_web_stealth.rs        ← Protocol-level stealth (P29)
└── nt_web_session_route.rs  ← Multi-session routing (P28)
```

#### F4: 决策路由分离

新增 `nt_decision_router/` (L5_Cognition):

```
nt_decision_router/
├── nt_decision_model.rs     ← 决策模型抽象 (P11: Jev pattern)
├── nt_decision_shadow.rs    ← Shadow mode (P13)
├── nt_decision_risk.rs      ← Risk word floor (P15)
└── nt_decision_batch.rs     ← Batched ranking (P14)
```

### 3.3 跨域错位修复

#### M1: 视觉路由下沉到 L2

```
Before: nt_file_ability::visual::route_prompt (L1_Action)
After:  nt_core_sense::nt_visual_route (L2_Perception)
        nt_file_ability 保留调用接口, 实现移入 L2
```

#### M2: 图标能力独立化

```
Before: nt_file_ability::pdf_icon_enhance (PDF耦合)
After:  nt_design::nt_design_icon (独立图标管线)
        pdf_icon_enhance 改为调用 nt_design_icon
```

#### M3: 写作反模式提升到 L5

```
Before: lieflat-less-ai-tone (L4_Feel 写作域)
After:  nt_design_rules.rs 中增加 "ai_tone_detection" 规则集
        L5_Cognition 的 ui-reasoning 可直接引用
```

#### M4: Shield→Web 感知桥接

```
新增: nt_shield::nt_stealth_bridge.rs
     导出: StealthConfig, ProbeResult, FingerprintProfile
nt_web_perception::nt_web_stealth 引用 nt_stealth_bridge
```

---

## 四、核心能力路线任务清单

### Phase 1: 冗余清理 (Week 1-2)

| # | 任务 | 优先级 | 影响范围 | 预估工时 |
|---|------|--------|---------|---------|
| T1.1 | 合并 design skill: 创建 design-core/design-audit/design-identity | HIGH | skills/design/ | 4h |
| T1.2 | 统一 XLSX 读取入口: parse_xlsx auto-route | HIGH | nt_file_ability | 3h |
| T1.3 | 聚合 PDF 子能力: pdf_pipeline 统一入口 | MEDIUM | nt_file_ability | 4h |
| T1.4 | 检索范式选择器: nt_memory_kb/router | MEDIUM | nt_memory_kb | 6h |
| T1.5 | 更新 skills/index.json 合并后的技能注册 | HIGH | skills/ | 1h |

### Phase 2: 扁平缺陷修复 (Week 2-4)

| # | 任务 | 优先级 | 影响范围 | 预估工时 |
|---|------|--------|---------|---------|
| T2.1 | 新增 nt_design/ 模块: token + bm25 + rules + icon | HIGH | neotrix-core L1 | 16h |
| T2.2 | 新增 nt_media_pipeline/ 统一媒体管线 | MEDIUM | neotrix-core L1 | 8h |
| T2.3 | 增强 nt_web_perception: adaptive + stealth | MEDIUM | neotrix-core L2 | 10h |
| T2.4 | 新增 nt_decision_router/ 决策路由分离 | HIGH | neotrix-core L5 | 8h |
| T2.5 | 所有新模块添加 self_test + capability_registry 注册 | HIGH | 全局 | 4h |

### Phase 3: 跨域错位修复 (Week 3-5)

| # | 任务 | 优先级 | 影响范围 | 预估工时 |
|---|------|--------|---------|---------|
| T3.1 | 视觉路由下沉 L1→L2: nt_visual_route | MEDIUM | L1→L2 迁移 | 4h |
| T3.2 | 图标能力独立: pdf_icon_enhance → nt_design_icon | MEDIUM | L1 重构 | 3h |
| T3.3 | 写作反模式提升: lieflat→nt_design_rules | LOW | L4→L5 桥接 | 2h |
| T3.4 | Shield→Web 桥接: nt_stealth_bridge | MEDIUM | L3→L2 桥接 | 4h |
| T3.5 | 更新 6-layer 依赖图文档 | LOW | docs/ | 2h |

### Phase 4: 外部模式融合 (Week 4-6)

| # | 任务 | 优先级 | 影响范围 | 预估工时 |
|---|------|--------|---------|---------|
| T4.1 | 融合 beautifului.dev 21组件原语到 design_lib | MEDIUM | nt_design | 6h |
| T4.2 | 融合 hermes-jev shadow mode 到 decision_router | MEDIUM | nt_decision_router | 4h |
| T4.3 | 融合 Graphify AST→图谱到 code_search | LOW | nt_core_code_search | 8h |
| T4.4 | 融合 Atlas session→commit linkage 到 historian | LOW | nt_memory_historian | 6h |
| T4.5 | 融合 Scrapling adaptive tracking 到 web_perception | MEDIUM | nt_web_perception | 6h |

### Phase 5: 多Agent自动巡检修复 (持续)

| # | 任务 | 优先级 | 影响范围 | 预估工时 |
|---|------|--------|---------|---------|
| T5.1 | 编写 auto-patrol skill: 依赖图循环检测 | HIGH | skills/ | 4h |
| T5.2 | 编写 auto-patrol skill: 能力节点孤岛检测 | HIGH | skills/ | 3h |
| T5.3 | 编写 auto-patrol skill: 跨层违规引用检测 | MEDIUM | skills/ | 3h |
| T5.4 | 编写 auto-patrol skill: 扁平缺陷自动提升建议 | MEDIUM | skills/ | 4h |
| T5.5 | 集成到 self-iteration-agent: 巡检→修复闭环 | HIGH | root/ | 4h |

---

## 五、多Agent自动巡检修复方案

### 5.1 巡检架构

```
┌─────────────────────────────────────────┐
│         Self-Iteration-Agent            │
│  (orchestrator, 调度巡检周期)            │
├─────────┬──────────┬──────────┬─────────┤
│ Patrol- │ Patrol-  │ Patrol-  │ Patrol- │
│ Depend  │ Island   │ Layer    │ Flat    │
│ Graph   │ Detect   │ Violate  │ Detect  │
├─────────┼──────────┼──────────┼─────────┤
│ petgraph│ capability│ 6-layer │ constellation│
│ cycle   │ _registry │ dep    │ level   │
│ detect  │ orphan   │ check   │ < C3    │
└─────────┴──────────┴──────────┴─────────┘
         ↓ 发现问题 → 自动修复建议/执行
┌─────────────────────────────────────────┐
│         Repair-Healer Agent             │
│  (执行修复: 合并/迁移/提升)              │
└─────────────────────────────────────────┘
```

### 5.2 巡检规则

| 规则ID | 检测内容 | 修复动作 |
|--------|---------|---------|
| PATROL-001 | 能力图有向环 → 违反6层依赖 | 标记环路, 建议下沉/上移 |
| PATROL-002 | 能力节点无出边且无入边 → 孤岛 | 建议删除或接入图谱 |
| PATROL-003 | L(n) 引用 L(n+k) (k>0) → 跨层违规 | 建议引入桥接层 |
| PATROL-004 | constellation < C3 且注册 > 30天 → 扁平缺陷 | 建议 Rust 化或标记弃用 |
| PATROL-005 | 同一 provides 列表出现 >2 次 → 冗余 | 建议合并 |
| PATROL-006 | skill 无 triggers 或 dependencies → 配置缺失 | 自动补充 |

### 5.3 执行节奏

- **每次 commit**: PATROL-001 (依赖图), PATROL-003 (跨层)
- **每日**: PATROL-002 (孤岛), PATROL-005 (冗余)
- **每周**: PATROL-004 (扁平), PATROL-006 (配置)
- **每次迭代结束**: 全量巡检 + 修复报告

---

## 六、投入产出估算

| 阶段 | 工时 | 预期收益 |
|------|------|---------|
| Phase 1 冗余清理 | 18h | 减少 40% 设计技能碎片, 统一文件入口 |
| Phase 2 扁平修复 | 46h | 新增 4 个 Rust 原生模块, 能力从 C2→C4 |
| Phase 3 错位修复 | 15h | 修正 4 个跨层依赖, 消除架构坏味道 |
| Phase 4 模式融合 | 30h | 吸收 8 个外部模式, 增强 5 个子系统 |
| Phase 5 巡检修复 | 18h | 6 条自动巡检规则, 持续质量保障 |
| **总计** | **127h** | **设计能力从纯 markdown → Rust 原生 + 智能路由 + 自动巡检** |

---

## 七、关键决策点

1. **Design Rust 化深度**: 是做薄包装(调用 LLM)还是完整本地实现? 建议: BM25/rules 本地, 生成走 LLM
2. **检索范式选择器**: 需要新的 trait 还是扩展 UnifiedCapability? 建议: 新增 Retrievable trait
3. **巡检修复权限**: 自动修复还是仅建议? 建议: Phase 5 前期仅建议, 成熟后开放自动修复
4. **hermes-jev 集成方式**: 作为独立 crate 还是嵌入 neotrix-gateway? 建议: 独立 crate + gateway 注册

---

## 八、Liquid Glass 生态分析 (补充)

### 8.1 生态规模

- GitHub topic `liquid-glass`: **712 个公开仓库**
- 语言分布: Swift 254 | TypeScript 124 | JavaScript 68 | Python 36 | HTML 34 | Kotlin 34 | CSS 27 | Dart 21 | C# 13 | C++ 10
- 驱动力: Apple WWDC25 Liquid Glass 设计语言发布

### 8.2 核心技术路线对比

| 仓库 | ★ | 技术路线 | 平台 | 可融合度 |
|------|---|---------|------|---------|
| **hyalite** (VII-Cae) | 202 | SDF lens map + SVG displacement + backdrop-filter, 单文件无WebGL | Web | **极高** — 纯 CSS/SVG, 可直接集成到 Tauri 前端 |
| AndroidLiquidGlass (Kyant0) | 3.9k | Compose Multiplatform Liquid Glass | Android/Cross | 高 — Jetpack Compose shader |
| liquid-glass (shuding) | 1.2k | Copy-paste Liquid Glass shader with SVG | Web | 高 — 轻量 SVG 方案 |
| liquid-glass-studio (iyinchao) | 704 | WebGL2 + WebGPU 全功能工作室 | Web | 中 — 重量级, 适合参考不适合直接用 |
| electron-liquid-glass (Meridius-Labs) | 582 | Electron bindings for Apple Liquid Glass | Desktop | 高 — 可用于 Tauri |
| LiquidGlassKit (DnV1eX) | 432 | iOS 13-18 backport + iOS 26 重实现 | iOS | 中 — Swift, 需桥接 |
| vaso (huozhi) | 345 | React Liquid Glass 组件 | Web | 高 — React, 可直接用于 Tauri 前端 |
| liquid_glass_widgets (sdegenaar) | 637 | Flutter shader-based glass widgets | Flutter/Cross | 中 — Dart, 需桥接 |
| liquid-glass-react (rdev) | — | React Liquid Glass 组件 (lencx 收录) | Web | **高** — React, 有在线 demo |
| liquid-glass-vue (Muggleee) | — | Vue shader/WebGL glass (lencx 收录) | Web | **高** — Vue, shader 路线 |
| liquid-glass-effect-macos (lucasromerodb) | — | WWDC 2025 effect 复刻 (lencx 收录) | Web | 中 — CodePen demo, 参考价值 |

### 8.3 hyalite 核心技术深度分析

**架构**: 单文件 `hyalite.js` (MIT), 无依赖, 无构建步骤

**渲染管线**:
1. SDF (Signed Distance Function) → 圆角矩形距离场
2. Snell's law 折射 → 玻璃厚度 + 斜面宽度 → 位移场
3. 可选折叠 (slope > 1) → 液态漩涡效果
4. PNG 编码: R=x偏移, G=y偏移, B=边缘光
5. SVG filter: feImage → feGaussianBlur → feDisplacementMap → feComposite
6. `backdrop-filter: url(#id)` 实时渲染

**关键参数**:
- `bevel` (37px): 弯曲区域宽度
- `thickness` (59px): 玻璃厚度
- `slope` (2.7): 折叠斜率 (>1 = 折叠 = 液态效果)
- `shape`: circle/squircle/lip 三种斜面轮廓
- `dispersion` (1.6): 色散像素分离
- `shade` (0.46): 边缘暗化 (焦散 + 菲涅尔)
- `rim` (1.76): 边缘反射光
- `edge` (0.32): CSS 边缘线强度
- `light` (-140°): 光照方向

**性能优化**:
- 四角对称: 只计算 1/4 象限, 镜像其余
- 尺寸桶: 相近元素共享 map (≤2% 偏差)
- 两级缓存: map 缓存 (几何+参数) + filter 缓存 (视觉效果)
- 大尺寸降采样: field 平滑, feImage 拉伸无可见损失

**浏览器支持**: Chromium (完整), Safari/Firefox (CSS fallback)

### 8.4 Liquid Glass → NeoTrix 融合方案

#### 融合点 L1: Tauri 前端 liquid glass 效果

```
Before: src-tauri/frontend/ 无 glass 效果
After:  集成 hyalite.js (单文件, 无依赖)
        设计系统增加 glass token 层
        components/ 增加 GlassCard, GlassPanel, GlassNav
```

**实现路径**:
1. 复制 `hyalite.js` 到 `frontend/src/lib/hyalite.js`
2. 在 `design-tokens.css` 增加 glass 语义 token:
   ```css
   --glass-bevel: 37px;
   --glass-thickness: 59px;
   --glass-slope: 2.7;
   --glass-blur: 1px;
   --glass-dispersion: 1.6;
   --glass-shade: 0.46;
   --glass-rim: 1.76;
   --glass-edge: 0.32;
   ```
3. 创建 `GlassCard.tsx` 封装 Hyalite.attach/detach
4. 在 SmartCanvas 面板渲染器中启用 glass 效果

#### 融合点 L2: 设计系统增加 glass 设计语言

```
design-identity skill 增加:
  - Glass archetype: 透明度层级 (frosted/translucent/clear)
  - Glass color roles: tintColor, edgeColor, rimColor
  - Glass geometry: bevel profile 选择规则
  - Glass anti-patterns: 过度折射、色散溢出、边缘锯齿
```

#### 融合点 L3: 设计审计增加 glass 维度

```
design-audit D11: Glass Quality
  - 折射强度与内容可读性平衡
  - 边缘光与背景对比度
  - 色散不导致文字模糊
  - reduced-motion 下的 fallback 表现
  - 跨浏览器一致性 (Chromium vs fallback)
```

### 8.5 Apple 官方资源 (lencx/awesome 收录)

| 资源 | 类型 | 关键信息 |
|------|------|---------|
| [Liquid Glass 文档](https://developer.apple.com/documentation/technologyoverviews/liquid-glass) | 官方文档 | 设计+开发指南 |
| [Get to know the new design system](https://developer.apple.com/videos/play/wwdc2025/356) | WWDC 视频 | 设计系统全览 |
| [Meet Liquid Glass](https://developer.apple.com/videos/play/wwdc2025/219) | WWDC 视频 | Liquid Glass 技术介绍 |
| [Icon Composer](https://developer.apple.com/icon-composer) | 工具 | 多层 Liquid Glass 图标创建, 支持 Xcode 集成 |
| [SVG Displacement Filtering](https://www.smashingmagazine.com/2021/09/deep-dive-wonderful-world-svg-displacement-filtering/) | 深度文章 | SVG 位移滤镜原理 (hyalite 理论基础) |

### 8.6 Liquid Glass 生态其他可吸收模式

| 模式 | 来源 | 融合目标 |
|------|------|---------|
| Compose Multiplatform shader | AndroidLiquidGlass | L1: 跨平台 glass 效果抽象 |
| Electron bindings | electron-liquid-glass | L1: Tauri backend glass API |
| iOS backport architecture | LiquidGlassKit | L5: 设计系统跨版本兼容模式 |
| Flutter fragment shader | liquid_glass_widgets | L1: shader-based glass 路径 |
| React component API | vaso + liquid-glass-react | L1: React glass 组件设计 |
| Vue shader/WebGL | liquid-glass-vue | L1: Vue glass 组件参考 |
| SVG displacement 原理 | Smashing Magazine 文章 | L2: 视觉感知理论基础 |
| Icon Composer 多层图标 | Apple 官方 | L1: 图标生成管线参考 |

### 8.7 更新后的任务清单 (Phase 4 补充)

| # | 任务 | 优先级 | 影响范围 | 预估工时 |
|---|------|--------|---------|---------|
| T4.6 | 集成 hyalite.js 到 Tauri 前端 | HIGH | frontend/ | 4h |
| T4.7 | 创建 glass design tokens 层 | HIGH | design-tokens.css | 2h |
| T4.8 | 创建 GlassCard/GlassPanel/GlassNav 组件 | HIGH | frontend/components/ | 6h |
| T4.9 | SmartCanvas 面板启用 glass 效果 | MEDIUM | frontend/canvas/ | 4h |
| T4.10 | design-identity 增加 glass 设计语言 | MEDIUM | skills/design/ | 3h |
| T4.11 | design-audit 增加 D11:Glass Quality 维度 | MEDIUM | skills/design/ | 2h |
| T4.12 | 研究 liquid-glass-react 组件 API 设计 | LOW | 参考 | 2h |
| T4.13 | 研究 liquid-glass-vue shader 路径 | LOW | 参考 | 2h |
| T4.14 | 吸收 Icon Composer 多层图标格式到 nt_design_icon | MEDIUM | nt_design/ | 4h |

---

## 十、UI-Layouts + 图标生态 + 设计系统 全景融合

### 10.1 UI-Layouts.com 组件矩阵 (60+ 组件)

**来源**: https://www.ui-layouts.com — Tailwind + Motion, copy-paste 组件库

| 类别 | 组件 | 融合价值 |
|------|------|---------|
| **视觉特效** | Liquid-Glass, Noise, Blur Vignette, Spotlight Cards, Animated Beam | **极高** — glass 效果直接可用 |
| **动画/运动** | Scroll Animation, Stacking Card, Marquee, Sparkles, Type Writer, Terminal UI | 高 — SmartCanvas 动效参考 |
| **叠加层** | Dialog, Linear Modal, Directional Drawer, Motion Drawer | 高 — Tauri 面板系统 |
| **表单** | Color Picker, Range Slider, File Upload, Datetime Picker | 中 — 设置页面组件 |
| **3D/视觉** | Globe, Mesh Gradients, R3F Blob, Image Ripple | 中 — 可视化增强 |
| **布局** | Footers, Responsive Header, Masonry Grid | 中 — 页面骨架 |
| **卡片** | Hover Cards, Product Cards, Gradient Border | 高 — 知识卡片/能力卡片 |
| **代码** | Code Block, Tree Code Viewer, Code Tabs | 高 — 代码展示 |

**关键模式**:
- `motion` 库 (非 framer-motion) 作为动画引擎
- `clsx` + `tailwind-merge` 组合 (`cn()` 工具函数)
- HSL 色彩系统 (CSS 变量 `--background: 0 0% 100%`)
- 语法高亮颜色 token 化 (`--ch-0` 到 `--ch-26`)

### 10.2 图标生态全景 (NeoTrix 图标能力骨架)

#### 10.2.1 顶级 SVG 图标库

| 库 | ★ | 图标数 | 网格 | 笔画 | 融合方式 |
|----|---|--------|------|------|---------|
| **Lucide** | 24.2k | 1,600+ | 24×24 | stroke | **首选** — shadcn/ui 默认, 最活跃社区 |
| Heroicons | 23.8k | 292 | 24×24 | stroke/fill | Tailwind 原生, 精选集 |
| Phosphor | 7.4k | 5,400+ | 24×24 | 6 weights | 权重层级 + Duotone |
| Tabler | 21.2k | 6,166 | 24×24 | 2px stroke | 最大 MIT 开源集 |
| Material Symbols | 49.8k | 3,000+ | 可变字体 | Variable Font | Google 生态 |
| SF Symbols | — | 7,000+ | SF 对齐 | 9 weights | Apple 生态 |
| Iconify | 6.3k | 300,000+ | 聚合 200+ 集 | 统一语法 | 万能聚合器 |

#### 10.2.2 AI 图标生成工具

| 工具 | 技术路线 | 融合价值 |
|------|---------|---------|
| **OmniSVG** | VLM (4B/8B) → SVG, NeurIPS 2025 | 高 — 端到端 text-to-SVG |
| icon-gen-ai | Iconify 275K + Anthropic/OpenAI | 中 — CLI 图标生成 |
| Zikon | Diffusion + auto PNG→SVG | 中 — Claude Code skill 友好 |
| SVG ORA Studio | 浏览器端 AI SVG 编辑器 | 中 — 参考 |

#### 10.2.3 图标动画工具

| 工具 | 技术路线 | 融合价值 |
|------|---------|---------|
| **ShapeShifter** | SVG 路径变形 + 时间线编辑 | 高 — 图标状态转换动画 |
| SVG-Morpheus-ts | Material Design "Delightful Details" | 高 — 图标 morphing |
| VectorForge | 3D 挤出 + 关键帧 + GLB 导出 | 中 — 3D 图标 |

#### 10.2.4 NeoTrix 视觉设计通用能力 (nt_design_visual)

> **设计原则**: `nt_design_visual` 不是"图标工具"，而是**所有视觉符号/设计图的通用能力层**。
> 覆盖: 图标、Logo、插图、架构图、流程图、数据图、徽章、图案、3D 资产、UI 截图标注。

##### 视觉符号分类体系

| 类别 | 子类 | 尺寸范围 | 输出格式 | 典型场景 |
|------|------|---------|---------|---------|
| **Icon** | app-icon, favicon, menu-icon, toolbar-icon | 16-1024px | SVG/PNG/ICO | 应用图标, 网站图标, 菜单图标 |
| **Logo** | wordmark, logomark, combination, emblem | 任意 | SVG/PDF | 品牌标志 |
| **Illustration** | vector-art, technical-drawing, isometric | 任意 | SVG/GLB | 技术文档, 产品展示 |
| **Diagram** | architecture, flowchart, sequence, ER, mind-map | 任意 | SVG/PNG | 架构图, 流程图 |
| **Infographic** | chart, timeline, comparison, process | 任意 | SVG/PNG | 数据可视化 |
| **Badge** | status, count, label, chip, tag | 16-256px | SVG/PNG | 状态标记, 标签 |
| **Pattern** | geometric, organic, noise, grid | 平铺 | SVG/PNG | 背景纹理, 装饰 |
| **3D Asset** | model, scene, animation | — | GLB/GLTF | 三维展示 |
| **UI Annotation** | screenshot-overlay, callout, highlight | — | SVG/PNG | 文档标注, 教程 |

##### 模块结构

```
nt_design_visual/                          ← 统一入口
│
├── nt_visual_core.rs                      ← L0: 核心类型
│   ├── VisualAsset (enum)                 ← 所有视觉资产的统一表示
│   ├── AssetMetadata                      ← 尺寸/格式/色彩空间/网格
│   ├── VisualGrid                         ← 网格系统 (24×24, 48×48, 任意)
│   └── VisualFormat                       ← SVG/PNG/ICO/WOFF2/GLB/PDF
│
├── nt_visual_construct.rs                 ← L1: 矢量构建引擎
│   ├── svg_path_ops()                     ← SVG 路径操作 (布尔运算)
│   ├── grid_snap()                        ← 网格对齐
│   ├── optical_adjust()                   ← 光学调整 (圆角/断开/缩放)
│   ├── stroke_system()                    ← 笔画系统 (stroke-width/linecap/join)
│   └── color_system()                     ← 色彩系统 (primary/secondary/accent)
│
├── nt_visual_generate.rs                  ← L1: AI 生成引擎
│   ├── text_to_svg()                      ← 文本→SVG (OmniSVG 路线)
│   ├── text_to_image()                    ← 文本→位图 (扩散模型)
│   ├── image_to_svg()                     ← 位图→矢量 (vtracer)
│   ├── image_to_icon()                    ← 图片→图标 (crop+resize+optimize)
│   ├── contact_sheet()                    ← 多候选生成 (6-12 候选)
│   └── style_transfer()                   ← 风格迁移 (品牌一致性)
│
├── nt_visual_transform.rs                 ← L1: 变换引擎
│   ├── resize_intelligent()               ← 智能缩放 (内容感知)
│   ├── crop_content_aware()               ← 内容感知裁剪
│   ├── rotate precise()                   ← 精确旋转
│   ├── color_replace()                    ← 色彩替换
│   ├── background_remove()                ← 背景移除
│   └── svg_optimize()                     ← SVG 优化 (SVGO 模式)
│
├── nt_visual_animate.rs                   ← L1: 动画引擎
│   ├── path_morph()                       ← 路径变形 (ShapeShifter)
│   ├── stroke_draw()                      ← 笔画绘制动画
│   ├── frame_sequence()                   ← 帧序列
│   ├── lottie_export()                    ← Lottie 导出
│   └── icon_state_transition()            ← 图标状态转换
│
├── nt_visual_validate.rs                  ← L1: 验证引擎
│   ├── readability_check()                ← 可读性检查 (32px 最小)
│   ├── contrast_check()                   ← 对比度检查 (WCAG)
│   ├── brand_consistency()                ← 品牌一致性 (VI-First)
│   ├── grid_compliance()                  ← 网格合规性
│   ├── stroke_consistency()               ← 笔画一致性
│   └── color_palette_check()              ← 色彩调色板检查
│
├── nt_visual_export.rs                    ← L1: 多平台导出
│   ├── export_favicon()                   ← favicon (16/32/48/ICO)
│   ├── export_appicon_ios()               ← iOS AppIcon.appiconset
│   ├── export_appicon_android()           ← Android adaptive-icon
│   ├── export_appicon_macos()             ← macOS AppIcon.icns
│   ├── export_font()                      ← 图标字体 (TTF/WOFF2)
│   ├── export_lottie()                    ← Lottie JSON
│   ├── export_glb()                       ← 3D GLB
│   └── export_all_platforms()             ← 一键全平台导出
│
├── nt_visual_lib.rs                       ← L1: 图标库管理
│   ├── LucideIndex (1,600+)               ← Lucide 索引
│   ├── HeroiconsIndex (292)               ← Heroicons 索引
│   ├── PhosphorIndex (5,400+)             ← Phosphor 索引
│   ├── TablerIndex (6,166)                ← Tabler 索引
│   ├── IconifyAggregate (300K+)           ← Iconify 聚合
│   ├── search_by_name()                   ← 按名称搜索
│   ├── search_by_tag()                    ← 按标签搜索
│   ├── search_by_category()               ← 按分类搜索
│   └── search_similar()                   ← 相似图标搜索
│
├── nt_visual_diagram.rs                   ← L1: 图表/架构图引擎
│   ├── architecture_diagram()             ← 架构图生成
│   ├── flowchart()                        ← 流程图生成
│   ├── sequence_diagram()                 ← 时序图生成
│   ├── er_diagram()                       ← ER 图生成
│   ├── mind_map()                         ← 思维导图生成
│   └── mermaid_to_svg()                   ← Mermaid→SVG 转换
│
└── nt_visual_pipeline.rs                  ← L2: 统一管线编排
    ├── pipeline_from_text()               ← 文本→完整设计管线
    ├── pipeline_from_image()              ← 图片→优化管线
    ├── pipeline_brand_kit()               ← 品牌套件生成管线
    ├── pipeline_icon_set()                ← 图标集批量生成管线
    └── pipeline_document_assets()         ← 文档资产生成管线
```

##### 与现有模块的关系

```
Before (碎片化):
  nt_file_ability::pdf_icon_enhance    ← PDF 图标增强 (与 PDF 耦合)
  nt_file_ability::image_super_resolve ← 图像超分 (12 模型)
  nt_media::thumbnail                  ← 缩略图
  skills/design/ui-skill               ← 5 modes (碎片化)

After (统一):
  nt_design_visual/                    ← 所有视觉设计的单一入口
    ↑ 调用
    ├── nt_file_ability::image_super_resolve (超分作为后端)
    ├── nt_media::thumbnail (缩略图作为特化)
    └── skills/design/design-core (组件实现作为消费方)
```

##### 通用能力矩阵

| 能力维度 | Icon | Logo | Illustration | Diagram | Badge | Pattern | 3D |
|---------|------|------|-------------|---------|-------|---------|-----|
| AI 生成 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 矢量构建 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| 网格对齐 | ✅ | ✅ | — | — | ✅ | — | — |
| 笔画系统 | ✅ | ✅ | — | — | ✅ | — | — |
| 动画 | ✅ | ✅ | — | ✅ | ✅ | — | ✅ |
| 多平台导出 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 品牌一致性 | ✅ | ✅ | ✅ | — | ✅ | ✅ | — |
| 可读性验证 | ✅ | — | — | — | ✅ | — | — |

##### 与 6 层架构的映射

```
L6_Meta:      visual 品牌一致性演化
              visual 设计语言版本控制
L5_Cognition: visual 风格选择推理
              visual 质量评估 (audit D11-D12)
L4_Feel:      visual 情感化设计规则
L3_Embodiment: visual 降级策略
              visual 跨浏览器兼容
L2_Perception: visual 质量感知
              visual 内容理解路由
L1_Action:    nt_design_visual/ (全部模块)
L0_Substrate: VisualAsset, VisualGrid, VisualFormat 类型定义
```

### 10.3 设计系统生态映射

| 项目 | ★ | 核心模式 | NeoTrix 融合点 |
|------|---|---------|---------------|
| **shadcn/ui** | 122k | Copy-paste 所有权模型 | design-core build 模式参考 |
| Ant Design | — | 企业级组件 + 设计语言 | design-system 思考 |
| Radix UI | — | 无样式可访问原语 | design-core 原子组件基础 |
| DaisyUI | 33k | 语义组件类 + 35 主题 | theme 模式参考 |
| Style Dictionary | 1.7M/周 | W3C DTCG v1 token 标准 | **design tokens 规范基础** |
| Storybook | — | 组件文档 + 测试 | design-audit 集成点 |
| Figwright | 592 | Figma↔Code MCP 双向同步 | 设计稿→代码管线 |

### 10.4 前端设计资源映射

#### 动画引擎选择

| 场景 | 推荐 | 理由 |
|------|------|------|
| React 组件动画 | **Motion** (32k★) | 声明式, 2.6KB, 布局动画 |
| 品牌/获奖网站 | **GSAP** (26k★) | 复杂时间线, 滚动触发 |
| 轻量级通用 | **Anime.js** (70k★) | 命令式, 17KB |

#### Tailwind 组件库选择

| 需求 | 推荐 |
|------|------|
| React 默认 | **shadcn/ui** |
| 多框架 | DaisyUI / Flowbite |
| 无样式原语 | Radix UI / Headless UI |
| 视觉特效 | **ui-layouts.com** (Liquid-Glass, Spotlight, Noise) |

#### 色彩工具链

```
Coolors (探索) → Realtime Colors (预览) → Contrast Checker (验证)
         ↓
    Huemint (AI 品牌色) → Atmos (UI 调色板) → Tailwind 导出
```

#### 字体配对

```
Fontpair (发现) → Fontjoy (AI 匹配) → Google Fonts (托管)
         ↓
    font-display: swap (性能) → CSS 变量映射
```

### 10.5 NeoTrix 设计能力骨架 (完整映射)

```
┌─────────────────────────────────────────────────────────┐
│                    L6_Meta                               │
│  design-identity (VI-First + Guard Rules)               │
│  glass 设计语言演化                                      │
│  visual 品牌一致性检查 + 设计语言版本控制                  │
├─────────────────────────────────────────────────────────┤
│                    L5_Cognition                          │
│  design-audit (10+1 维度审计)                            │
│  BM25 排名引擎 (style/palette/font 匹配)                │
│  192 条行业推理规则 + 反模式目录                           │
│  glass 风格选择推理 + visual 风格选择推理                  │
├─────────────────────────────────────────────────────────┤
│                    L4_Feel                               │
│  写作反模式检测 (lieflat-less-ai)                         │
│  品牌语气一致性 + visual 情感化设计规则                   │
├─────────────────────────────────────────────────────────┤
│                    L3_Embodiment                         │
│  glass 效果降级策略 (Chromium→fallback)                   │
│  visual 降级策略 + 跨浏览器兼容                          │
├─────────────────────────────────────────────────────────┤
│                    L2_Perception                         │
│  视觉理解路由 (route_prompt)                              │
│  visual 质量感知 + 内容理解路由                           │
├─────────────────────────────────────────────────────────┤
│                    L1_Action                             │
│  design-core (component build)                          │
│  nt_design_visual/ (统一视觉设计引擎)                     │
│    ├── construct (矢量构建)                               │
│    ├── generate (AI 生成)                                 │
│    ├── transform (变换)                                   │
│    ├── animate (动画)                                     │
│    ├── validate (验证)                                    │
│    ├── export (多平台导出)                                │
│    ├── lib (图标库管理)                                   │
│    ├── diagram (图表/架构图)                              │
│    └── pipeline (统一管线编排)                            │
│  nt_design_glass (hyalite bridge)                       │
│  nt_design_tokens (token 系统)                           │
│  nt_design_bm25 (排名引擎)                               │
│  nt_design_rules (行业规则索引)                           │
├─────────────────────────────────────────────────────────┤
│                    L0_Substrate                          │
│  VisualAsset, VisualGrid, VisualFormat 类型定义          │
│  Token 类型 (DesignToken, ColorRole, TypeScale)          │
│  Glass 参数类型 (GlassConfig)                             │
└─────────────────────────────────────────────────────────┘
```

---

## 十一、全项目并行能力统一融合

### 11.1 现有能力盘点 (44 个相关能力)

#### 强重叠 (28 个 — 应合并到 nt_design_visual)

| # | 现有模块 | 层级 | 路径 | 重叠维度 | 融合目标 |
|---|---------|------|------|---------|---------|
| 1 | `nt_file_ability::pdf_icon_enhance` | L1 | neotrix-core | 图标增强 | visual/validate + visual/export |
| 2 | `nt_file_ability::image_super_resolve` | L1 | neotrix-core | 图像超分 | visual/transform (后端) |
| 3 | `nt_file_ability::pdf_image_extract` | L1 | neotrix-core | PDF 图像提取 | visual/transform |
| 4 | `nt_file_ability::ocr` | L1 | neotrix-core | OCR 识别 | visual/validate (文字检测) |
| 5 | `nt_file_ability::visual::route_prompt` | L1 | neotrix-core | 视觉路由 | visual/pipeline (路由层) |
| 6 | `nt_media::thumbnail` | L1 | neotrix-core | 缩略图生成 | visual/export |
| 7 | `nt_media::video_audit_trail` | L1 | neotrix-core | 视频审计 | visual/validate |
| 8 | `nt_media::vtuber` | L1 | neotrix-core | 虚拟形象 | visual/generate (3D) |
| 9 | `skills/design/ui-skill` | — | skills/ | 5 modes UI | design-core (已合并) |
| 10 | `skills/design/agentic-design-system` | — | skills/ | 3-pass 审计 | design-audit (已合并) |
| 11 | `skills/design/ui-reasoning` | — | skills/ | 10 维度推理 | design-audit (已合并) |
| 12 | `skills/design/visual-taste-lab` | — | skills/ | VI-First | design-identity (已合并) |
| 13 | `skills/design/decantr-design` | — | skills/ | 3 层上下文 | design-identity (已合并) |
| 14 | `skills/design/knowledge_engine.json` | — | skills/ | 设计知识库 | visual/lib (知识索引) |
| 15 | `skills/design/review-findings.json` | — | skills/ | 审查发现 | visual/validate |
| 16 | `skills/design/evidence-atlas.html` | — | skills/ | 证据图谱 | visual/diagram |
| 17 | `skills/design/neotrix-management-ui.html` | — | skills/ | 管理 UI | design-core (参考) |
| 18 | `skills/design/template/preview-ui-v2.html` | — | skills/ | UI 模板 | design-core (模板) |
| 19 | `skills/design/previews/*` | — | skills/ | 预览文件 | visual/export |
| 20 | `frontend/src/components/DesignModePanel.tsx` | L1 | frontend | 设计模式面板 | design-core (消费方) |
| 21 | `frontend/src/components/GenUIView.tsx` | L1 | frontend | UI 生成视图 | visual/pipeline (消费方) |
| 22 | `frontend/src/components/LivePreview.tsx` | L1 | frontend | 实时预览 | visual/pipeline (消费方) |
| 23 | `frontend/src/components/AnnotatedImage.tsx` | L1 | frontend | 标注图片 | visual/diagram (消费方) |
| 24 | `frontend/src/components/FlowerOfLife.tsx` | L1 | frontend | 生命之花 | visual/generate (图案) |
| 25 | `frontend/src/components/OfficeFloor.tsx` | L1 | frontend | 办公楼层 | visual/diagram |
| 26 | `frontend/src/components/GlobeView.tsx` | L1 | frontend | 球体视图 | visual/generate (3D) |
| 27 | `frontend/src/components/KbGraphVisualization.tsx` | L1 | frontend | 知识图谱 | visual/diagram |
| 28 | `frontend/src/components/CausalMap.tsx` | L1 | frontend | 因果图 | visual/diagram |

#### 部分重叠 (10 个 — 保留但桥接)

| # | 现有模块 | 层级 | 保留理由 | 桥接方式 |
|---|---------|------|---------|---------|
| 29 | `nt_core_e8/` (E8 格数学) | L2 | 格数学独立 | visual/construct 可调用 |
| 30 | `nt_core_hcube/` (超立方 VSA) | L2 | VSA 独立 | visual/animate 可调用 |
| 31 | `nt_core_code_search/` | L2 | 代码搜索独立 | visual/lib 可引用 |
| 32 | `nt_memory_kb/` (知识库) | L4 | 知识库独立 | visual/lib 可查询 |
| 33 | `nt_core_graph/` (图结构) | L0 | 图结构基础 | visual/diagram 底层 |
| 34 | `nt_core_edit/` (diff) | L0 | 编辑操作基础 | visual/transform 底层 |
| 35 | `nt_core_e8_predictor/` | L2 | 预测独立 | visual/generate 可调用 |
| 36 | `nt_feel/` (情感引擎) | L4 | 情感独立 | design-identity 可引用 |
| 37 | `nt_world/` (世界模型) | L2 | 世界模型独立 | visual/pipeline 可查询 |
| 38 | `nt_routing/` (路由) | L2 | 路由独立 | visual/pipeline 路由层 |

#### 轻微重叠 (5 个 — 保留不动)

| # | 现有模块 | 层级 | 保留理由 |
|---|---------|------|---------|
| 39 | `nt_core_event_bus/` | L0 | 事件总线基础 |
| 40 | `nt_core_di/` | L0 | 依赖注入基础 |
| 41 | `nt_core_telemetry/` | L0 | 遥测基础 |
| 42 | `nt_core_cache/` | L0 | 缓存基础 |
| 43 | `nt_core_time/` | L0 | 时间基础 |
| 44 | `nt_core_span/` | L0 | 追踪基础 |

### 11.2 统一融合方案

#### 融合原则

```
1. 单一入口: 所有视觉设计能力通过 nt_design_visual/ 统一暴露
2. 向后兼容: 现有调用方通过适配器继续工作
3. 能力下沉: 通用能力下沉到 L0/L1, 特化能力保留在原模块
4. 桥接模式: 保留但不强制迁移的模块通过 trait 桥接
```

#### 融合拓扑图

```
                    ┌──────────────────────────────┐
                    │     nt_design_visual/         │
                    │     (统一视觉设计引擎)         │
                    └──────────┬───────────────────┘
                               │ 调用
          ┌────────────────────┼────────────────────┐
          │                    │                    │
    ┌─────▼─────┐      ┌──────▼──────┐      ┌─────▼─────┐
    │ 合并模块   │      │ 桥接模块     │      │ 保留模块   │
    │ (28→合并)  │      │ (10→桥接)    │      │ (6→不动)   │
    └───────────┘      └─────────────┘      └───────────┘
```

#### 具体合并操作

**操作 A: 能力迁移 (28 个模块)**

```
nt_file_ability::pdf_icon_enhance
  → nt_design_visual::validate::icon_quality_check()
  → nt_design_visual::export::enhance_icon()
  (原模块保留为 thin wrapper 调用 visual)

nt_file_ability::image_super_resolve
  → nt_design_visual::transform::super_resolve()
  (原模块保留为 thin wrapper, 12 模型选择逻辑移入 visual)

nt_file_ability::pdf_image_extract
  → nt_design_visual::transform::extract_images_from_pdf()
  (原模块保留为 thin wrapper)

nt_media::thumbnail
  → nt_design_visual::export::generate_thumbnail()
  (原模块保留为 thin wrapper)

frontend/src/components/AnnotatedImage.tsx
  → 消费 nt_design_visual::diagram::annotate()
  (组件保留, 改为调用 visual API)

frontend/src/components/KbGraphVisualization.tsx
  → 消费 nt_design_visual::diagram::render_graph()
  (组件保留, 改为调用 visual API)

frontend/src/components/CausalMap.tsx
  → 消费 nt_design_visual::diagram::render_causal()
  (组件保留, 改为调用 visual API)
```

**操作 B: Trait 桥接 (10 个模块)**

```rust
// nt_design_visual/bridge.rs
pub trait VisualBackend: Send + Sync {
    /// E8 格数学后端 (用于图案生成)
    fn e8_lattice(&self) -> &dyn E8Lattice;
    /// VSA 后端 (用于向量符号运算)
    fn hcube_vsa(&self) -> &dyn HyperCubeVSA;
    /// 图结构后端 (用于 diagram 底层)
    fn graph_store(&self) -> &dyn GraphStore;
    /// 知识库后端 (用于图标语义搜索)
    fn knowledge_base(&self) -> &dyn KnowledgeBase;
    /// 预测后端 (用于风格预测)
    fn predictor(&self) -> &dyn E8Predictor;
    /// 情感后端 (用于情感化设计)
    fn emotion_engine(&self) -> &dyn EmotionEngine;
}

// 实现: 各现有模块注册为 backend
impl VisualBackend for NtCoreE8 { ... }
impl VisualBackend for NtCoreHcube { ... }
impl VisualBackend for NtMemoryKB { ... }
```

### 11.3 缺口填补 (10 个缺失能力)

| 缺口 | 影响 | 工作量 | 优先级 | 融合目标模块 |
|------|------|--------|--------|------------|
| **图标生成管线** | HIGH | MED | **P0** | visual/generate |
| **色彩调色板生成** | HIGH | LOW | **P0** | visual/construct |
| **设计 Token 管线** | HIGH | MED | **P0** | visual/pipeline |
| **字体系统** | MED | LOW | **P1** | visual/construct |
| **布局/间距系统** | MED | LOW | **P1** | visual/construct |
| **截图→代码** | HIGH | HIGH | **P1** | visual/pipeline |
| **视觉资产管线** | MED | MED | **P2** | visual/export |
| **动画/动效设计** | MED | MED | **P2** | visual/animate |
| **响应式设计智能** | LOW | HIGH | **P3** | visual/pipeline |
| **设计系统文档** | LOW | MED | **P3** | visual/validate |

### 11.4 更新后的 nt_design_visual/ 完整结构

```
nt_design_visual/
│
├── core/                           ← L0: 核心类型
│   ├── mod.rs
│   ├── asset.rs                    ← VisualAsset (9 类视觉符号)
│   ├── grid.rs                     ← VisualGrid (24/48/任意)
│   ├── format.rs                   ← VisualFormat (SVG/PNG/ICO/WOFF2/GLB/PDF)
│   ├── token.rs                    ← DesignToken, ColorRole, TypeScale
│   └── bridge.rs                   ← VisualBackend trait (桥接 10 个模块)
│
├── construct/                      ← L1: 矢量构建引擎
│   ├── mod.rs
│   ├── path_ops.rs                 ← SVG 路径布尔运算
│   ├── grid_snap.rs                ← 网格对齐
│   ├── optical.rs                  ← 光学调整
│   ├── stroke.rs                   ← 笔画系统
│   ├── color.rs                    ← 色彩系统 + 调色板生成 [新增 P0]
│   ├── typography.rs               ← 字体系统 + 配对 [新增 P1]
│   └── layout.rs                   ← 布局/间距系统 [新增 P1]
│
├── generate/                       ← L1: AI 生成引擎
│   ├── mod.rs
│   ├── text_to_svg.rs              ← OmniSVG 路线
│   ├── text_to_image.rs            ← 扩散模型
│   ├── image_to_svg.rs             ← vtracer 矢量化
│   ├── image_to_icon.rs            ← 图片→图标
│   ├── contact_sheet.rs            ← 多候选生成
│   ├── style_transfer.rs           ← 风格迁移
│   └── icon_pipeline.rs            ← 图标生成管线 [新增 P0]
│
├── transform/                      ← L1: 变换引擎
│   ├── mod.rs
│   ├── resize.rs                   ← 智能缩放
│   ├── crop.rs                     ← 内容感知裁剪
│   ├── rotate.rs                   ← 精确旋转
│   ├── color_replace.rs            ← 色彩替换
│   ├── background_remove.rs        ← 背景移除
│   ├── svg_optimize.rs             ← SVG 优化
│   ├── super_resolve.rs            ← 超分辨率 (from nt_file_ability) [合并]
│   └── pdf_extract.rs              ← PDF 图像提取 (from nt_file_ability) [合并]
│
├── animate/                        ← L1: 动画引擎
│   ├── mod.rs
│   ├── path_morph.rs               ← 路径变形
│   ├── stroke_draw.rs              ← 笔画绘制
│   ├── frame_sequence.rs           ← 帧序列
│   ├── lottie_export.rs            ← Lottie 导出
│   ├── state_transition.rs         ← 图标状态转换
│   └── motion_tokens.rs            ← 动效 Token [新增 P2]
│
├── validate/                       ← L1: 验证引擎
│   ├── mod.rs
│   ├── readability.rs              ← 可读性检查
│   ├── contrast.rs                 ← 对比度检查 (WCAG)
│   ├── brand_consistency.rs        ← 品牌一致性
│   ├── grid_compliance.rs          ← 网格合规
│   ├── stroke_consistency.rs       ← 笔画一致性
│   ├── color_palette_check.rs      ← 调色板检查
│   ├── icon_quality.rs             ← 图标质量 (from pdf_icon_enhance) [合并]
│   └── visual_regression.rs        ← 视觉回归 [新增 P3]
│
├── export/                         ← L1: 多平台导出
│   ├── mod.rs
│   ├── favicon.rs                  ← favicon 导出
│   ├── appicon_ios.rs              ← iOS AppIcon
│   ├── appicon_android.rs          ← Android adaptive-icon
│   ├── appicon_macos.rs            ← macOS AppIcon
│   ├── font.rs                     ← 图标字体
│   ├── lottie.rs                   ← Lottie JSON
│   ├── glb.rs                      ← 3D GLB
│   ├── thumbnail.rs                ← 缩略图 (from nt_media) [合并]
│   └── all_platforms.rs            ← 一键全平台
│
├── lib/                            ← L1: 图标库管理
│   ├── mod.rs
│   ├── lucide.rs                   ← Lucide 1,600+
│   ├── heroicons.rs                ← Heroicons 292
│   ├── phosphor.rs                 ← Phosphor 5,400+
│   ├── tabler.rs                   ← Tabler 6,166
│   ├── iconify.rs                  ← Iconify 300K+
│   ├── search.rs                   ← 搜索引擎
│   └── metadata.rs                 ← 元数据注册
│
├── diagram/                        ← L1: 图表/架构图引擎
│   ├── mod.rs
│   ├── architecture.rs             ← 架构图
│   ├── flowchart.rs                ← 流程图
│   ├── sequence.rs                 ← 时序图
│   ├── er.rs                       ← ER 图
│   ├── mind_map.rs                 ← 思维导图
│   ├── mermaid.rs                  ← Mermaid→SVG
│   ├── graph_render.rs             ← 图渲染 (from KB/Graph) [桥接]
│   └── annotate.rs                 ← 截图标注 [合并 frontend]
│
├── pipeline/                       ← L2: 统一管线编排
│   ├── mod.rs
│   ├── from_text.rs                ← 文本→完整设计
│   ├── from_image.rs               ← 图片→优化管线
│   ├── brand_kit.rs                ← 品牌套件生成
│   ├── icon_set.rs                 ← 图标集批量生成
│   ├── doc_assets.rs               ← 文档资产生成
│   ├── screenshot_to_code.rs       ← 截图→代码 [新增 P1]
│   └── token_sync.rs               ← Token 同步 [新增 P0]
│
└── lib.rs                          ← 统一导出
```

### 11.5 更新后的全量方案

| Phase | 内容 | 工时 | 状态 |
|-------|------|------|------|
| Phase 1 | 冗余清理 (5→3 design skill) | 18h | ✅ 完成 |
| Phase 2 | 统一融合 (28 模块→nt_design_visual) | 40h | 待执行 |
| Phase 3 | 跨域错位修复 | 15h | 待执行 |
| Phase 4A | 外部模式融合 | 30h | 待执行 |
| Phase 4B | Liquid Glass 融合 | 29h | 待执行 |
| Phase 4C | 视觉设计生态融合 | 18h | 待执行 |
| Phase 4D | UI-Layouts 组件吸收 | 16h | 待执行 |
| Phase 5 | 多Agent自动巡检修复 | 18h | 部分完成 |
| Phase 6 | 缺口填补 (10 个新增能力) | 45h | 待执行 |
| **总计** | | **229h** | |

---

## 九、Liquid Glass 技术路线决策

### 9.1 推荐方案: hyalite (SVG displacement) 为主, shader 为备

**理由**:
1. **零依赖**: 单文件, 无 WebGL, 无构建步骤 → 与 NeoTrix "轻量" 原则一致
2. **Chromium 完整支持**: Tauri 使用 Chromium WebView, 完美匹配
3. **Fallback 优雅**: 非 Chromium 浏览器自动降级为 CSS blur
4. **性能可控**: 两级缓存 + 尺寸桶 + 降采样, 适合 SmartCanvas 多面板场景
5. **参数化**: bevel/thickness/slope/dispersion 等参数可映射为 design tokens

### 9.2 Shader 路径作为备选

当需要以下能力时切换到 WebGL/WebGPU shader:
- 跨浏览器一致渲染 (Safari/Firefox)
- 更复杂的光学效果 (多层折射、动态光照)
- 与 3D 场景集成 (Three.js/Babylon.js)

参考: liquid-glass-studio (WebGL2/WebGPU), liquid-glass-vue (shader)

### 9.3 与 NeoTrix 6层架构的映射

```
L1_Action:    nt_design_glass.rs (Rust bridge to hyalite.js)
              GlassCard/GlassPanel 组件
              glass design tokens

L2_Perception: nt_visual_glass_perception (glass 效果下的内容可读性检测)

L5_Cognition: glass 质量评估规则 (D11)
              glass 风格选择推理 (squircle vs circle vs lip)

L6_Meta:      glass 设计语言演化 (基于用户反馈调整默认参数)
```
