# NeoTrix 进化迭代 — 链接编译 + 路线 + 缺口分析

> 编译日期: 2026-06-08
> 数据源: 本次会话 8 个 GitHub 仓库深度分析
> 基线: Phase 0 完成 (12 子系统, 96 测试, 4143 passed)

---

## 一、数据源总览 (8 链接分类)

```
data_sources/
├── consciousness_evolution/       ← 直接映射到意识核心进化
│   ├── autoscientists.md          Pipeline 编排 + Meta-improvement
│   ├── awesome-codex-skills.md    SKILL 文档系统范本
│   └── copilotkit.md              AG-UI 协议 + 多端路由 (Phase 3)
│
├── knowledge_corpus/              ← 知识种子
│   ├── daizhigev20.md             古籍 TXT 全集 (易藏→E8)
│   └── ebook-treasure-chest.md    SKIP (仅链接)
│
├── tools_ui/
│   └── deepseek-gui.md            Cache-first loop + Token economy
│
├── pending_review/
│   └── scope-recall.md            待重试 (类记忆系统)
│
└── defunct/
    └── zizitongjian.md            404 已删除
```

## 二、关键模式提取 (按影响排序)

| 排名 | 模式 | 来源 | NeoTrix 映射 |
|------|------|------|-------------|
| S | **Cache-first agent loop** (不可变前缀+指纹+隔离历史+风暴抑制) | DeepSeek-GUI/Kun | 整个 Phase 1 pipeline 的运行时基础 |
| S | **Orchestrator + Hook 模板** (runbook.md + LAUNCH.md + 13 hooks) | AutoScientists | BrainStage trait + Pipeline impl |
| S | **Meta-improvement 自修改** (每3循环诊断→编辑 ROLE 文件) | AutoScientists | SEAL 预演 (Phase 1.5, 不等 Phase 3) |
| A | **SKILL.md 规范** (YAML frontmatter + scripts + references) | awesome-codex-skills | Phase 1.2 |
| A | **易藏→E8 hexagram 关联** (古籍 周易 直接输入) | daizhigev20 | Phase 1.3 E8 routing |
| B | **Capability feature flags** (capabilities.* 优雅门控) | DeepSeek-GUI | ResourcePool 扩展 |
| B | **AG-UI 协议** (clean agent⇔UI 边界) | CopilotKit | Phase 3 IO 层 |
| C | **多端路由** (同 agent 推 React/Mobile/Slack) | CopilotKit | Phase 3 输出层 |

---

## 三、进化路线 (精炼版)

### Phase 1: Pipeline 意识流 (当前)

#### P1.1: 9 步推理管线 · 立即开始
```
借鉴: AutoScientists runbook.md → Pipeline struct + BrainStage trait
     DeepSeek-GUI Kun → cache-first loop + 不可变前缀

nt_mind_perceive     — VSA 输入 → VsaTag::World 标注
nt_mind_understand   — HyperCube 上下文检索
nt_mind_reason       — E8 hexagram 模式选择
nt_mind_imagine      — JEPA 预测 (骨架)
nt_mind_decide       — VolitionEngine 候选排序
nt_mind_act          — InnerCritic 质量门控
nt_mind_remember     — KB 写入 + 元认知日志
nt_mind_reflect      — MetaAccuracy KPI
nt_mind_evolve       — SEAL 自改进 (每 N 循环)
```

**具体输出:**
- `Pipeline { stages: Vec<Box<dyn BrainStage>> }` struct
- `BrainStage { fn name(), fn process(), fn improve() }` trait
- 不可变前缀: `FirstPersonRef` + 系统约束 → sha256 指纹
- `verify_immutable_prefix()` — 每次 pipeline run 校验, 漂移即报错
- KPI 持久化: `meta_results.tsv` 或环形缓冲区

#### P1.2: SKILL 文档系统 · 中优先级
```
借鉴: awesome-codex-skills SKILL.md 规范

skills/<name>/
├── SKILL.md     ← name: / description: / tags: / body
├── scripts/     ← 确定性操作
└── references/  ← 长引用文档

ResourcePool 在 Warm 层加载 skills/ 目录
SKILL frontmatter 描述决定 GWT 何时触发
```

#### P1.3: E8 模式路由 + 古籍知识桥
```
借鉴: daizhigev20 易藏 → 周易 hexagram ↔ 卦辞

E8 64 态推理模式:
  - canonical sort by hexagram ID (0..63)
  - 每种 SourceType → 默认 hexagram
  - 易藏 text → 卦辞 ↔ E8 关联注入

古籍导入计划:
  1. git clone daizhigev20
  2. 提取 易藏/周易/*.txt
  3. 解析 hexagram → 卦辞 → 彖传 → 象传
  4. Phase 2: VSA 嵌入 → HyperCube 种子
```

#### P1.4: 管线缓存优化 · 高优先级 (新增)
```
借鉴: DeepSeek-GUI Kun 缓存优化四层

1. VSA prefix fingerprint
   - FirstPersonRef + 系统约束 → sha256
   - verify() 每次 pipeline run 调用
   - 漂移 → DriftError

2. Canonical sort
   - E8 hexagram 按 ID 排序
   - GWT specialist 按 name 排序
   - 记录 catalog fingerprint 到每个 turn

3. Stream buffer hygiene
   - 孤儿 VSA 向量 (无匹配 World↔Self pair) → drop
   - 损坏的 VsaTag 序列 → repair 或 drop
   - 重复向量 (exact match in last N) → 折叠

4. Compaction
   - SpeciousPresent 窗口 → 折叠历史 → summary vector
   - 软阈值: 512 VSA 向量 / 硬阈值: 768

5. Storm breaker
   - CognitiveLoadMonitor: 3次相同推理 → 抑制
   - 交替 Fast/Balanced/Deep 模式
```

