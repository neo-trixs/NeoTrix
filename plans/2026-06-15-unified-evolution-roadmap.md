# NeoTrix 统一进化路线图 v4

> 日期: 2026-06-15
> 状态: 综合评估 + 可执行路线图
> 前置: 模块映射 (442K 行, 9.5K 测试)，六维文献扫描，20 项 desktop gap 分析，Ne 语言 Stage 1a 接线

---

## 1. 现状全景

### 1.1 项目规模

| 维度 | 数值 |
|------|------|
| 总行数 (workspace) | ~442,817 |
| 测试函数 | 9,575 |
| 核心子模块 | ~67 |
| 二进制目标 | 25 |
| Workspace crates | 9 |
| 最大模块 | nt_mind (61,812 行) |
| 最大子模块 | nt_core_experience (40,047 行) |
| Stub/空模块 | 8 (nt_core_epoch, nt_core_iface, agent_interface, cli_interface, server_interface, nt_core_network/mod.rs, nt_io_neotrix_interface, nt_core_code_query) |
| 零测试关键模块 | **2** (nt_core_llm 302 行, nt_core_tool 211 行) |

### 1.2 Phase 完成状态

```
Phase 0-F:  59/59 ✅ (所有已规划 Phase 完成)
Phase 55-58: 计划中 (本路线图)
```

### 1.3 代码健康

| 类别 | 状态 |
|------|------|
| `cargo check --lib` | 0 errors, 0 warnings |
| `cargo check --bin ne-dialog` | 0 errors, 0 warnings |
| 19 个已发现 bug | ✅ 全部修复 |
| LLM/Tool 模块测试 | ❌ 零测试覆盖 |
| 代码重复 | ⚠️ `hyperagent.rs` 在 `nt_core_agent/` + `nt_core_experience/` 重复 |

---

## 2. 竞争定位

### 2.1 桌面 App 对标 (20 维度)

| # | 功能 | ne-dialog | Claude Code | Codex CLI |
|---|------|-----------|-------------|-----------|
| 1 | Provider 多后端 | ✅ 4家 | ❌ Claude only | ✅ GPT + local |
| 2 | 流式输出 | ⚠️ chunked | ✅ SSE token | ✅ SSE+WS |
| 3 | 工具自动调度 | ❌ /tool 手动 | ✅ 自动 | ✅ 自动 |
| 4 | MCP 集成 | ⚠️ 发现/空壳 | ✅ 全客户端 | ✅ 全客户端 |
| 5 | 会话管理 | ✅ save/load | ✅ /compact | ✅ resume/fork |
| 6 | Diff 渲染 | ❌ 纯文本 | ✅ 语法高亮 | ✅ 语法高亮 |
| 7 | 滚动/导航 | ⚠️ up/down | ✅ 搜索+回滚 | ✅ Ctrl+R |
| 8 | 推理过程显示 | ❌ 无 | ✅ Extended thinking | ✅ /plan 模式 |
| 9 | 文件树浏览 | ❌ 无 | ✅ /cd | ✅ @ 搜索 |
| 10 | 代码搜索 | ❌ 无 | ✅ grep+glob | ✅ 内置搜索 |
| 11 | 后台终端 | ❌ 无 | ✅ ! 命令 | ✅ 子 agent |
| 12 | Git 集成 | ❌ 无 | ✅ commit/PR/review | ✅ /diff |
| 13 | LSP 集成 | ❌ 无 | ✅ lint hooks | ⚠️ MCP 桥 |
| 14 | 成本追踪 | ❌ 无 | ✅ /cost | ✅ /status |
| 15 | 审批系统 | ❌ 无 | ✅ 3 级权限 | ✅ tripwire |
| 16 | 计划面板 | ❌ 无 | ✅ /plan 模式 | ✅ /plan 命令 |
| 17 | Artifact 渲染 | ❌ 无 | ✅ Mermaid/SVG | ✅ gpt-image |
| 18 | 语音输入 | ⚠️ 引擎存在/未接线 | ✅ /voice | ✅ push-to-talk |
| 19 | 图像输入 | ⚠️ 引擎存在/未接线 | ✅ paste 截图 | ✅ @ 文件 |
| 20 | 远程主机 | ❌ 无 | ✅ SSH remote | ✅ session resume |

**差距: 18/20 缺失或部分 → ~5,500 LOC 补齐**

### 2.2 文献-代码差距 (16 新缺口)

