# NeoTrix 设计意识体架构 v1 — Visual Design Consciousness

> 设计者: NeoTrix 意识体自架构
> 哲学根基: Three Bodies × VSA 统一表征 × 设计感知循环 × 品味进化
> 触发: Vercel Geist 范式吸收 + 7 个外部设计系统分析 + 现有 5 技能审计
> 日期: 2026-06-23

---

## 0. 当前状态审计

### 0.1 存在但碎片化

```
设计相关代码分布 (5 处散落, 2 组重复):
  ├─ nt_core_design_token/   10 files  → 视频渲染 token (FFmpeg/HUD/音频)
  ├─ nt_io_design_review/     4 files  → UI 反模式检测 (重复×3)
  ├─ nt_shield_design_review/ 4 files  → 同上 (副本)
  └─ nt_shield/inner_critic/  4 files  → 同上 (原始)
  
设计技能分布 (5 个 SKILL.md, 仅 1 个有 Rust 表示):
  ├─ ui-reasoning/            SKILL.md (11维推理框架, 无 Rust 枚举变体)
  ├─ visual-taste-lab/        SKILL.md (VI-First 工作流, 无 Rust 变体)
  ├─ decantr-design/          SKILL.md (3层守卫规则, 无 Rust 变体)
  ├─ ui-skill/                SKILL.md (5模式设计审计, 无 Rust 变体)
  └─ agentic-design-system/   SKILL.md (3-pass 评估, 有 KnowledgeSource::AgenticDS)
```

### 0.2 关键缺口

1. **设计感知不存在** — 无视觉输入分析 (颜色和谐度/对比度/节奏检测)
2. **设计知识与 VSA 断开** — `vsa_token.rs` 绑定 name→value 但无语义层级
3. **设计推理分散** — 反模式检测×3副本 (nt_io/nt_shield/inner_critic), 无统一 eval
4. **设计品味不可进化** — 无 SEAL 管道评估设计输出质量
5. **设计技能无 Rust 表示** — 5 skill 中 4 个无 KnowledgeSource 变体
6. **无设计记忆** — 品牌身份/用户偏好/设计历史不持久

---

## 1. 架构定位

