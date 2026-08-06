# External Repository Feature Absorption Workflow

## Description
Analyze external repositories, compare against NeoTrix, identify feature gaps, assess architecture fit, and iteratively absorb features. Also covers batch URL absorption (user pastes a URL list → auto extract/dedup/categorize/capability-map/insert into KB via `scripts/kb_*`).

## Skill Type
workflow

## Tags
- absorption
- architecture
- gap-analysis
- repo-analysis
- iterative-feature
- batch-absorption
- kb-pipeline

## Trigger Phrases
- "分析这个仓库" / "analyze this repo"
- "对比当前项目" / "compare with current project"
- "吸收XX的功能" / "absorb features from"
- "哪些有而本项目没有" / "what features do they have that we don't"
- "融合到当前架构" / "integrate into current architecture"
- "继续后续迭代" / "continue with remaining phases" / "继续后续迭代对应的模块任务"
  → 自动加载 TODO.md，进入下一个未完成的 Phase
- 粘贴 URL 列表 / "把这些 URL 入库" / "absorb these URLs" → 触发吸收对话方法论（六步流水线 + 启发式规则）

## Workflow

### Phase 0: Project Baseline
1. Read `AGENTS.md`, `TODO.md`, `Cargo.toml` for current state
2. Run `cargo check --lib 2>&1 | grep "^error" -A 3` to establish compilation baseline
3. Identify: what is NeoTrix? core abstractions? current completion %?

### Phase 1: External Repository Analysis
For each external repo:
1. `webfetch` README + docs for high-level understanding
2. Clone with `git clone --depth=1`
3. Use subagent (explore) to analyze: module structure, core abstractions, code size
4. Extract: what problem does it solve? key mechanisms? design philosophy?

#### Sub-Agent Dispatch Contract (吸收研究子代理派发契约)

> **来源**: Cycle 159 三连失败教训 (子代理返回问题而非报告 ×3 → 手动 webfetch 兜底)。
> **根因**: 派发 prompt 未明确"自主执行契约"，子代理默认行为(澄清)压过了任务执行。

派发子代理时必须注入以下契约，缺一不可：

| # | 契约条款 | 强制内容 |
|---|----------|----------|
| **C1 自主执行** | `Do NOT ask clarifying questions. Execute autonomously. If a source is unreachable, mark it BLOCKED and continue to the next one. Zero questions, zero stall.` | 子代理不得返回任何问题/选项菜单，必须产出具名报告 |
| **C2 逐源输出** | `For EVERY source in the list, produce exactly one entry. No omissions.` | 防"部分报告"（仅覆盖可获取的来源） |
| **C3 磁盘持久化** | `Write your full findings to {path} BEFORE returning. Use write tool + verify wc -l.` | 防上下文压缩丢失；结果必须落在磁盘 artifact |
| **C4 固定模式** | 每个条目必须用四字段模式：`Source \| Pattern \| NeoTrix mapping \| Reinforce-or-New + consumer` | 输出必须结构化、可合并 |
| **C5 证据接地** | `Every claim must cite what you actually read (README/source path). No inference without evidence.` | 防幻觉 (R-P10/D16) |
| **C6 摘要返回** | `Final message = ONLY the compact table + pointer to the artifact file.` | 返回消息小、artifact 大 |

**派发前检查清单**：
- [ ] URL 清单已写入 `notes/absorption-{batch}.md` 清单头（不依赖对话记忆）
- [ ] prompt 包含 C1-C6 全部契约文本（逐字）
- [ ] 明确指定 artifact 落盘路径（`{PROJECT}/notes/absorption-{batch}.md`）
- [ ] 声明期望返回：表格 + 文件路径，两者缺一即视为失败

**子代理失败兜底协议**：
1. 若子代理返回空/问题/截断 → 判定失败（不重试超过 1 次）
2. 直接切换到主 agent 手动 `webfetch` 逐源抓取，用同一四字段模式产出
3. 每抓 3-5 个来源立即写入 artifact（防上下文压缩）

#### Research Output Schema (吸收研究输出模式)

```
| Source | Pattern (机制) | NeoTrix 映射节点 | 判定 (强化/新增) | 消费者 (R-P79) |
```

- **强化 (Reinforce)** = 注入已有节点（R-P42 正例）；须给出目标模块名
- **新增 (New)** = 能力树完全不存在；须给出同 session 消费者，否则 R-P79 拒绝
- **Blocked** = 来源不可获取（认证/404），注明原因

### 吸收对话方法论 (Batch URL Absorption — Cycle 160b 内化)

当用户一次性粘贴大量 URL 而非单个仓库时，走以下六步流水线（`scripts/` 已实现为可复用脚本）：

```
extract → dedup → categorize → capability map → insert + 显式 FTS → rebuild fallback
```

