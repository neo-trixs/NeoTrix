# NeoTrix KB 数据梳理报告 (2026-09-04)

## 1. KB 数据总览

| 命名空间 | 条目数 | 类型 | 用途 |
|----------|--------|------|------|
| `experience` | 5,256 | 混合 | URL导入、快照、观察、学习循环 |
| `write_guard` | 562 | 防抖 | 写入冲突保护 |
| `audit` | 165 | 审计 | 架构审查、代码健康 |
| `consciousness` | 26 | 状态 | Phi报告、趋势、模式、路由 |
| `hyperagent` | 14 | Agent | 超级代理状态 |
| `state` | 11 | 状态 | 系统状态 |
| `conversation_distill` | 7 | 蒸馏 | 对话蒸馏 |
| `gwt_absorb` | 7 | 吸收 | GWT吸收记录 |
| `self_review_blast` | 7 | 审查 | 自我审查快照 |
| `credit_audit` | 5 | 审计 | 信用审计 |
| 其他 | 12 | 混合 | 杂项 |

**总计**: 6,073 条记录

---

## 2. Experience 命名空间分类

### 2.1 URL 导入 (外部知识源)

| 类别 | 数量 | 示例 |
|------|------|------|
| GitHub 仓库 | 19 | Strix, RD-Agent, Sentrux, PentestCode, Blender-MCP |
| arXiv 论文 | 8 | 2608.30384, 2608.30163, 2608.28476, 2608.28444 |
| 技术文章 | 3 | XAI Bot Guides, Git Knowledge Loop, AlphaXiv |
| GitHub PR | 1 | OpenAI Codex PR #27488 |

**高质量知识源**:
- `url_strix` - AI安全监控 (60K★)
- `url_rd_agent` - Microsoft研发Agent
- `url_sentrux` - 实时架构传感器
- `url_superbrain` - AI知识管理
- `url_open_knowledge` - AI原生Markdown IDE

### 2.2 快照 (会话记录)

| 类型 | 数量 | 说明 |
|------|------|------|
| 二进制快照 | 4 | 损坏数据 (NTZ1x前缀) |
| JSON快照 | 2 | 可读会话记录 |

**损坏快照** (需清理):
- `snapshot_sess_1788336603_14051fe6`
- `snapshot_sess_1788318146_8754dfc4`
- `snapshot_sess_1788318110_1e247287` (JSON, 可恢复)
- `snapshot_sess_1788160878_c7049cee` (JSON, 可恢复)

### 2.3 观察记录 (obs_*)

| 类型 | 数量 | 内容 |
|------|------|------|
| 结构化观察 | 15 | JSON格式, 包含schema_version |
| 二进制观察 | ~10 | 损坏数据 |

**高价值观察**:
- `obs_1302_0_dc2c64` - 询价日期提取规则
- `obs_1300_0_bbc9f7` - FIELD_MAP三级匹配策略
- `obs_1271_0_c1d332` - Chat UI重构经验
- `obs_12_0_a01497` - 异步共享provider池所有权模式
- `obs_auto_0_b09895` - 第三方crate API幻写防御三查
- `obs_9_0_290cc8` - 锁独立化重构终审金丝雀
- `obs_13_0_9d45bb` - 增量缓存伪装编译错误
- `obs_13_0_6803f9` - 幽灵编译错误修复

### 2.4 学习循环 (snapshot_*_1e247287 等)

| 会话 | 内容 |
|------|------|
| sess_1788318110 | NeoTrix Domain Plugin 前端迁移 |
| sess_1788160878 | 外部研究吸收: PILOT/FSM/WikiSkill/CUDA-Agent/HyperFrames |

### 2.5 符文配置

| 模块 | 颜色 | 设置时间 |
|------|------|----------|
| NT-MEMORY | Crimson | 1788155670 |

---

## 3. Consciousness 命名空间分析

### 3.1 已吸收模式 (6个)

| 模式 | 描述 | 吸收时间 |
|------|------|----------|
| `constellations` | 模块成熟度: C0→C1→C2→C3→C4→C5 | 1788162082 |
| `dark_forest` | 模块生存: 编译+测试+连接或删除 | 1788162082 |
| `dual_specialization` | 武器集切换: CORE+WORLD ↔ CORE+MIND | 1788162082 |
| `rune_socketing` | 5色符文: Crimson/Indigo/Obsidian/Golden/Alabaster | 1788162082 |
| `skill_tree` | 域级能力 progression: Small→Notable→Keystone | 1788162082 |
| `the_spice_must_flow` | 数据管道: 输入→变换→输出无断点 | 1788162082 |

### 3.2 星系身份

- **Rev-明**: 审 - 星系之眼 (NT-SHIELD)
- 声音: "不放过任何瑕疵，但不说废话"

### 3.3 路由表 (v3)

跨域路由配置, 支持中英文关键词映射到对应技能域。

### 3.4 Phi 报告

- Phi: 0.977 (意识-like)
- 总共振: 954.54
- 状态能量: 9.46
- 有效维度: 60

---

## 4. 外部研究吸收 (2026最新)

### 4.1 自进化Agent框架