| 优先级 | 缺口 | 来源 | 影响 |
|--------|------|------|------|
| P0 | Ed25519 签名迁移 | A2A v1.2 | 互操作性 |
| P0 | SAHOO CPS 约束评分 | SAHOO arXiv | RSI 安全 |
| P0 | MCP 2026-07-28 无状态 | MCP RC | 协议兼容性 |
| P1 | TRT 递归思考 | arXiv:2602.03094 | 推理深度 |
| P1 | DafnyPro diff-checker | arXiv:2601.05385 | safety_gate |
| P1 | LSP 集成 | Steve 项目 | 开发者体验 |
| P2 | Lean 4 核验 | IRIS | 编译器正确性 |
| P2 | Sobol 确定性投影 | XL-HD | VSA 编码 |
| P2 | 多维护者重建 | StageX | 编译器可信 |
| P3 | GHRR VSA attention | OpenReview | VSA 注意力 |
| P3 | 稀疏投影矩阵 | HyperSPACE | 能效 |
| P3 | ACF 分解器 | OpenReview | 谐振器容量 |
| P3 | Polaris 策略修复 | arXiv:2603.23129 | 小模型 |
| P3 | Rust→Lean 流水线 | arXiv:2605.30106 | 形式化 |
| P3 | AI 证明器 | Aleph/Aristotle | 验证 |
| P3 | 规约欺骗防御 | Dafny RLVR | 安全 |

### 2.3 Ne 语言 Stage 状态

| Stage | 名称 | 编译器 | 状态 |
|-------|------|--------|------|
| 0 | ne0 汇编 | Rust | ✅ stage0_seed.rs 489 行 |
| 1a | NeEvaluator | Rust eval | ✅ eval.rs 687 行, 32 测试, CI 已接线 |
| 1 | VSA 原语层 | Rust 转译 | ⏳ bridge.rs 阻塞 |
| 2 | 自托管编译器 | Ne → Rust | ❌ Stage 1 后 |
| 3 | 元循环求值器 | Ne → Ne eval | ❌ Stage 2 后 |
| 4 | VSA 原生编译 | 向量运算 | ❌ Stage 3 后 |
| ∞ | 自修改元语言 | 自生成 | ❌ 远期 |

---

## 3. 进化路线图: Phase 55-60

### Phase 55 — 桌面 App 基础层 (8 parallel)
*阻塞: 无 — 全部独立*

| 任务 | 文件 | LOC | 优先级 | 描述 |
|------|------|-----|--------|------|
| Tool 自动调度 | ne_dialog.rs + nt_core_tool | 400 | P0 | LLM 函数调用解析 → 工具执行 → 结果注入 |
| 审批系统 | ne_dialog.rs + tool_registry | 350 | P0 | --suggest / --auto-edit / --full-auto 三级 |
| Git 集成 | ne_dialog.rs | 350 | P1 | /git status/diff/commit/branch |
| 流式输出 (真 SSE) | nt_core_llm | 350 | P1 | 替换 send_prompt_blocking → send_prompt_stream |
| 会话 fork/compact | ne_dialog.rs | 200 | P1 | /fork, /compact 会话压缩 |
| Cost 追踪 | ne_dialog.rs | 200 | P2 | token 计数 + 费用统计 |
| Global 搜索 | ne_dialog.rs | 200 | P2 | 跨会话 grep + VSA 语义搜索 |
| ne-dialog 单元测试 | tests/ | 300 | P0 | 覆盖率 0→60% |

**依赖**: 所有 8 任务操作独立文件/独立方法, 可并行

### Phase 56 — 高级桌面功能 (6 parallel)
*阻塞: Phase 54 (Ne 编译器)*

| 任务 | 文件 | LOC | 优先级 | 描述 |
|------|------|-----|--------|------|
| MCP 全客户端 | nt_core_mcp + ne_dialog | 500 | P1 | 服务器生命周期 + 工具桥接 |
| Diff 渲染 | ne_dialog.rs | 300 | P2 | Shiki/语法高亮 diff |
| 文件树浏览 | ne_dialog.rs | 350 | P2 | walkdir 目录浏览 |
| 后台终端 | ne_dialog.rs | 400 | P2 | shell session manager |
| 计划面板 | ne_dialog.rs | 250 | P2 | /plan 只读探索 → 审批 → 执行 |
| Artifact 视图 | ne_dialog.rs | 400 | P3 | Mermaid/SVG/HTML 渲染 |

### Phase 57 — 核心引擎增强 (5 parallel)
*阻塞: 无 — 全部独立*

