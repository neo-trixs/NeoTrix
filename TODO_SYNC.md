# NeoTrix 多 Session TODO 同步标准
> 代数视角：每个 session 是向量空间 V_s，同步 = 并集 + 去重 + 排序
> 最后更新：2026-04-29
> 自动同步脚本：`scripts/sync_todos.py`

## TODO 项标准格式（YAML front matter + Markdown）

每个 TODO 项必须包含：
```yaml
---
id: S-09                                    # 唯一标识（项目级唯一）
title: CapabilityVector 相似度去重           # 简短描述
priority: high|medium|low                    # 优先级
status: pending|in_progress|done|blocked  # 状态
session: ses_223ff7776ffewnDVQh54z0f48C    # 来源 session ID
session_name: S-09 CapabilityVector        # 人类可读 session 名
created: 2026-04-29T10:00:00+08:00       # 创建时间
updated: 2026-04-29T15:30:00+08:00       # 最后更新
subagent: ses_223ff7776ffewnDVQh54z0f48C    # 子代理 ID（如有）
files:                                    # 涉及文件
  - src/neotrix/reasoning_brain/core.rs
  - src/neotrix/reasoning_brain/self_iterating.rs
depends_on: []                             # 依赖的其他 TODO ID
blocked_by: []                            # 被阻塞的 TODO ID
---

## 描述（Markdown）

具体任务描述...

### 完成标准
- [ ] 标准 1
- [ ] 标准 2

### 进度
- 2026-04-29 10:00: ses_xxx 启动
- 2026-04-29 15:30: ses_xxx 完成 50%
```

## Session 元数据（用于溯源）

每个 session 记录：
```yaml
# sessions/metadata/{session_id}.yml
session_id: ses_223ff7776ffewnDVQh54z0f48C
session_name: S-09 CapabilityVector
started_at: 2026-04-29T10:00:00+08:00
last_active: 2026-04-29T15:30:00+08:00
status: active|idle|completed
created_todos: [S-09, S-12, ...]      # 本 session 创建的 TODO
completed_todos: [S-12, ...]          # 本 session 完成的 TODO
subagents:                             # 启动的子代理
  - id: ses_223ff7776ffewnDVQh54z0f48C
    task: S-09 CapabilityVector
    status: in_progress
```

## 去重规则

1. **ID 完全相同** → 合并为一个，保留最新 `updated` 时间戳
2. **标题相似度 > 0.8**（Levenshtein）→ 合并，生成新 ID（取第一个）
3. **跨 session 相同文件修改** → 标记为 `potential_conflict: true`

## 优先级排序

$$Priority = w_1 \cdot P_{priority} + w_2 \cdot P_{session_age} + w_3 \cdot P_{dependency}$$

其中：
- $P_{priority}$: high=3, medium=2, low=1
- $P_{session_age}$: 越新越高（1 - age_hours / 24）
- $P_{dependency}$: 被依赖数 × 0.5

## 同步触发条件

1. **手动触发**：`python scripts/sync_todos.py`
2. **Session 结束前**：自动运行（集成到 session end hook）
3. **文件变更检测**：核心文件变更时自动检查相关 TODO

## 输出文件

同步后生成：
- `TODO.md`：人类可读的统一列表（已存在）
- `TODO.yml`：机器可读的 YAML（供脚本解析）
- `sessions/metadata/`：各 session 元数据目录
- `TODO_CONFLICTS.md`：冲突报告（如有）

---
*本文件定义同步标准，不解决问题本身*
