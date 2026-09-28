# Handoff — Qwen-MM-Plugins 吸收（A+B+C 全做）· 2026-09-28

> 分支：`feat/capability-absorb-20260828`（主工作树，未提交——用户未要求提交）。
> 正典记录：`docs/architecture/ABSORPTION-QWEN-MM-2026-09-28.md`（缺口矩阵／交付／
> 模式沉淀／反模式表／E2E 实证）。patch 兜底：`.neotrix/worktree-salvage/qwen-mm-20260928.patch`。

## 1. 做了什么（用户决策：A+B+C 全做）

- **A1** `neotrix-core/src/nt_mcp_stdio_session.rs`（新建，~800 行含 11 单测）：
  MCP stdio 会话客户端（initialize→initialized→tools/list|call，stdin 保持打开
  等回包，stdout/stderr 双泵线程，image 块落盘、`McpSessionTool` NativeTool 适配）。
- **A2** `agent.rs`（+79/-9）：`McpServer` 加 `use_session`＋`session_timeout_ms`；
  `register_stdio_session(+_global)`；`as_native_tools` 分叉。旧 `StdioNativeTool`
  零改动。＋`nt_qwen_mm_manifests.rs`（新建，~870 行含 10 单测）：core 7 工具＋
  search 3 工具真实 manifest、uvx→PATH→checkout 三阶探测（零 spawn）、SYSTEM_DEPS
  探测、search key 门。＋`entry/interactive.rs` 双路径接线＋`lib.rs` 注册两模块。
- **A3** `skills/nt_multimodal/SKILL.md`（新建）＋`index.json`（＋15 行纯增，
  曾因 `json.dump` 整文件重排 1112 行，已 revert 改手术式插入）。
- **B** search manifest 注册＋credential-gated（本机无 key → 正确 skip，已验）。
- **C** `scripts/check-capability-manifests.sh`（新建）：M1–M5 门，首跑
  `checked=24 fails=19 warns=36`——全是存量（M3 双语漂移），我的条目零 mismatch。
- **E2E**（本机，真服务器 v1.1.0，sparse-checkout 48py/1.1M＋venv）：
  read_image（600x800→448x576＋JPEG 块）、media_info（ffprobe 全量）、tools/list
  （恰好 7 工具，与 manifest 逐名一致）。驱动帧与 Rust 客户端同字节。

## 2. 验证证据（全部可重跑）

| 门 | 结果 |
|---|---|
| `cargo check -p neotrix --lib` | 0 errors（修过 2 错：`Path` 未用导入、`Instant::saturating_sub`→`saturating_duration_since`） |
| 目标单测 20 个 | 全绿（含 bash 伪造服务器真进程 framing：list/call-text/call-image/error/spawn-fail/timeout/dead/native＋结构单测） |
| 全量 `--lib` | **12171 passed＋1 flake**：`test_backoff_increases`（jitter 随机断言，l6_meta，我没碰；重跑 3/3 过）——按 R-SCAN-1 不碰 |
| `nt_lock_audit.py neotrix-core/src` | **0 条**（改了 .rs 后重跑，非沿用） |
| `check-layer-deps.sh --strict` | 0 new（101/101；根文件不在门内） |
| fmt | 我的行零 diff（agent.rs 4 处已手修；全仓 13853 存量 diff 不动） |
| clippy `--lib` | 我的文件 0 hits；`neotrix-types` 533 存量错（`deny(warnings)`＋unwrap，自有，不动） |
| `check-skill-gate.sh` | 我的条目零失败（strict 红的是 58 存量无 license） |

## 3. 关键教训（已写进正典 §3⚠️）

初版"写完三帧即关 stdin"被真服务器证伪：服务端收到 CallToolRequest 但 stdin EOF
拆除会话、零回包 rc=0。R-SCAN-2 活例子——脑内模拟≠真实行为，改写成"stdin 保持
打开等回包"后才通。另：`json.dump(indent=2)` 会重排 index.json（1112 行 churn），
共享 JSON 必须手术式插入。

## 4. 他窗状态（本会话观察）

- 开工时 `.worktrees/ratchet` 脏=1 且近 3h 有 .rs 改动（疑似他窗在用）→ 全程未碰；
  收工时已干净（脏=0，他窗提交了），仍活跃 → **勿删**。
- `cargo test -p`（PID 88825，rustc 98% CPU）导致 mem 门 BLOCKED（exit 2）约 1h；
  其退出后门开，才跑 cargo。期间只做非 cargo 工作。
- 主树存量改动（10 文件：docs×6＋kb_primitives＋evolution_daemon＋baseline）开工前
  已在，与我零交集（我的文件：agent.rs／interactive.rs／lib.rs／index.json＋5 新建）。
- `skills/index.json` 我动过（＋15 行）；`TODO.md`／`CHANGELOG.md` 刻意没碰（§5）。

## 5. 下一步（正典 §8）

1. mem 门开后第一件事已做完（本会话做完）；若他窗又起全量构建，先
   `nt_mem_gate.sh` 再动手。
2. 配 `SERPER_API_KEY` → search 自动注册，e2e `web_search` 一轮。
3. `QWEN_MM_PLUGINS_CHECKOUT` 指向 sparse-checkout 目录 → 无 uvx 也可生产。
4. §4-P1/P2/P3（mhs 硬地板／spatio 几何分离／不可变版本）进路线图，需先过 0.2 证伪门。

## 6. 没做的（刻意）

- `TODO.md`／`CHANGELOG.md` 更新：共享文件＋他窗活跃， release-note 口径是维护者事，
  不顺手写（理由：churn＋归属）。
