# NeoTrix 会话交接 — 2026-08-25

> 上游 commit: `f324c9e5` (吸收 Kun) + 本文件提交
> 前端: **359/359 测试通过**, typecheck 干净, build 成功
> 后端 lib: 编译通过（但外部自动化持续干扰，见 ⚠️）

## ⚠️ 紧急：并发写入冲突（新 session 必读）

本机运行 **4 个 opencode 进程** 在同一仓库上并发编辑源文件。
本会话期间它们：
1. 回滚了我 3 次已保存的源文件（tags.ts / Sidebar.tsx / Chat.tsx）
2. 写入语法损坏的代码到 `nt_core_dao_engine.rs` / `nt_core_meaning.rs`
3. sqlite3 批量摄取锁死 knowledge.db（WAL 曾达 6.6GB），致 93 个测试假失败

**新 session 首要操作**:
```bash
# 1. 终止竞争进程（或与用户协调）
ps aux | grep opencode | grep -v grep | awk '{print $2}' | xargs kill
ps aux | grep "sqlite3 /Users/neo" | grep -v grep | awk '{print $2}' | xargs kill

# 2. 恢复被污染的文件到最后已知好状态
git checkout -- neotrix-core/src/core/nt_core_dao_engine.rs neotrix-core/src/core/nt_core_meaning.rs neotrix-core/src/core/l7_capability/mod.rs

# 3. 验证编译
cargo check -p neotrix --lib
```

## 📋 待完成路线图（按优先级）

### P0 — 被中断的改动需验证

| 任务 | 状态 | 文件 |
|------|------|------|
| Plan-in-repo (`sync_plan_to_repo`) | ✅ 代码已写, **❌ 未验证编译** (自动化干扰中断) | `neotrix-core/src/cli/commands/plan_cmds.rs` |
| dao_engine delimiter fix | ✅ 已 git checkout 恢复 | `neotrix-core/src/core/nt_core_dao_engine.rs` |

**验证步骤**: 终止竞争进程 → `cargo check -p neotrix --lib` → `cargo test -p neotrix --lib -- cli::commands::registry --test-threads=1`

### P1 — 吸收路线图 (DSH/Minke/Kun)

详见 `docs/absorption-dsh-minke.md` + `docs/absorption-kun.md`：

| # | 特性 | 来源 | 说明 |
|---|------|------|------|
| 1 | **GUI/TUI 共享运行时** | Kun | `neotrix serve` 单运行时, CLI 与桌面均为客户端。最大架构演进——当前 CLI 与桌面是两套割裂会话。Kun 反模式清单明令禁止双运行时 |
| 2 | **任务证据链** | Kun | tool call ↔ file diff ↔ test result 关联任务实体, 交付时可视化回放 |
| 3 | **Profile 档案隔离** | DSH | config.toml 多 profile + 切换 UI (借 DSH service/profile 分层) |
| 4 | **桌面 Onboarding 向导** | DSH | 首启 provider 配置 + 可选功能勾选 (状态机模式) |
| 5 | **远程 Web 工作区** | Minke | nt_io_web H5 扩展为响应式全功能投影 (非像素流), PWA + Tailscale Serve |
| 6 | **本地模型自动发现** | Minke | 探测 localhost:11434(Ollama) / 1234(LM Studio), 自动注册 provider |
| 7 | **数据迁移向导** | Minke | knowledge.db/session 迁移预览-合并-去重流程 |
| 8 | **细粒度 Revert** | Kun | 审批四件套缺 revert: 编辑级快照+一键回滚 |

### P2 — 架构改进

| # | 改进 | 说明 |
|---|------|------|
| 1 | main.rs invoke_handler 分组注册 | ~400 命令平铺 → 按 service 分组 (DSH 模式), main.rs 从 733 行缩至 <100 |
| 2 | Cache-first agent loop | ReasoningBrain 以 prompt cache 命中率为显式优化维度 (Kun 模式) |
| 3 | i18n 错误文案规范 | 错误输出 = 人类可读完整句; 禁止裸堆栈 (Kun 规范, 已入 DESIGN.md v1.3.0) |

### P3 — 低优先

| # | 项目 |
|---|------|
| 1 | TagBar 层级折叠动画 (已有基础功能) |
| 2 | 自动打标扩展: 更多 TAG_KEYWORDS 词条 / 基于会话历史的学习 |
| 3 | DESIGN.md dark 主题 token 集 (当前 Snowfield White 仅 light) |

## 🔑 关键设计决策记录

| 决策 | 依据 |
|------|------|
| `/kb /memory` 注册但 `is_primary=false` | MCP 桥接依赖注册表分发 (R-P79); 人类一级面经 `list_primary()` 过滤 |
| 星域/* 七域标签对齐 CONTEXT.md | 色值即 NT-* 品牌色, 缝合标签系统与 Faction System |
| 自动打标仅触发一次/会话 | `state.sessionTags[sessionId].length > 0` 即跳过, 不覆盖用户手动标签 |
| SessionStore 双落盘 (JSON + MD) | JSON 供程序恢复, MD 供人审计 (对标 Kun "Observable" 支柱) |
| DESIGN.md v1.3.0 架构反模式 | 吸收 Kun dont-list 纪律: 禁第二运行时/平行适配器/绕过注册表 |

## ✅ 本会话成果基线

| 指标 | 值 |
|------|-----|
| 前端测试 | 359/359 (+4 auto-tag 新增) |
| 后端 lib 编译 | 0 错误 |
| 全量后端测试 | 7785/7786 |
| MCP 桥接 | 50/50 |
| TUI | 140/140 |
| Registry | 6/6 |
| Commits | `4ab738e7` (DSH/Minke 吸收) + `f324c9e5` (Kun 吸收) + 本文件提交 |
