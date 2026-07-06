# NeoTrix CLI Fusion Design — 自我进化型 CLI

> 2026-05-28 | 基于竞品分析: Claude Code / OpenCode / Codex CLI / Aider / Cursor / Windsurf

## 战略定位

**口号**: 首个能自我进化的 AI 编码 CLI

**差异化**:
| 维度 | 竞品 (Claude Code 等) | NeoTrix |
|------|----------------------|---------|
| 自我进化 | ❌ 固定能力 | ✅ SEAL 循环自动改进能力向量 |
| 元认知 | ❌ 不知道自己不知道 | ✅ 觉醒度量 + 自我模型 |
| 自主目标 | ❌ 仅单指令 | ✅ GoalLoop 24/7 持续目标追求 |
| 知识挖掘 | ❌ 仅依赖用户输入 | ✅ 自动爬取/分析/吸收外部知识 |
| 安全内置 | ❌ 可选沙箱 | ✅ Guardrails + Permissions + Vault |
| 多Agent | ✅ 子代理 | ✅ AgentTeam + Orchestrator + 协议发现 |
| 架构 | 固定 | ✅ CapabilityVector + HyperCube + GWT |

## 架构设计 — 统一命令系统

### 当前问题
1. 两套平行命令系统: `cli/commands/mod.rs` (带 brain 引用) + `cli/commands/builtin.rs` (无 brain 引用)
2. 16 个注册命令中 13 个返回硬编码字符串
3. CLI 缺少文件操作、git、搜索等基础能力

### 统一方案

**合并为单一 `ReplCommand` trait**:
```rust
pub trait ReplCommand: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str>;
    fn description(&self) -> &str;
    fn execute(&self, ctx: &CmdContext) -> CommandOutput;
}
```

**CmdContext** 携带所有共享状态:
```rust
pub struct CmdContext<'a> {
    pub brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    pub skills: Option<&Arc<RwLock<SkillsEngine>>>,
    pub mcp: Option<&Arc<RwLock<McpRegistry>>>,
    pub hooks: Option<&HookRegistry>,
    pub goal_loop: Option<&mut GoalLoop>,
    pub work_dir: &'a Path,
    pub output_format: OutputFormat,
}
```

### 命令分类

| 分类 | 命令 | 优先级 | 当前状态 |
|------|------|--------|----------|
| **信息** | help, stats, version, status, awakening | P0 | 3/5 mocked |
| **进化** | evolve, absorb, self-improve, knowledge-mine | P0 | 2/4 mocked |
| **记忆** | mem, recall, cortex, history | P1 | 3/4 mocked |
| **文件** | file read, file write, file edit, file diff | P1 | 全新 |
| **Git** | git status, git diff, git commit | P1 | 全新 |
| **搜索** | grep, glob, semantic-search | P1 | 全新 |
| **会话** | session list, session switch, session save | P1 | 全部 mocked |
| **Agent** | agent create, agent list, agent team | P1 | 全部 mocked |
| **MCP** | mcp list, mcp register, mcp status | P2 | 全部 mocked |
| **目标** | goal start, goal status, goal history | P2 | 全部 mocked |
| **配置** | config show, config set | P2 | 全部 mocked |
| **进化独有** | self-improve (/si), knowledge-mine (/km), awakening (/awa), cortex, hypercube | P0 | 部分真实 |

### 核心接口

```rust
#[derive(clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
    // ...global flags
    
    /// 交互模式 (默认)
    #[arg(long, short = 'i')]
    interactive: bool,
    
    /// 一次性执行 (类似 codex exec 或 aider -m)
    #[arg(long, short = 'p')]
    prompt: Option<String>,
    
    /// 后台模式 (类似 claude --bg)
    #[arg(long)]
    background: Option<String>,
    
    /// 恢复会话 (类似 claude -r)
    #[arg(long, short)]
    resume: Option<String>,
    
    /// JSON 输出
    #[arg(long)]
    json: bool,
    
    /// 追加文件到上下文 (类似 aider /add)
    #[arg(long)]
    add: Vec<String>,
}
```

### 进化独有功能 (竞品没有的)