- `omni-*`／`video-edit`／`api` 执行层：需 DashScope key，无 key 不可验证，
  R-P79 拒绝（反模式表已记）。
- clippy 全仓 533 存量错、fmt 13853 存量 diff、19 个 M3 描述漂移：存量债，
  批量修是 churn（门头注释＋正典 §5 已记）。

## 7. 测试命令（复制即跑）

```sh
sh scripts/ops/nt_mem_gate.sh; echo $?
cargo test -p neotrix --lib -- nt_mcp_stdio_session nt_qwen_mm_manifests agent::tool
NT_QWEN_MM_LIVE=1 cargo test -p neotrix --lib -- --ignored live_qwen_core_keyless --nocapture
python3 scripts/ops/nt_lock_audit.py neotrix-core/src
bash scripts/check-layer-deps.sh --strict
bash scripts/check-skill-gate.sh; bash scripts/check-capability-manifests.sh
bash scripts/ops/nt_qwen_mm_setup.sh check
```

## 8. 收工自查（必填）

### 8.1 worktree 去向

```
[worktree-gate] worktree=1 个 | 合计 50M | target 占 0M
[worktree-gate] 带未提交改动: 0 个 | 近3h有改动: 1 个
[worktree-gate] ⚠️  1 个 worktree 近 3 小时仍有 .rs 改动 ⇒ 可能他窗在用，勿删
```

| worktree | 用途 | 去向 |
|---|---|---|
| （本会话未新建 worktree） | — | 无需 prune；`.worktrees/ratchet` 是他窗的，脏=0 但仍活跃 → **勿动** |

### 8.2 未提交改动的去向

本会话改动（分支 `feat/capability-absorb-20260828`，用户未要求提交 → 不提交，
patch 兜底＋工作树保留）：

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `neotrix-core/src/nt_mcp_stdio_session.rs`（新建） | MCP stdio 会话客户端＋11 单测 | ☑ patch 兜底（`.neotrix/worktree-salvage/qwen-mm-20260928.patch`，`apply --check --reverse` 已验）＋工作树保留 |
| `neotrix-core/src/nt_qwen_mm_manifests.rs`（新建） | core7＋search3 manifest＋探测＋10 单测 | ☑ 同上 |
| `neotrix-core/src/agent.rs` | use_session 路由（+79/-9） | ☑ 同上 |
| `neotrix-core/src/entry/interactive.rs` | 双路径接线（+25） | ☑ 同上 |
| `neotrix-core/src/lib.rs` | 注册两模块（+2） | ☑ 同上 |
| `skills/index.json` | nt_multimodal 分类＋索引（+15 纯增） | ☑ 同上 |
| `skills/nt_multimodal/SKILL.md`（新建） | skill 正文 | ☑ 同上 |
| `scripts/check-capability-manifests.sh`（新建） | M1–M5 一致性门 | ☑ 同上 |
| `docs/architecture/ABSORPTION-QWEN-MM-2026-09-28.md`（新建） | 吸收正典 | ☑ 同上 |
| 本文件 | 交接 | ☑ 同上（patch 含本文件提交前版本说明：见 §8.2 注） |

> 注：patch 在 handoff 写完后生成，含上述全部 10 项（`git add -N`＋`diff HEAD`）。
> 「留给下一个 agent 不算去向」——此处去向是 **patch 兜底（已验可回放）＋分支工作树
> 原位保留**，接手者 `git apply --check` 可验、`git checkout` 可取，不依赖口头移交。

### 8.3 门状态

- `nt_worktree_gate.sh check` 的 exit code：0（1 worktree，他窗活跃，勿删；我零新建）
- 提交前是否跑过 `cargo check -p neotrix --lib`：☑ 是（0 errors；目标 20 单测全绿；
  全量 12171＋1 存量 flake 已证伪）
- 门红：`check-layer-deps --strict` 0 new（PASS）；`check-capability-manifests --strict`
  红 19——**全是存量 M3 漂移（本会话引入 0）**，advisory 下 exit 0，已在门头注释写明不清零不接 CI

## 9. 附录（2026-09-28 追加：无 key 体的方案）

用户问"无 key 有没有体的方案" → 有（core 原生无 key）：

- `scripts/ops/nt_qwen_mm_setup.sh`（新建，`setup|check|update`，已 `bash -n`＋
  本机跑通）→ 落盘 `~/.neotrix/qwen-mm/{src,venv,bin}`（仓外）→
  `export PATH="$HOME/.neotrix/qwen-mm/bin:$PATH"` 即生产（`LaunchVia::Path`，零 Rust 改动）。
- `.neotrix/task-index.json` 31→32 条（`qwen-mm-setup`，手术式＋9 行）。
- 无 key 实测：read_image／media_info／read_video／visualize-NIfTI（3 切片 JPEG，
  此前全仓零 `.nii`）／visualize-Rust 源码／tools-list 7 工具逐名一致——全绿。
- Rust 活证明 `#[ignore] live_qwen_core_keyless`：**已跑，全绿**
  （`live via Path` → 7 工具清单 → media_info 真元数据 → read_image 落盘
  `/T/neotrix-mcp-artifacts/mcp_img_*_0.jpg` → read_video 真帧 →
  `ALL GREEN (no keys used)`，2.00s）。跑法见 §7。
- pre-commit 预警：`agent.rs` 存量非 `nt_` 名，`git add` 它会触发命名门 exit 1——
  提交本轮改动须 `--no-verify` 或钩子加白（存量条件，非本轮引入；本轮未提交）。
- patch 兜底已**重新生成**（含本附录＋setup 脚本＋活测试）：
  `.neotrix/worktree-salvage/qwen-mm-20260928.patch`（`apply --check --reverse` 重验）。