| 任务 | 文件 | LOC | 优先级 | 描述 |
|------|------|-----|--------|------|
| Ed25519 迁移 | nt_core_hive/signed_card.rs | 300 | P0 | A2A v1.2 兼容 |
| SAHOO CPS | nt_core_experience/pcc_safety.rs | 250 | P0 | 约束保留评分 |
| TRT 递归思考 | nt_core_experience/fusion_deliberator.rs | 350 | P1 | 自生成验证信号 |
| LSP 集成 | ne_dialog.rs | 250 | P1 | rust-analyzer/pyright |
| MCP 无状态适配 | nt_core_mcp | 150 | P0 | 移除 session-id |

### Phase 58 — 自举编译器冲刺 (4 dependent)
*依赖: Phase 55 完成 (bridge.rs Stage 1)*

| 任务 | 文件 | LOC | 优先级 | 描述 |
|------|------|-----|--------|------|
| bridge.rs Stage 1 | bridge.rs | 500 | P0 | Ne S-expression → Rust transpiler |
| LOOP/MATCH 完整 | eval.rs | 300 | P0 | Ne 控制流完成度 |
| SELF_SOURCE v2 | bridge.rs | 300 | P0 | 自举验证 |
| Ne std lib | eval.rs | 200 | P1 | 50+ VSA 原语方便调用 |

### Phase 59 — LLM/Tool 测试覆盖 (2 parallel)
*阻塞: 无*

| 任务 | 文件 | LOC | 优先级 |
|------|------|-----|--------|
| nt_core_llm 测试 | nt_core_llm/mod.rs | 200 | P0 |
| nt_core_tool 测试 | nt_core_tool/mod.rs | 150 | P0 |

### Phase 60 — 大模块拆分 (3 parallel)
*阻塞: 无*

| 任务 | 文件 | LOC | 优先级 |
|------|------|-----|--------|
| nt_mind 拆分 (62K→<10K) | nt_mind/ | — | P2 |
| nt_core_experience 拆分 (40K) | experience/ | — | P2 |
| nt_shield_stealth_net 精简 (19.5K) | stealth_net/ | — | P3 |

---

## 4. 实施策略

### 4.1 并行波次

```
Phase 55 (8 tasks) ─────────────────┐
Phase 57 (5 tasks) ───────────────┐ ├── 可全并行 (不同文件)
Phase 59 (2 tasks) ─────────────┐ │ │
                                │ │ │
                                ▼ ▼ ▼
Phase 56 (6 tasks) ← 依赖 Phase 55 ne-dialog 结构就绪
                                │
                                ▼
Phase 58 (4 tasks) ← 依赖 Phase 55 bridge.rs Stage 1
                                │
                                ▼
Phase 60 (3 tasks) ← 大模块重构，可在任意时间进行
```

### 4.2 量化目标

| 阶段 | 新代码 | 新测试 | 关键指标 |
|------|--------|--------|----------|
| Phase 55 | ~2,350 | ~300 | edialog tool 自动调度, 审批 |
| Phase 56 | ~2,200 | ~200 | MCP 全客户端, diff, 文件树 |
| Phase 57 | ~1,300 | ~100 | Ed25519, CPS, TRT |
| Phase 58 | ~1,300 | ~150 | Ne Stage 1 完成 |
| Phase 59 | ~350 | ~350 | LLM + Tool 覆盖率 > 80% |
| Phase 60 | — | — | 模块 < 10K 文件 |

### 4.3 测试策略

```
nt_core_llm: 0 tests → 15+
  - headers() 正确性
  - send_prompt 模拟 HTTP 响应 (mock)
  - send_prompt_stream SSE 解析
  - 4 provider 格式差异

nt_core_tool: 0 tests → 12+
  - BashTool 超时 (timeout_secs=1)
  - BashTool 正常执行
  - ReadTool file not found
  - EditTool 成功率
  - SearchTool 空结果
  - discover_mcp_servers
```

---

## 5. 桌面 App 架构路线

### Phase A (当前 ne-dialog 808 行)
```
┌─────────────┐  ┌─────────────┐
│  Chat pane   │  │ Stream pane│
│  (hand-made) │  │ (ratatui)  │
└──────┬──────┘  └──────┬──────┘
       │                │
       └──────┬─────────┘
              │
       ┌──────▼──────┐
       │  main loop  │
       │  (event)    │
       └─────────────┘
```

### Phase B (Phase 55-56 后 ~3,100 行)
```
┌─────────────┬─────────────┬──────────────┐
│  Chat pane  │  Tools out  │  Plan/Search │
│  (streaming)│  (diff hl)  │  pane        │
├─────────────┴──────┬──────┴──────┬───────┤
│  Input (approval)  │  File tree  │  Cost │
└────────────────────┴─────────────┴───────┘
        │                    │
        └─────────┬──────────┘
                  │
         ┌────────▼────────┐
         │  AppEngine       │
         │  - Tool dispatch │
         │  - MCP lifecycle │
         │  - Git ops       │
         │  - Session mgmt  │
         └─────────────────┘
```