| # | 步骤 | 脚本/入口 | 关键点 |
|---|------|-----------|--------|
| 1 | **extract** | `kb_batch_absorb.py` | 按域名分派: github→GitHub API(或 HTML 回退), arxiv→export API, 其他→HTML 正文 |
| 2 | **dedup** | `kb_batch_absorb.py` | URL 规范化 + 已在库检查 → `duplicate` 跳过 |
| 3 | **categorize** | 内置于 extract | 4 类: repo / paper / article / org |
| 4 | **capability map** | `absorb_to_capability.py --apply` | 见下方启发式规则 |
| 5 | **insert + 显式 FTS** | `insert_node()` | **必须显式 `INSERT INTO nodes_fts(rowid,title,summary,content,domain)`** |
| 6 | **rebuild fallback** | `rebuild_fts()` | `INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild')` 仅作 fallback |

#### 启发式规则 (后续吸收复用)

| 规则 | 说明 | 命中率 |
|------|------|--------|
| **TITLE_HIT ×3** | 仓库名/论文标题命中能力关键词 → 高置信度映射 (`score = title_hits * 3 + hits`) | ~87% of Top 10 |
| **KNOWN_REPOS 确定性** | 已知顶级仓库 → 确定性 capability (如 `openai/codex`→NT-ACT/execute, `crawl4ai`→NT-WORLD/retrieve, `langfuse`→NT-SHIELD/verify) | 100% for known |
| **API 429 → HTML fallback** | GitHub REST API 限流时降级到 `fetch_github_html` (OG meta + raw README)，保留 stars/language/topics 关键元数据 | 100% 降级成功 |
| **404 pre-filter** | extract 前验证 URL 有效性；600 条中发现 6 个真 404 (1%) — 预检可避免 6 轮空跑 | 防 1% 无效源 |
| **共享临时文件竞态 (Cycle 208 事故)** | `curl()` 若共享同一输出文件 (如 `/tmp/_batch_out`)，并发 workers 会互相覆盖 → 节点 title/content 错配且**无声失败**。必须每调用唯一临时文件 (PID+counter+thread_id)；插入后必须 title↔URL 匹配校验 (R-P16) | 13/45 首插节点污染，已修复 |
| **KNOWN_REPOS 专家键 (Cycle 208)** | 大仓库 (ComfyUI/lobe-chat/semantica/graphrag/ragflow 等) 的 README 含 security/audit 等词会被 keyword 规则误伤 → 顶级仓库必须进 KNOWN_REPOS 确定性键，新增键须查重防 dict 覆盖 | 消除 ~20 误映射 |
| **FTS5 rebuild 陷阱** | 非 external-content FTS5 表，`INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild')` 只重建 shadow 表已有行，**不会**从 nodes 拉新数据 → 显式插入才是检索可用的唯一路径 | 已验证 (kb_batch_absorb.py:232) |
| **顺序预检吞吐瓶颈 (Cycle 232)** | 脚本内置顺序 HEAD 预检是吞吐瓶颈 (150 URL × ≤10s ≈ 25min 超时) → 先外部并发预检 (16 workers) 过滤 dead，再 `--skip-prefilter` 跳过脚本内重复预检 (301 URL ≈ 2min) | 901s 超时 → 2min |
| **fake-ip DNS 污染 (Cycle 232)** | 本地 DNS 被代理 fake-ip 污染 (api.github.com → 198.18.x，curl code:000) → `dig +short <host> @8.8.8.8` 拿真实 IP + `curl --resolve <host>:443:<IP>` 绕过。web 域名 (github.com) 不受影响，仅 API 子域可能被污染 | 235 enrich 全失败 → 0 失败 |
| **enrich 数据文件字段契约 (Cycle 232)** | 外部生成的数据文件列格式必须在脚本内先校验：`owner/repo` 组合列若被当单独 owner 用 → 全批 404。`split('/', 1)` 拆解后再拼 URL | 235 全 Not Found |
| **--apply 只刷 batch_% 节点 (Cycle 232)** | `absorb_to_capability.py --apply` 仅处理 `id LIKE 'batch_%'` 节点；历史 UUID 节点永不被专家键刷新 → 新增专家键后需定向 SQL 修复历史误映射 (内存匹配 17k 节点批量 UPDATE) | 修复 203 误映射 |
| **UPDATE 静默失败 + rowcount 校验 (Cycle 232)** | enrich 脚本 UPDATE 遇 `database is locked` 时异常重试循环会自然退出不 raise，但 `ok += 1` 仍执行 → 日志虚报成功数据未写入。根因：常驻 `neotrix-experience` 进程持有 KB WAL 锁。修复：UPDATE 后必须校验 `cur.rowcount > 0` + 严格重试 | 32 虚报 → 26 真实 |
| **GitHub repo 301 改名 (Cycle 232)** | `api.github.com/repos/{o}/{r}` 对已改名 repo 返回 "Moved Permanently"（无 stars 字段）→ curl 必须加 `-L` 跟随重定向拿新 `full_name`。改名后需更新 KB 节点 url + metadata.owner + 记录 `redirected_from` | 3 repo 改名处理 |
| **GitHub Pages 阻断 → API 桥接 (Cycle 232)** | Pages CDN (185.199.x) 被网络出口白名单阻断时，站点内容 = 对应 repo 源码 (`GET /repos/{o}/{r}/pages` 返回 source.branch+path)。经 `api.github.com/repos/{o}/{r}/contents/{path}` (base64) 拉源文件组装 node 字段，复用 insert_node 入库 | 2 个 .github.io 从 000 → 100% 吸收 |
| **SPA 站点内容在 TS 字符串字面量** | Vite/React 站点 (index.html 仅 647B 壳) 正文在 `src/content/*.ts`/`*.tsx` 的字符串字面量 → 正则 `"([^"\\]*(?:\\.[^"\\]*)*)"` 提取 + len>30 过滤，即可组装 article 节点 | vibe-designing-playbook chapter1.ts → 2577B |
| **静态站内容直接是 markdown** | 静态 GitHub Pages (index.html 是入口壳) 正文在 `article.md`/`README.md` → markdown strip 格式符 (`#>*_\`~|-`、图片/链接语法) 后取行 >30 字符拼接 | morpho article.md 55KB → 4000B 节点 |