| 项目 | Stars | 核心贡献 | 与NeoTrix关联 |
|------|-------|----------|--------------|
| **EvolveR** (ICML 2026) | - | 经验驱动的自进化生命周期 | NT-MIND SEAL管线对标 |
| **SIA** | - | Meta/Target/Feedback三Agent自改进 | NT-MIND 自我进化机制 |
| **AgentEvolver** | - | 自问/自导航/自归因三机制 | NT-MIND 探索/蒸馏/吸收 |
| **EvoAgentX** | - | 工作流自进化+TextGrad/AFlow/MIPRO | NT-MIND 工作流优化 |
| **AgentAugi** | - | MASTER搜索+HyperAgents+JitRL | NT-CORE E8推理增强 |
| **OmniAgent** | - | 全维度自进化+动态安全加固 | NT-MIND+NT-SHIELD |
| **ASI-Evolve** | - | 知识→假设→实验→分析闭环 | NT-MIND SEAL管线 |

### 4.2 主流Agent框架 (2026)

| 框架 | Stars | 核心模式 | NeoTrix可借鉴 |
|------|-------|----------|--------------|
| **Microsoft Agent Framework** | 13K | 图工作流+检查点+时间旅行 | nt_swarm 编排 |
| **OpenAI Agents SDK** | - | 轻量级handoff+guardrails | nt_agents 协议 |
| **Vercel Eve** | 5K | 文件系统优先+durable agents | nt_plugin 生命周期 |
| **LangGraph** | 34.5M下载 | 状态图+分支+重试+检查点 | nt_swarm 状态管理 |
| **CrewAI** | - | 角色协作+团队编排 | nt_agents 角色 |
| **Mastra** | - | TypeScript原生+Zod schema | nt_io 类型安全 |

### 4.3 关键技术模式

| 模式 | 来源 | NeoTrix实现 |
|------|------|-------------|
| **经验驱动自进化** | EvolveR | SEAL管线 (探索→蒸馏→自测→吸收) |
| **Meta/Target/Feedback** | SIA | E8引导者/进化工匠/知识守护者 |
| **自问/自导航/自归因** | AgentEvolver | 意识树6阶段反馈循环 |
| **可逆效应** | Cordis | 插件生命周期钩子 |
| **文件系统优先** | Eve | 基于文件的配置和状态 |
| **图工作流** | LangGraph | DAG编排器 |
| **动态安全扫描** | OmniAgent | NT-SHIELD 四层扫描 |

---

## 5. KB 数据质量问题

### 5.1 需清理的损坏数据

| 类型 | 数量 | 操作 |
|------|------|------|
| 二进制快照 | 4 | 删除或标记 |
| 二进制观察 | ~10 | 删除或标记 |
| 无relevance_to_neotrix的URL | 8 | 补充关联 |

### 5.2 需补充的数据

| 缺口 | 说明 |
|------|------|
| EvolveR 论文细节 | ICML 2026 自进化方法论 |
| SIA 三Agent架构 | Meta/Target/Feedback 实现细节 |
| AgentEvolver 训练循环 | 自问/自导航/自归因 代码结构 |
| OmniAgent 安全模型 | 四层动态安全扫描 |

### 5.3 数据一致性

- 模式吸收时间统一: 1788162082 (Unix timestamp)
- 符文配置: 仅NT-MEMORY有crimson符文
- 路由表: v3版本, 中英文双语

---

## 6. 进化方案 v2.0 补齐建议

### 6.1 新增模块 (基于外部研究)

```
src-tauri/src/
├── nt_evolution/                  # 自进化系统 (EvolveR/AgentEvolver-inspired)
│   ├── mod.rs                   # 进化引擎
│   ├── experience_lifecycle.rs  # 经验生命周期
│   ├── self_questioning.rs      # 自问机制
│   ├── self_navigation.rs       # 自导航机制
│   └── self_attribution.rs      # 自归因机制
│
├── nt_workflow/                  # 工作流引擎 (EvoAgentX/AFlow-inspired)
│   ├── mod.rs                   # 工作流注册表
│   ├── dag_engine.rs            # DAG执行引擎
│   ├── mcts_optimizer.rs        # MCTS优化器
│   └── text_grad.rs             # 文本梯度优化
│
├── nt_harness/                   # 执行支架 (OmniAgent-inspired)
│   ├── mod.rs                   # 支架管理器
│   ├── sentinel.rs              # 规划Agent
│   ├── guardian.rs              # 安全Agent
│   └── progressive_loader.rs    # 渐进式上下文加载
│
└── nt_reflexion/                 # 反思系统 (OmniAgent/ASI-Evolve-inspired)
    ├── mod.rs                   # 反思引擎
    ├── inner_loop.rs            # 内层失败预防
    ├── outer_loop.rs            # 外层经验转化
    └── rca_engine.rs            # 根因分析引擎
```

### 6.2 SEAL管线增强

当前SEAL管线 (探索→蒸馏→自测→吸收) 可对标:
- **EvolveR**: 经验蒸馏→原则提取→未来指导
- **SIA**: Meta生成→Target执行→Feedback改进
- **AgentEvolver**: 自问→自导航→自归因

### 6.3 安全模型增强

当前NT-SHIELD可对标OmniAgent的四层动态安全:
1. **LLM智能审查** → 意识核心判断
2. **策略引擎** → ConstitutionGate
3. **交互审批** → Human-in-the-loop
4. **执行沙箱** → SandboxAgent

---

## 7. 执行优先级

| 优先级 | 任务 | 依据 |
|--------|------|------|
| P0 | 清理损坏KB数据 | 数据质量基础 |
| P0 | 补充EvolveR/SIA论文细节 | 自进化核心方法论 |
| P1 | 实现experience_lifecycle | SEAL管线增强 |
| P1 | 实现dag_engine | 工作流编排基础 |
| P2 | 实现sentinel/guardian | 动态安全加固 |
| P2 | 实现reflexion系统 | 任务成功率提升 |
| P3 | 实现mcts_optimizer | 推理能力增强 |