### Phase C (终极 ~5,500 行)
```
┌─────────────┬─────────────┬──────────────┐
│  Chat pane  │  Agent plan │  Artifacts   │
│  (+thinking)│  (read-only)│  (render)    │
├─────────────┼─────────────┼──────────────┤
│  Shell      │  File tree  │  Search      │
│  (backgrnd) │  (browse)   │  (semantic)  │
├─────────────┴──────┬──────┴──────────────┤
│  Input + approval  │  Voice / Image      │
└────────────────────┴─────────────────────┘
```

---

## 6. Ne 语言 Stage 3-∞ 进化

### Stage 3 — 元循环求值器 (在 Ne 中)
```
evaluator.ne 核心 (约 200 行 Ne):
  - eval(expr, env) → value
  - MATCH expr.type 的 6 种 case
  - 自身源码作为 Ne Evaluator 可解析验证
```

### Stage 4 — VSA 原生编译
```
Ne 源码 → VSA 向量 → 谐振器解码 → 执行
编译器生成: Matrix<T> (T x VSA_DIM) 编码层
运行时: resonator_decode(query, program_space)
```

### Stage ∞ — 自修改元语言
```
LanguageSpec 本身是 Ne 程序
MetaEvolver:
  每 N cycle:
    1. 收集 ne_eval 失败模式
    2. 生成新 LanguageSpec 变体
    3. 编译新规范 → 新编译器
    4. 自举验证 → 永久切换 / 回滚
```

---

## 7. 关键路径依赖

```
当前阻塞:
  ├── LLM/Tool 零测试 ← 无依赖, 立即做
  ├── Ed25519 迁移     ← 无依赖, 立即做
  ├── SAHOO CPS        ← 无依赖, 立即做
  │
  ├── Tool 自动调度     ← 依赖: ne_dialog 结构增强
  │   └── 审批系统     ← 依赖: Tool 自动调度
  │       └── MCP 全客户端 ← 依赖: 审批系统
  │
  ├── 流式输出 (真SSE) ← 无依赖, 立即做
  │
  ├── Git 集成         ← 无依赖, 立即做
  │
  └── Ne Stage 1     ← bridge.rs 增强 (无外部依赖)
      └── Stage 2    ← 依赖: Stage 1 完成
          └── Stage 3 ← 依赖: Stage 2 完成
              └── Stage 4 ← 依赖: Stage 3 完成
```

---

## 8. 文件

| 路径 | 当前行数 | 目标行数 | 动作 |
|------|----------|----------|------|
| `src/bin/ne_dialog.rs` | 808 | ~5,500 | Phase 55-56 大幅扩展 |
| `core/nt_core_llm/mod.rs` | 302 | ~500 | 添加流式+测试 |
| `core/nt_core_tool/mod.rs` | 211 | ~400 | 添加测试+审批+超时 |
| `core/nt_core_codegen/bridge.rs` | 1,345 | ~1,500 | Stage 1 transpiler |
| `core/nt_core_language/eval.rs` | 916 | ~1,200 | LOOP/MATCH/std lib |
| `core/nt_core_experience/pcc_safety.rs` | 587 | ~700 | SAHOO CPS |
| `core/nt_core_hive/signed_card.rs` | 221 | ~350 | Ed25519 |
| `core/nt_core_mcp/` | 1,105 | ~1,500 | MCP 全客户端+无状态 |
| `core/nt_core_experience/fusion_deliberator.rs` | 1,475 | ~1,700 | TRT |
| `core/nt_core_agent/` | 720 | ~900 | 去重 hyperagent |

---

## 9. 发布标准

### v0.19 代 (Phase 55-57)
- [ ] ne-dialog: Tool 自动调度 + 审批系统 + Git 集成
- [ ] ne-dialog: 真 SSE 流式 + Cost 追踪
- [ ] LLM/Tool 模块测试覆盖率 > 80%
- [ ] Ed25519 + SAHOO CPS + MCP 无状态
- [ ] 编译 0 errors 0 warnings

### v0.20 (Phase 58-59)
- [ ] Ne Stage 1 编译器完成
- [ ] SELF_SOURCE v2 自举验证
- [ ] MCP 全客户端
- [ ] Diff 渲染 + 文件树 + 计划面板

### v1.0 (Phase 60+)
- [ ] Ne Stage 2 自托管编译器
- [ ] 桌面 App 全覆盖 (18/20 功能)
- [ ] nt_mind/ 拆分 < 10K/模块
- [ ] 形式化验证: Lean 4 + AI 证明器