#### absorbed_capability 数据层追踪 (R-P79 闭环)

`absorb_to_capability.py --apply` 写节点 metadata 四元组：`{branch, capability, evidence, mapped_at}`。
- **D14/D20 审计不依赖外部文档** — 节点级元数据本身就是审计证据
- **R-P42 落地验证** — 每节点映射到现有能力树分支 (NT-IO/SHIELD/ACT/MIND/CORE/WORLD/MEMORY)，零新建平行模块
- **数据可追溯** — 吸收来源、能力分支、证据链接全部落位 KB

### Phase 2: Gap Identification
1. For each feature found in external repos, search NeoTrix source to confirm presence/absence:
   ```bash
   grep -r "feature_name" src/ --include="*.rs" | head -5
   ```
2. Classify each gap:
   - ✅ Present: has equivalent
   - ❌ Absent: no equivalent
3. Build a feature matrix: Source | Feature | Present? | Location

### Phase 3: Architecture Fit Assessment
Evaluate each ❌ feature for integration feasibility:

| Tier | Criteria | Action |
|------|----------|--------|
| 🟢 Tier 1 | Existing type/struct — just add enum variant or field | +10-30 lines |
| 🔵 Tier 2 | Existing function/impl block — add new method | +30-100 lines |
| 🟡 Tier 3 | Replace monolith with pipeline pattern | 1-2 day refactor |
| 🟠 Tier 4 | New module, but architecture compatible | New file |
| 🔴 Tier 5 | Requires architecture adjustment | Assess ROI first |
| ⛔ Blocked | Architecture conflict | Document for later |

### Phase 4: Iterative Absorption (the Loop)

```
Each iteration:
  ┌──────────────────────────────────────────┐
  │ 1. Write code                          │
  │ 2. cargo check --lib (0 errors gate)   │
  │ 3. cargo test --lib <module>           │
  │ 4. If fail → fix, goto 2               │
  │ 5. Update TODO.md                      │
  │ 6. User presents next task             │
  └──────────────────────────────────────────┘
```

Per-phase discipline:
- **Phase N**: 1 feature per phase (or 1 cohesive group)
- **Pre-existing errors** not caused by changes: note but proceed
- **Borrow checker issues**: use `take/put_back` for self-borrows, `Send + Sync` bounds for trait objects
- **Test gap**: always write tests for new modules (min 3-5 tests per new file)

### Phase 5: Review & Document
1. `cargo check --lib` — must be 0 errors
2. Run all affected module tests
3. Summarize: which features absorbed, from which source, file locations
4. Update AGENTS.md lookup chains if adding new core abstractions
5. Update TODO.md: mark absorbed features as [[completed]]
6. **吸收对话经验沉淀**：批量吸收后更新本文「吸收对话方法论」章节 — 新启发式/新容灾/新陷阱应回写，使经验随代码演进（R-P79 接线门延伸：经验记录同样不能滞后于实现）

**子代理产出验证 (必做)**：子代理声称完成后，主 agent 必须
1. `wc -l` + `grep "Source|"` artifact 文件 → 确认条目数 = 来源数（防幻影报告，R-P49/R-P16）
2. 抽查 2-3 个条目引用是否与 README 原文一致（防幻觉，R-P10）
3. 只有验证通过才可将其合并入吸收矩阵；否则视为失败走兜底协议

## Success Criteria
- `cargo check --lib`: 0 errors (ignoring pre-existing errors in unrelated modules)
- All new module tests pass
- Feature gap count reduced (track in TODO.md)
- No regression in existing test suites

## Common Patterns

### Self-borrow resolution
When pipeline stages need `&mut self` and pipeline is a field of self:
```rust
let mut pipeline = std::mem::take(&mut self.pipeline);
let result = pipeline.execute(self);
self.pipeline = pipeline;
```

### Send + Sync for trait objects
When adding traits to structs used in async contexts:
```rust
pub trait MyTrait: Send + Sync { }
```

### Module registration
New file → declare in parent's `mod.rs` → add `pub use` re-exports → `cargo check`