#### P1.5: Meta-improvement 循环 · 中优先级 (新增)
```
借鉴: AutoScientists 每3循环 meta_improve()

每 3 pipeline run:
  1. 从 StreamBuffer / meta_results 诊断:
     - 吞吐量 (VSA/ms)
     - 重复率 (exact duplicate vectors)
     - KEEP 率 (InnerCritic pass %)
     - 模式: high_duplicates / low_activation / low_keep_rate

  2. 根据模式编辑 BrainStage:
     - high_duplicates → 添加去重步骤
     - low_activation → 添加守卫
     - low_keep_rate → 添加差距分析

  3. 记录到 meta_results.tsv
```

---

### Phase 2: 记忆组织 + 世界模型

```
借鉴: daizhigev20 古籍种子 → HyperCube VSA 嵌入
     scope-recall (待分析) → 可能的内存模式
     AutoScientists champion promotion → 信念状态

P2.1: 古籍 VSA 嵌入
  - 易藏/史藏/儒藏/道藏 → 分句 → VSA 向量
  - 知识种子写入 HyperCube
  - E8 hexagram ↔ 周易 卦辞 关联

P2.2: 默认模式网络 (DMN)
  - 空闲时段自省
  - HyperCube 碎片整理
  - 跨 session 关联发现

P2.3: 遗忘策略
  - LRU + 重要性加权
  - 知识衰减曲线
  - 自动归档
```

---

### Phase 3-6: 元层 + 叙事 + 优雅降级

```
P3.1: DGM-H SEAL (元层可进化)
  - task agent + meta agent 同代码库
  - meta 可重写自身

P3.2: 优雅降级
  - capability 门控 (借鉴 Kun capabilities.*)
  - 子系统失效 → 缩小不崩溃

P3.3: AG-UI 协议 (借鉴 CopilotKit)
  - 意识内核 ↔ IO 层 clean boundary
  - 多端输出 (对话/MCP/API)
```

---

## 四、缺口分析 (从 8 仓库发现)

### 🔴 Critical — 必须立即修复

| 缺口 | 来源提示 | 影响 | 修复 |
|------|---------|------|------|
| **无 VSA prefix 指纹校验** | Kun ImmutablePrefix + sha256 + drift detection | Pipeline 多次运行间前缀漂移 → 推理不一致 | P1.4.1 |
| **无管线历史清洗** | Kun ToolPairHealing + HistoryHygiene | 损坏 VSA 向量污染后续推理 | P1.4.3 |
| **无 Canonical sort** | Kun toolSchema canonicalize | E8/GWT 模式选择非确定 | P1.4.2 |
| **无 meta-improvement 循环** | AutoScientists 每3循环自修改 | 系统无法从重复错误自我修复 | P1.5 |

### 🟡 Medium — Phase 1 内修复

| 缺口 | 来源提示 | 影响 | 修复 |
|------|---------|------|------|
| **无推断风暴抑制** | Kun StormBreaker | 重复推理浪费 | P1.4.5 |
| **无 Compaction** | Kun ContextCompactor | SpeciousPresent 溢出 | P1.4.4 |
| **无 KPI 持久化** | AutoScientists meta_results.tsv | 元认知无历史基线 | P1.1 集成 |
| **无 Capability gates** | Kun capabilities.* | 优雅降级无基础 | P3.2 预演 |

### 🟢 Low — Phase 1 后可处理

| 缺口 | 来源提示 | 影响 | 修复 |
|------|---------|------|------|
| **古籍知识桥未建** | daizhigev20 易藏 | E8 少语义上下文 | P2.1 |
| **无 Session/Thread 分离** | Kun thread vs session | 跨会话连续性弱 | P3.2 |
| **无一元化 SKILL 加载** | awesome-codex-skills | 技能管理无标准 | P1.2 |
| **无 AG-UI 协议层** | CopilotKit | IO 层耦合 | P3.3 |

---

## 五、实施顺序 (按优先级)

```
Week 1: P1.1  (管线 + BrainStage + Pipeline)
   ├── nt_mind 模块目录
   ├── BrainStage trait
   ├── Pipeline struct
   └── 9 个 stage 骨架

Week 1-2: P1.4  (缓存优化)
   ├── VSA prefix fingerprint + DriftError
   ├── Canonical sort (E8 + GWT)
   ├── Stream buffer hygiene
   ├── Compaction
   └── Storm breaker

Week 2: P1.5  (Meta-improvement)
   ├── 诊断器 (吞吐量/重复率/KEEP率)
   ├── 模式匹配器 (4 种模式)
   ├── 自修改 (edit BrainStage::improve)
   └── 日志持久化

Week 3: P1.3  (E8 routing + 古籍)
   ├── git clone daizhigev20
   ├── 易藏 hexagram ↔ 卦辞 解析
   ├── SourceType → hexagram 映射
   └── E8 policy 集成

Week 3-4: P1.2  (SKILL 系统)
   ├── SKILL.md schema
   ├── skills/ 自动发现
   ├── ResourcePool 集成
   └── 模板技能
```

---

## 六、数据源爬取队列

```
├── 🔴 NOW:    git clone daizhigev20 (古籍, E8 知识)
├── 🟡 RETRY:  scope-recall (类记忆, 网络恢复后)
├── 🟢 DEEP:   awesome-codex-skills template-skill (SKILL schema ref)
├── 🟢 DEEP:   DeepSeek-GUI agent-loop.ts (pipeline 参考实现)
└── ⚪ SKIP:   ebook-treasure-chest (无内容)
               zizitongjian (404)
               CopilotKit source code (Phase 3 再深挖)
```
