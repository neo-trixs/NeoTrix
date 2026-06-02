---
name: self-iteration-agent
description: Full-stack project iteration protocol — multi-source ingestion, tier absorption, auto orchestration
origin: NeoTrix
triggers: iterate, evolve, absorb, improve, 迭代, 进化, 吸收, 改进
condition: task:iteration
---

# Self-Iteration Agent — 项目自迭代协议

自动编排整个项目的迭代流程：从外部仓库吸收 → 内部模块演进 → 全量同步。

## 工作流引擎

```
[1] 任务到达 → 解析意图 → 查询 TODO 队列
    │
    ▼
[2] 并行执行阶段 ──┬── 外部分析（URLs → README → 特征矩阵）
                   ├── 内部扫描（TODO.md → 代码真实状态 → 差距分析）
                   └── 上下文加载（AGENTS.md / SOUL.md / USER.md）
    │
    ▼
[3] 评估排队 ──┬── 按 Tier 排序（Tier1 > Tier2 > ...）
               ├── 按依赖关系（无依赖的先执行）
               └── 按影响力（用户可见性 > 架构清理 > 测试）
    │
    ▼
[4] 迭代循环 ──┬── 执行子任务
               ├── cargo check --lib（必须 0 error）
               ├── TODO.md 即时同步
               ├── AGENTS.md 8 项检查
               └── 重新评估队列 → 调度下一批
    │
    ▼
[5] 闭环验证 ──┬── 编译门控（0 errors）
               ├── 测试门控（全部通过）
               ├── TODO 同步（TODO.md + TODO.yml）
               └── Session 记录（notes/session-logs/YYYY-MM-DD.md）
```

## 外部仓库吸收协议

当收到 GitHub URLs 或 "对比分析" 指令时：

### Phase 0: 前置检查
1. `cargo check --lib` 记录当前错误数（只关注新引入的）
2. 读 `TODO.md` 获取现有功能清单和进度
3. 读 `AGENTS.md` Lookup Chains 了解当前核心抽象

### Phase 1: 并行读取 + 分类
```
for each URL:
  → webfetch README (提取项目本质)
  → classify: 语言 / 核心抽象 / 与 NeoTrix 关联度
  → assign tier: 🟢Tier1 / 🔵Tier2 / 🟠Tier4 / ⛔Blocked
```

### Phase 1.5: 过滤门控
```
决定每个功能的命运：
  ├── 🟢 Tier1-2: 吸收（现有类型扩展/加方法）
  ├── 🟠 Tier4: 评估 ROI 后吸收（新建模块代价高）
  ├── 🔴 Tier5/Blocked: 跳过，记录理由到 USER.md 盲点
  └── KnowledgeOnly: 仅注入 KnowledgeSource 不实现代码
```

### Phase 2: 生成吸收计划文档
```
1. 草拟能力向量映射（各知识源的 capability_vector 细节）
2. 定义种子知识（ReasoningBank 预注入内容）
3. 分阶段分解（每个 Phase 1-2 个功能）
4. 预计 Session 数量
```

### Phase 2.5: 知识注入（预吸收）
```
1. KnowledgeSource 枚举添加新变体
2. 实现 capability_vector()（扩展维度 + provenance）
3. 定义 source_weight() 优先级
4. 注入种子知识到 ReasoningBank
5. 更新 USER.md 知识来源表格
```

### Phase 3: 特征矩阵
构建 `| 项目 | 功能 | NeoTrix 状态 | 源码位置 | Tier |`

### Phase 4: 迭代吸收
```
Phase N: 取 1-2 个最高 Tier 功能
  → 先在 KnowledgeSource 注册变体
  → 阅读 NeoTrix 对应模块代码
  → 写实现代码
  → cargo check --lib（0 new errors）
  → cargo fix --lib（自动修复警告）
  → 更新 TODO.md + TODO.yml
  → 更新特征矩阵状态列 → "✅"
  → 下一 Phase
```

### Phase 5: 验证 + 同步
```
1. cargo check --lib: 0 errors（排除预存）
2. 全部新增测试通过
3. USER.md 知识来源表格更新
4. TODO.md 特征矩阵完成标记
5. Session 日志记录
```

### Phase 6: 差距重新分析
```
1. 更新特征矩阵 "NeoTrix 状态" 列
2. 评估剩余覆盖率是否达标
3. 决定是否吸收更多或关闭该批次
```

### Tier 分类标准
| Tier | 含义 | 示例 | 并行度 |
|------|------|------|--------|
| 🟢 Tier1 | 现有类型扩展（+enum/+field） | 加个枚举变体、加个字段 | 🚀 可并行 |
| 🔵 Tier2 | 现有模块加方法 | 加函数、impl 块 | 🚀 可并行 |
| 🟠 Tier4 | 新模块，架构兼容 | 新建 .rs 文件 + mod 注册 | ⛓️ 需串行 |
| ⛔ Blocked | 架构冲突/领域不匹配 | 硬件/机器人等 | ❌ 跳过 |

## 内部模块迭代协议

### 从 TODO.md 执行任务时

1. **验证真实状态**：每个 `[ ]` 任务先用 `ls` / `grep` 确认代码是否真的不存在（35% 的 TODO checkbox 已过时）
2. **优先执行**：C-Series(新功能) > R-Series(重构) > I-Series(集成) > D-Series(桌面) > X-Series(分发)
3. **每次修改后**：`cargo check --lib` 必须 0 new errors

### 多 Agent 编排

```
TODO 队列 → SelfIteratingBrain.assess_queue()
  → 识别独立子任务组（Group 1, Group 2, ...）
  → 并行执行独立组
  → 等待所有组完成
  → 合并结果 + 解决冲突
  → 更新 TODO + cargo check
```

## 质量门控

每步必须通过以下检查：

```
□ cargo check --lib: 0 errors
□ AGENTS.md 8 项检查全部完成
□ TODO.md 已同步
□ Session 记录已写入
□ 所有新增代码有完整类型注解
□ 所有 use 语句已验证
□ 未使用变量已加下划线前缀
```

## 自迭代闭环

整个协议自身也遵循迭代原则：

```
每完成一个模块迭代后：
  → 记录该模块的迭代模式到 skills/
  → 更新 AGENTS.md 的"常见错误模式"表
  → 更新 USER.md 的"盲点"章节
```

## 启动命令

```bash
# 完整自迭代
# 1. 吸收外部仓库
cargo check --lib

# 2. 执行 TODO
# cargo check --lib（每次修改后）

# 3. 同步文档
# TODO.md → AGENTS.md → skills/
```