1. **`/self-improve`** — 触发元认知自检: WeaknessAnalyzer 扫描 → 检测弱点 → 生成改进计划 → 执行 → 验证
2. **`/knowledge-mine <url>`** — 从外部知识源自动挖掘: 抓取 → 分析 → 分类 → 吸收到能力向量
3. **`/awakening [--json]`** — 显示觉醒度量 (Φ, FCS, USK, awakening_speed)
4. **`/cortex [--detail]`** — 显示 7 维知识链 + 建议
5. **`/evolve [--seal]`** — 触发完整 SEAL 自迭代循环
6. **`/hypercube`** — 显示 VSA 知识超立方体状态
7. **`/cognitive-map`** — 认知映射可视化

## 实现计划

### Phase 1 (P0): 修复模拟命令 → 真实后端
- `/stats` → `SelfIteratingBrain.get_statistics()`
- `/absorb` → `ReasoningBrain.absorb_batch()`
- `/evolve` → SEAL loop 调用
- `/mem` → `ReasoningBank.retrieve_relevant()`
- `/save` → `ReasoningBrain.save()`
- `/awakening` → `SelfMeasure.generate_report()`
- `/config show` → 读取 brain.config

### Phase 2 (P1): 新增基础能力
- `/file read/write/edit/diff` — 基于 `agent/tools/patch.rs`
- `/git status/diff/commit` — 调用 git CLI
- `/grep <pattern>` — 基于 ripgrep 集成
- `/session list/switch/new/save/resume` — 基于 `TuiApp::Session`

### Phase 3 (P2): 进化独有
- `/self-improve` — WeaknessAnalyzer → EvolutionPlanner → 执行
- `/knowledge-mine` — URL 抓取 → 分析 → 吸收
- `/cognitive-map` — 认知映射可视化

## 技术细节

### Mock → Real 映射表

| 命令 | 当前行为 | 目标行为 | 后端API |
|------|---------|---------|---------|
| `/stats` | 硬编码 "23维" | brain stats 实时数据 | `brain.get_statistics()` |
| `/absorb` | 打印消息 | `absorb_batch(&[source])` | `brain.absorb_batch()` |
| `/evolve` | 打印消息 | `brain.iterate(TaskType)` | `SelfIteratingBrain::iterate()` |
| `/mem` | 硬编码 "12条" | `bank.retrieve_relevant()` | `ReasoningBank::retrieve_relevant()` |
| `/save` | 打印 "💾" | `brain.save()` | `brain.save()` |
| `/config show` | 硬编码值 | `brain.config` 实时读取 | `brain.learning_rate` 等 |
| `/awakening` | 部分真实 | 完整报告 + `--json` | `SelfMeasure.generate_report()` |
| `/goal` | 打印消息 | `GoalLoop::pursue_all()` | GoalLoop 方法 |
| `/session` | 打印消息 | 读取 TuiApp sessions | `TuiApp::sessions` |
| `/agent` | 打印消息 | `AgentTeam` 真实操作 | `AgentTeam::add_agent()` |
| `/mcp` | 打印消息 | `McpRegistry` 真实查询 | `McpRegistry::list_servers()` |
| `/trace` | 模拟树 | 真实推理轨迹 | `ReasoningEngine::traces` |
| `/avatar` | 打印消息 | Avatar 真实管理器 | avatar_channel 模块 |

### 新命令设计

```
/file read <path>        — 读取文件内容
/file write <path> <text> — 写入文件
/file edit <path> <old> <new> — 精确替换 (基于 patch.rs)
/file diff <path>        — 显示文件 git diff

/git status              — git status 简略
/git diff [path]         — git diff 文件
/git commit <msg>        — git commit + 自动生成消息
/git log [n]             — 最近 n 条提交

/grep <pattern> [path]   — ripgrep 搜索
/glob <pattern>          — glob 文件查找
/semantic <query>        — 语义搜索 (基于 embedding)

/self-improve            — WeaknessAnalyzer → 改进
/knowledge-mine <url>    — URL 抓取 → 分析 → 吸收
```

### 输出格式标准

所有命令支持:
- 默认: 彩色表格/树形输出
- `--json`: 完整 JSON 输出 (脚本化)
- `--quiet`: 仅成功/失败状态码