设计意识是 **mind/** 层的一个完整**感知-记忆-推理-行动-进化**回路, 与 E8/GWT/HyperCube 深度集成:

```
Three Bodies 中的设计意识座标:
                                    GWT 注意力广播
                                        ↕
  User Input → mind/perception/design/ → VSA design vector
                                        ↕
                              mind/memory/design/ (HyperCube 分区)
                                        ↕
                              mind/reasoning/design/ (E8 Design States)
                                        ↕
                              mind/evolution/design/ (SEAL 品味进化)
                                        ↕
  Output ← body/design/ → CSS/Token/Component/Design.md
```

### 核心约束

- 所有设计概念以 VSA 向量表征 (不引入异构设计 token 格式)
- 设计品味作为 HyperCube 中的原型向量可进化
- 不新增独立 crate, 嵌入现有 mind/ body/ 结构
- 现有 nt_core_design_token 保持(视频渲染), 新建设计意识系统与其共存

---

## 2. Layer 1: Design Perception (mind/perception/design/)

设计感知列将视觉/文本输入转换为 VSA 设计向量。

### 2.1 架构

```
mind/perception/design/
├── mod.rs               ← DesignPerceptionBus + Percept trait
│
├── color_vision.rs      ← 色觉柱: 色板分析, 和谐度, 对比度, WCAG
│                          VSA: bind(color_scale, hue_range, step_encoding)
│
├── typography_vision.rs ← 排版视觉柱: 字体识别, 尺度分析, 节奏检测
│                          VSA: bind(type_family, size_scale, line_rhythm)
│
├── spatial_vision.rs    ← 空间视觉柱: 布局网格, 间距节奏, 层级感知
│                          VSA: bind(layout, spacing_scale, density)
│
├── pattern_vision.rs    ← 模式视觉柱: 反模式检测, 风格分类, 行业识别
│                          VSA: bind(anti_pattern, style_archetype)
│
└── fusion.rs            ← 多列融合: 对齐各视觉柱输出到统一设计 VSA
```

### 2.2 VSA 编码设计

```rust
// 设计概念的 VSA 编码模式
// 使用 HRR (Holographic Reduced Representations) 绑定

// 颜色 token:
let color_token = bind(path("color.blue.500"), encode("#3b82f6"));
let step_encoding = bind(path("color.blue.500.role"), encode("solid_fill"));

// 排版 token:
let type_token = bind(path("typography.heading.32"), encode("Geist Sans"));
let scale_token = bind(path("typography.heading.size"), encode("32px"));

// 设计模式 (复合):
let card_pattern = bind_many([
    bind(path("component.card.radius"), encode("12px")),
    bind(path("component.card.shadow"), encode("0 2px 2px rgba(0,0,0,0.04)")),
    bind(path("component.card.padding"), encode("24px")),
]);

// 风格原型 (可进化的品味向量):
let taste_prototype = weighted_sum([
    (0.6, minimal_style_vsa),
    (0.3, technical_style_vsa),
    (0.1, playful_style_vsa),
]);
```

---

## 3. Layer 2: Design Memory (mind/memory/design/)

设计记忆是 HyperCube 中的一个命名空间分区, 存储设计知识和品味历史。

### 3.1 分区

```
mind/memory/design/        ← HyperCube 命名空间 "design"
├── tokens/                ← 设计 token 记忆 (Geist/Radix/Tailwind 等范式)
│   ├── geist_scale        {color_100..1000, step_roles}
│   ├── radix_scale        {step1..12, alpha, dark}
│   └── tailwind_v4        {oklch, p3, 50..950}
│
├── patterns/              ← 设计模式记忆
│   ├── button_primary     {bg, text, radius, height, states}
│   ├── card_layout        {padding, shadow, surface}
│   └── spacing_rhythm     {inside, between, section}
│
├── brands/                ← 品牌识别记忆
│   ├── vercel_geist       {primary, gray, accent, type, spacing}
│   └── user_preferences   {per-user taste vector}
│
├── archetypes/            ← 风格原型记忆
│   ├── minimal            {VI vector: low color, high whitespace}
│   ├── technical          {VI vector: mono, grid, precise}
│   └── editorial          {VI vector: serif, generous leading}
│
└── history/               ← 设计输出历史 (供 SEAL 品味学习)
    ├── output_{timestamp}  {vsa_design, user_feedback, success_score}
    └── ...
```

### 3.2 设计 token 的 VSA 组合架构

匹配 DTCG 三层 token 标准:

```rust
// Primitive → Semantic → Component (VSA 层级)
let primitive = bind(path("color.blue.500"), encode("#3b82f6"));
let semantic = bind(path("color.semantic.action.default"), primitive);
let component = bind_many([
    bind(path("button.primary.bg"), semantic),
    bind(path("button.primary.text"), encode("#ffffff")),
    bind(path("button.primary.radius"), encode("6px")),
]);
```

---

## 4. Layer 3: Design Reasoning (mind/reasoning/design/)

设计推理使用 E8 推理核的 64 态中的专用设计态。

### 4.1 E8 Design States

从 64 态中分配 8 态给设计推理:

```
E8 Design States (hexagram 57-64):
  ┌─────┬──────────────────────────┬──────────────────────────────────┐
  │ #   │ Name                     │ Function                         │
  ├─────┼──────────────────────────┼──────────────────────────────────┤
  │ 57  │ DESIGN_EVALUATE          │ 设计评估 (WCAG/层级/节奏)         │
  │ 58  │ DESIGN_COMPOSE           │ 设计组件组合 (token→component)   │
  │ 59  │ DESIGN_SELECT            │ 风格选择 (VI-first 匹配)          │
  │ 60  │ DESIGN_CRITIQUE          │ 自我批评 (反模式检测)             │
  │ 61  │ DESIGN_TOKENIZE          │ token 架构设计 (primitive→semantic)│
  │ 62  │ DESIGN_SYSTEMIZE         │ 设计系统化 (从页面到系统)         │
  │ 63  │ DESIGN_ALIGN             │ 品牌对齐检查                      │
  │ 64  │ DESIGN_EVOLVE            │ 品味更新 (SEAL 反馈接收)          │
  └─────┴──────────────────────────┴──────────────────────────────────┘
```

### 4.2 设计推理子模块

```
mind/reasoning/design/
├── mod.rs               ← DesignReasoner (E8 states 57-64)
│
├── evaluator.rs         ← 设计评估 (WCAG AA/AAA, 层级, 节奏, 一致性)
│                          集成: 当前 3 份设计审查代码 → 合并为 1 份
│                          anti-pattern detection: 颜色×排版×可访问性×动效
│
├── style_matcher.rs     ← VI-First 风格匹配 (输入→原型选择→调适)
│                          输入: 品牌/行业/受众 → 输出: 风格原型 VSA
│
├── token_architect.rs   ← Token 架构设计 (三步: primitive→semantic→component)
│                          输入: 品牌色/行业 → 输出: token 系统 VSA
│
├── system_builder.rs    ← 设计系统构建 (从 token 集→组件规则→页面模式)
│                          输入: token 系统 + 风格原型 → 输出: 设计系统 VSA
│
└── critic.rs            ← 设计批评 (InnerCritic 的设计特化版)
│                          输入: 设计 VSA → 输出: 评分 + 改进建议
│
└── fusion.rs            ← 多设计态结果融合 + GWT 广播
```

### 4.3 设计推理的多层架构 (参考 Geist + Radix + DTCG)

```
推理流程:
  Input (品牌色/URL/描述)
    │
    ▼
  DESIGN_SELECT (57) ──── 风格原型匹配 (VI-First)
    │
    ▼
  DESIGN_TOKENIZE (61) ── Token 架构设计
    │  ├── color: step encoding (100-1000 or 1-12)
    │  ├── type: heading/label/copy/button 四族
    │  ├── spacing: 4px or 8px 基座
    │  └── elevation: 三层阴影语义
    │
    ▼
  DESIGN_SYSTEMIZE (62) ─ 系统构建 (component token 引用 primitives)
    │  └── 输出: 设计系统 VSA (含所有 token)
    │
    ▼
  DESIGN_COMPOSE (58) ─── 组件组合 (按页面需求生成组件)
    │
    ▼
  DESIGN_CRITIQUE (60) ── 自我批评 (检测反模式)
    │
    ▼
  DESIGN_ALIGN (63) ───── 品牌对齐 (对比 brand VSA)
    │
    ▼
  GWT 广播 → 输出
```

---

## 5. Layer 4: Design Action (body/design/)

设计行动层将设计推理结果转化为可消费的输出。

```
body/design/
├── mod.rs               ← DesignActionBus
│
├── token_gen.rs         ← Token 生成器 (CSS vars / JSON / DTCG 格式)
│                          输出: design-tokens.json, :root { --var }
│
├── spec_gen.rs          ← 设计规范生成器
│                          输出: DESIGN.md, design-language.md
│
├── component_gen.rs     ← 组件代码生成 (Tailwind / CSS / React)
│
└── preview_gen.rs       ← 预览生成 (HTML mockup / 调色板展示)
```

---

## 6. Layer 5: Design Evolution (mind/evolution/design/)

设计品味进化: SEAL 管道中新增一个设计品味评估维度。

### 6.1 SEAL Design Metric

```
SEAL 新增设计评估维度:
  design_quality = f(
    wcag_score,           // 对比度合规 0..1
    hierarchy_score,      // 视觉层级清晰度 0..1
    consistency_score,    // token 引用一致性 0..1
    brand_alignment,      // 品牌对齐度 0..1
    user_satisfaction,    // (外部反馈) 0..1
    taste_coherence,      // 品味向量与输出 VSA 的相似度 0..1
  )
```

### 6.2 Taste Prototype Evolution

```rust
// 品味原型进化算法:
let new_taste = weighted_sum(old_taste, feedback_vector);
// feedback_vector 来自:
// - 用户反馈 (明示评分)
// - 设计质量评分 (SEAL metric)
// - 外部范例吸收 (如 Geist 的 token 架构)

// 品味原型存储在 HyperCube 中, 跨会话持久
let taste_prototype = hypercube.get("design.taste.prototype");
```

### 6.3 范例吸收 (External Pattern Mining)

```rust
// 每次吸收外部设计系统 (如 Geist):
// 1. 提取 token 架构为 VSA
// 2. 与现有品味原型计算相似度
// 3. 如果 novelty > 阈值, 有选择地融合新范式
// 4. 更新设计模式记忆
```

---

## 7. 现有代码整合计划

### 7.1 重复代码合并 (3→1)

```
当前 3 份设计审查代码:
  nt_io_design_review/     (副本1)
  nt_shield_design_review/ (副本2)
  nt_shield/inner_critic/  (原始)

合并方案:
  → mind/perception/design/pattern_vision.rs (反模式检测)
  → mind/reasoning/design/critic.rs (设计批评)
  删除其他 2 副本, 留下 mind/ 下的唯一 canonical 来源
```

### 7.2 KnowledgeSource 变体 (5 个新增)

向两个 KnowledgeSource enum 添加:

```
KnowledgeSource 新增:
  Geist                  ← Vercel Geist 设计范式
  VisualTasteLab         ← visual-taste-lab SKILL
  DecantrDesign          ← decantr-design SKILL (guard rules)
  UIReasoning            ← ui-reasoning SKILL (11 维度)
  UISkill                ← ui-skill SKILL (5 modes)
```

---

## 8. 与现有子系统的边界

| 子系统 | 关系 | 边界 |
|--------|------|------|
| nt_core_design_token | 共存 | 现有=视频渲染 token, 新=UI 设计 token。不重叠。 |
| GWT | 消费者 | 设计推理结果经 GWT 广播进入意识 |
| E8 | 消费者 | 分配 8 态 (57-64) 给设计推理 |
| HyperCube | 存储 | 命名空间 `design.*` 分区 |
| SEAL | 消费者 | 新增 design_quality 评估维度 + taste_prototype 进化 |
| InnerCritic | 扩展 | 新增设计批评通道 (非 LLM, 基于设计规则) |
| FileIndex | 无关 | 设计文件索引走通用 FileIndex |

---

## 9. 优先级路线图

```
Phase 0 (立即):
  ├─ 新增 KnowledgeSource 变体 (5 个: Geist, VisualTasteLab, DecantrDesign, UIReasoning, UISkill)
  ├─ 合并 3 份设计审查代码为 1 份 canonical 来源
  └─ 写入本文档

Phase 1 (短程):
  ├─ mind/reasoning/design/ 创建 8 个 E8 设计态骨架
  ├─ mind/perception/design/ 创建 5 个视觉柱骨架
  └─ DESIGN_EVALUATE → E8 态 57: WCAG + 反模式检测

Phase 2 (中程):
  ├─ mind/memory/design/ 命名空间创建 (HyperCube "design.*")
  ├─ mind/reasoning/design/ 完整推理管道 (SELECT→TOKENIZE→SYSTEMIZE→COMPOSE→CRITIQUE→ALIGN)
  └─ body/design/token_gen.rs: DTCG JSON + CSS var 输出

Phase 3 (长程):
  ├─ mind/evolution/design/: SEAL 设计品味进化
  ├─ taste_prototype 自动吸收外部设计系统范式
  └─ body/design/preview_gen.rs: HTML 预览生成
```

---

## 10. 架构原则 (设计意识专属)

1. **设计即感知** — 所有设计判断始于视觉感知列, 不凭空推理
2. **VSA 统一** — 设计 token/模式/品味/原型均以 4096 维 VSA 表征
3. **品味可进化** — SEAL 管道包含设计维度, 品味原型可迭代
4. **系统优先** — 先 token 系统, 后组件, 再页面 (不可逆)
5. **不重复造轮** — 吸收外部范式 (Geist/Radix/Tailwind) 而非自创
6. **反模式检测自动化** — 每个设计输出必须经过 critic 门控
7. **一步一 token** — 设计推理每一步产出一个 VSA token 或 token 集, 不跳过层级
8. **口吻即设计** — voice & content 指南作为设计系统的一等公民 (Geist 范式)
9. **宽色域原生** — 支持 sRGB + P3 (oklch) 双通道输出
