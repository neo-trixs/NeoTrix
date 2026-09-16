# NeoTrix CLI (`nt`) 架构熔炼方案

> 日期: 2026-09-14 | 状态: 设计阶段 | 目标: 聚焦冗余 + 扁平缺陷 + 跨域错位

---

## 一、现状诊断

### 1.1 冗余热力图 (20 项)

| ID | 冗余类型 | 严重度 | 位置 | 修复方案 |
|----|----------|--------|------|----------|
| R1 | `/model` 与 `/provider` 功能重叠 | HIGH | model_cmds.rs:31 / provider_cmds.rs:21 | 合并为 `nt provider` 统一入口 |
| R2 | `/cost` 与 `/budget` 费用追踪重叠 | MEDIUM | cost_cmds.rs:12 / budget_cmds.rs:8 | `/budget` 降级为 `/cost budget` 子命令 |
| R3 | `/session` 与 `/session-all` 聚合器重叠 | MEDIUM | session_cmds.rs:86 / consolidated_cmds.rs:584 | 移除 `/session-all`，保留 `/session` |
| R4 | `open_kb()` 多处重复定义 | HIGH | kb_cmds.rs:10 / kanban_cmds.rs:833 / cortex_cmds.rs:23 | 提取到 `cli::commands::kb_common` |
| R5 | 子命令解析逻辑重复 | MEDIUM | 多个 cmds.rs | 创建 `SubcommandRouter` trait |
| R6 | 模型列表硬编码 | HIGH | model_cmds.rs:49-56 | 从 provider_catalog 动态加载 |
| R7 | 端口硬编码 8337 | LOW | kb_cmds.rs:1308 | 从 config 读取 |
| R8 | CORTEX_ROOT 路径硬编码 | LOW | cortex_cmds.rs:20-21 | 从 config 读取 |
| R9 | JSON 输出结构重复 | MEDIUM | 多个 cmds.rs | 创建 `JsonOutputBuilder` |
| R10 | 错误消息格式不统一 | LOW | 多个 cmds.rs | 创建 `ErrorTemplate` |
| R11 | 帮助消息生成重复 | LOW | 多个 cmds.rs | 从命令元数据自动生成 |
| R12 | 聚合器命令过多 (7个) | MEDIUM | consolidated_cmds.rs | 合并为 3-4 个域聚合器 |
| R13 | 配置读取模式不统一 | MEDIUM | 多个 cmds.rs | 统一 `ConfigReader` 接口 |
| R14 | 统计信息收集重复 | LOW | doctor_cmds.rs / pool_health_cmds.rs | 创建 `StatsCollector` |
| R15 | 子命令路由 match 重复 | MEDIUM | 多个 cmds.rs | 创建宏统一处理 |
| R16 | 参数解析函数重复 | LOW | kb_cmds.rs:55 / bench_cmds.rs:46 | 提取到 `cli_utils` |
| R17 | Mutex/RwLock 处理重复 | LOW | 多个 cmds.rs | 创建 `lock_or_recover` 宏 |
| R18 | 时间戳格式化重复 | LOW | wiki_cmds.rs:174 / plan_cmds.rs:80 | 提取到 `time_utils` |
| R19 | UUID 生成分散 | LOW | session_cmds.rs:136 / goal_cmds.rs:155 | 统一 `IdGenerator` |
| R20 | JSON 构建逻辑重复 | LOW | kb_cmds.rs:205 / chain_cmds.rs:127 | 使用统一 Builder |

### 1.2 缺陷清单 (12 项)

| ID | 严重度 | 类型 | 位置 | 描述 |
|----|--------|------|------|------|
| D1 | **P0** | expect() panic | provider_cmds.rs | `pool.lock().unwrap_or_else(\|e\| e.into_inner())` 可能 panic |
| D2 | **P0** | expect() panic | cost_cmds.rs | 同上模式 |
| D3 | **P0** | expect() panic | budget_cmds.rs | 同上模式 |
| D4 | P1 | 代码重复 | 多个 cmds.rs | 复制粘贴的样板代码 |
| D5 | P1 | R-P110 违规 | consolidated_cmds.rs | 新能力通过 CLI 命令暴露而非内部调度 |
| D6 | P1 | 死代码模块 | 某些 cmds.rs | 未被消费的命令 |
| D7 | P1 | 缺测试 | 33 个文件 | 无单元测试覆盖 |
| D8 | P2 | 潜在死锁 | 多个 cmds.rs | 嵌套锁获取 |
| D9 | P2 | stdout 直接输出 | 多个 cmds.rs | 应通过统一输出层 |
| D10 | P2 | 参数风格不一致 | 多个 cmds.rs | `--json` vs `--format json` |
| D11 | P2 | STUB 功能虚化 | 部分 cmds.rs | 命令存在但功能未实现 |
| D12 | P2 | 错误静默忽略 | 多个 cmds.rs | `let _ = ...` 吞掉错误 |

### 1.3 域对齐错位 (8 项)

| ID | 命令 | 当前域 | 正确域 | 修复方案 |
|----|------|--------|--------|----------|
| A1 | `/search` | NT-MEMORY | NT-WORLD | 移动到 NT-WORLD 域 |
| A2 | `/game` | NT-CORE | NT-CORE (E8) | 重新分类为 E8 验证 |
| A3 | `/cortex` | System | NT-MEMORY | 移动到 NT-MEMORY 域 |
| A4 | `/guard` | 未注册 | NT-SHIELD | 注册到 registry |
| A5 | SEAL pipeline | 无 CLI | NT-MIND | 添加 CLI 可观测性 |
| A6 | EmotionLabel | 无 CLI | NT-FEEL | 添加查询入口 |
| A7 | `/compact` | NT-MEMORY | 内部调度 | 改为内部能力 |
| A8 | `/context` | NT-MEMORY | 内部调度 | 改为内部能力 |

---

## 二、统一架构设计

### 2.1 `nt` CLI 三层架构

```
┌─────────────────────────────────────────────────────┐
│  L3: nt CLI (统一入口)                               │
│  ┌─────────┬──────────┬──────────┬─────────┐        │
│  │nt doctor│nt provider│nt tree  │nt kb    │        │
│  │nt run   │nt config  │nt session│nt world │        │
│  └─────────┴──────────┴──────────┴─────────┘        │
├─────────────────────────────────────────────────────┤
│  L2: Command Router (路由层)                          │
│  ┌─────────────────────────────────────────────┐    │
│  │ NtCommands enum → dispatch → execute_cli()  │    │
│  │ + legacy fallback (replay / commands)        │    │
│  └─────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────┤
│  L1: Domain Dispatchers (域调度层)                    │
│  ┌──────┬──────┬──────┬──────┬──────┬──────┐       │
│  │NT-CORE│NT-MEM│NT-WLD│NT-ACT│NT-IO│NT-SHD│       │
│  └──────┴──────┴──────┴──────┴──────┴──────┘       │
└─────────────────────────────────────────────────────┘
```

### 2.2 命令域映射表

| `nt` 命令 | NT 域 | 聚合器 | 旧命令 |
|-----------|-------|--------|--------|
| `nt doctor` | NT-CORE | - | `/doctor`, `/stats` |
| `nt provider list` | NT-IO | - | `/provider list`, `/model list` |
| `nt provider status` | NT-IO | - | `/model current` |
| `nt provider ping` | NT-IO | - | `/provider challenge` |
| `nt provider configure` | NT-IO | - | `/model set` |
| `nt tree show` | NT-CORE | - | `/e8`, `/consciousness` |
| `nt tree bud` | NT-CORE | - | (新增) |
| `nt tree link` | NT-CORE | - | (新增) |
| `nt kb search` | NT-MEMORY | - | `/kb`, `/search` |
| `nt kb query` | NT-MEMORY | - | `/kb query` |
| `nt run <task>` | NT-ACT | - | `/agent`, `/chain` |
| `nt config` | NT-IO | - | `/config` |
| `nt session` | NT-MEMORY | - | `/session` |
| `nt world` | NT-WORLD | - | `/search`, `/explore` |
| `nt shield` | NT-SHIELD | - | `/guard` |
| `nt mind` | NT-MIND | - | `/consciousness`, `/skills` |
| `nt feel` | NT-FEEL | - | (新增) |

### 2.3 实现原则

1. **R-P42 强化现有节点**: 不创建新的平行适配器，强化已有命令
2. **R-P110 内部调度**: 能力实现走内部模块，CLI 只做入口
3. **R-P82 有序后端路由**: Provider 管理支持 ordered fallback
4. **R-P79 吸收接线门**: 新能力必须同 session 接线生产

---

## 三、核心路线任务清单

### Phase 1: P0 缺陷修复 (Day 1)
- [ ] **T1.1**: 修复 D1/D2/D3 的 `expect()` → 防御性错误处理
- [ ] **T1.2**: 修复 D8 潜在死锁 → 使用 `try_lock` + 超时

### Phase 2: 冗余清理 (Day 2-3)
- [ ] **T2.1**: 合并 `/model` + `/provider` → `nt provider` (R1)
- [ ] **T2.2**: 提取 `open_kb()` 到共享模块 (R4)
- [ ] **T2.3**: 创建 `SubcommandRouter` trait (R5)
- [ ] **T2.4**: 统一 JSON 输出 Builder (R9)
- [ ] **T2.5**: 统一配置读取接口 (R13)

### Phase 3: 域对齐 (Day 4-5)
- [ ] **T3.1**: 重新分类 `/search` → NT-WORLD (A1)
- [ ] **T3.2**: 注册 `/guard` 到 registry (A4)
- [ ] **T3.3**: 添加 SEAL pipeline CLI 可观测性 (A5)
- [ ] **T3.4**: 添加 EmotionLabel 查询入口 (A6)

### Phase 4: 架构重构 (Day 6-8)
- [ ] **T4.1**: 实现 `nt` CLI 三层架构 (L3/L2/L1)
- [ ] **T4.2**: 创建域调度层 (NT-CORE/MEMORY/WORLD/ACT/IO/SHIELD)
- [ ] **T4.3**: 迁移高频命令到新架构
- [ ] **T4.4**: 保留 legacy fallback 兼容

### Phase 5: 测试与文档 (Day 9-10)
- [ ] **T5.1**: 为高频命令补充单元测试 (D7)
- [ ] **T5.2**: 创建 `nt --help` 完整文档
- [ ] **T5.3**: 运行全量回归测试

---

## 四、多 Agent 巡检修复计划

### Agent 1: P0 修复专员
- 目标: 修复 D1/D2/D3 的 expect() panic
- 范围: provider_cmds.rs, cost_cmds.rs, budget_cmds.rs
- 验证: `cargo check --lib -p neotrix`

### Agent 2: 冗余清理专员
- 目标: 提取共享代码 (R4, R5, R9, R13)
- 范围: kb_cmds.rs, consolidated_cmds.rs, types.rs
- 验证: 代码复用率提升

### Agent 3: 域对齐专员
- 目标: 修复域错位 (A1, A4)
- 范围: registry.rs, search_cmds.rs, guard_cmds.rs
- 验证: 域映射一致性

### Agent 4: 测试补充专员
- 目标: 为高频命令补充测试 (D7)
- 范围: file_cmds.rs, kb_cmds.rs, session_cmds.rs
- 验证: `cargo test -p neotrix --lib`
