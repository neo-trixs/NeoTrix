# 交接件 — 抢救性归档窗口（2026-09-29）

## 1. 会话标识
- 窗口：s000（抢救归档窗口）
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-09-29 22:0x
- **⛔ 全程未推送** —— 远端仍停在 `ff672104`（09-17），本会话 **29 笔提交全部本地**

## 2. 目标（一句话）
把「躺在磁盘上但从未入库」的资产全部抢救进 git（78 份文档 + 82 份非文档 + 25 份训练源码），
并把「断言资产存在性」的查证纪律固化���规则 + 可执行门。

## 3. 已完成

- [x] **78 份文档入库** + 建索引 `docs/architecture/UNTRACKED-ARCHIVE-INDEX.md`
      （覆盖率 78/78 已核对；**未做删除归并**，4 组演进链取证为互补非替代）
- [x] **82 份非文档资产入库**（26 个集成测试 + 32 ops + 15 交接件 + 评测集 + 示例）
- [x] `docs/architecture/DESIGN-CHANNEL-DISPATCH.md`（1445 行）入库
      —— 我此前误判它「从未入库、永久丢失」，**已在 `TODO.md` 两处订正**
- [x] `evals/gaia_mini/` 入库 ⇒ 关闭 `DIR-AUDIT-2026-09-27.md:128` 记的「评测集不可复现」
- [x] `models/training/` 定性：25 个源码入库，273M 生成物按「可再生」忽略
- [x] **立 R-EXIST-1/2/3**（`RUST-STANDARDS.md` §17.8）
- [x] **立门** `scripts/check-untracked-assets.sh`（只读构造，已喂 4 真实样本验证）
- [x] 接入生产：`Makefile` 的 `untracked-assets` + 并入 `audit-all`（6→7 条）
      + `nt_find` 索引第 40 条（实测 1 hit 可达，R-P79）
- [x] `docs/2-PLANS/` 重整：2 份失效路线图归档（git R100 重命名，**删除行数 = 0**）
- [x] 恢复被删文档 `2026-07-01-multi-agent-orchestration-design.md`（326 行，从 `94a4770e`）
- [x] 修 `nt_core_grounded_gate.rs:3` 死链（原引用「§6/§7」**从未存在**，实为 §2.3）
- [x] 重锚 `DESIGN-CHANNEL-DISPATCH.md` 全部行号 + 复核架构判据仍成立
- [x] 清 `crystal.toml` 注释里 5 个真实密钥残留，复核残留 = 0
- [x] 立 `TODO.md` P0 密钥轮换操作单（9 个密钥 + 服务商域名 + 吊销优先级）
- [x] 跑 `check-fresh-build.sh --full` ⇒ **PASS（干净检出能构建）**
- [x] 补入库 5 个漏网文件（`models/training/*.md` + `smelt_repos.txt`）⇒ 坑 #9

## 4. 正在改的文件（我已全部提交，此处仅列他窗的）
**我的未提交改动 = 0。** 以下是他窗在途，**勿动**：

| 文件 | 说明 |
|---|---|
| `neotrix-core/src/l1_action/nt_media/audio_decode.rs` | 他窗在改 |
| `neotrix-core/src/l1_action/nt_media/thumbnail.rs` | 他窗在改 |
| `.neotrix/capability_registry.json` | 他窗在改 |
| `Cargo.lock` | 他窗在改 |
| `results.tsv` | 他窗在改 |
| `.github/workflows/security-scan.yaml` | 未跟踪，他窗产物 |
| `/private/tmp/nt-v9` | **他窗 worktree**，有未提交改动，勿 prune |

## 5. 下一步（按优先级）

1. **🔴 轮换 `crystal.toml` 的 9 个密钥** —— 只能用户做（需登录各家控制台）。
   操作单在 `TODO.md`（已标 🔴 P0，逐行给服务商域名）。含 **NVIDIA 官方 key**。
2. **🟡 IM `/stop` worker 池（`TODO.md` 原 P0 #7）** —— 设计已可施工，但**需独立会话**：
   - 4 文件架构改造：`nt_channel_dispatch.rs` 2123 行 / `nt_channel.rs` 433 /
     `nt_channel_serve.rs` 640 / `nt_channel_telegram.rs` 2353
   - 根因已确认：`on_inbound` 同步调 `run_local_turn_cancellable`（`:483`）阻塞 poller
   - 切分判据仍成立：`ChannelAdapter` 三方法**均无 `Send`/`Sync`**
   - ⛔ 共享工作树已有他窗在改 Rust，**不要边改边跑全量构建**
3. **🟡 `check-fresh-build.sh --full` 建议接进 CI** —— 目前只手动跑
4. **🟢 两处待用户裁决**（我没擅自改）：
   - `DOCUMENTATION-MAP.md:157` 命名示例是虚构格式示例，改它 = 把规约改成迁就现实 ⇒ 建议保持
   - `docs/2-PLANS/` 现为空目录，已加 `README.md` 占位（git 不跟踪空目录）⇒ 建议保留

## 6. 阻塞点
- **无技术阻塞。** 内存门本轮 OPEN，`--full` 干净检出构建 PASS。
- 唯一外部依赖：**密钥轮换需用户权限**。

## 7. 本会话踩的坑（供下个窗口避雷）

| # | 坑 | 代价 | 教训 |
|---|---|---|---|
| 1 | 只查 `git log` 就断言「文件丢失」 | 误判 1445 行设计文档为永久丢失，差点让一条 P0 判成「无法开工」 | **R-EXIST-1**（§17.8） |
| 2 | 编辑 Makefile 写成 `␠␠<TAB>` | **六个既有门 target 全部失效** | Tab/空格敏感文件，改完必须 `make -n` 验 |
| 3 | pre-commit 报「audit-all: 无 target」被当噪声跳过 | 那一刻它**就是真的**，是我打瞎了解析器 | 门报的「无 target」= 事故，不是噪声 |
| 4 | `git checkout -- <path>` 恢复的是**索引**不是 HEAD | 回滚无效 | 要 `git restore --source=HEAD --staged --worktree` |
| 5 | 把 pre-commit 输出误读成提交成功 | 一整笔提交**从未发生**，靠 R-P16 回读 `git log` 才发现 | R-P16：提交后必须回读落盘 |
| 6 | 子代理只查 `git ls-files` 判「文件从未存在」 | 实际是「曾存在后被删」，可恢复 | **R-EXIST-1 当场立功**：同一错误一天内出现两次 |
| 7 | 扫密钥时用 `elif` 兜底打印 | 报「8+1」漏了一个有效行；严格复扫才是 **14 个** | 打印 ≠ 验证（R-SCAN-2） |
| 8 | 报根目录「8 个 md」 | 漏了 `.toml`/`LICENSE`/`Makefile`，实为 **13** | 报数必须实测，违反指针守恒 |
| 9 | ignore 规则只堵了 `*.jsonl`/`smelt/`/`*.bak-*` | commit message 写「零残留」而实测**仍有 5 个漏网**（4 个 AB 实验报告 + `smelt_repos.txt` 熔炼名单） | **「加了规则」≠「已定性」**，规则只挡我想到的那几类 |

## 8. 收工自查

### 8.1 worktree 去向
```
[worktree-gate] worktree=1 个 | 合计 51M | target 占 0M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 1 个 worktree 的未提交改动**不在任何提交里**
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
```

- 本会话**新建**的 worktree：**无**（全程原地操作）。
- `check-fresh-build.sh` 建的临时检出在系统 temp，**脚本已自行清理**（复核 0 残留）。
- `/private/tmp/nt-v9` 是**他窗** worktree（含 `Cargo.lock` / `neotrix-core/Cargo.toml`
  改动 + `nt_evolution_exp.rs` 未跟踪）⇒ **未 prune、未手删**，按 R-DISK-5 留给原窗口。

| worktree | 用途 | 去向 |
|---|---|---|
| （无） | — | 本会话未新建 |
| `/private/tmp/nt-v9` | 他窗 | **不动**，移交原窗口 |

### 8.2 未提交改动去向
- **我的未提交改动 = 0**（全部 29 笔已提交，逐笔 `git cat-file -e HEAD:<path>` 验证过）。
- 他窗的 5 个 modified 文件**全程零交叉**（每笔提交前 `grep -vxF` 反查暂存区确认）。

### 8.3 门禁终态（实测）
```
check-layout.sh --strict          PASS
check-doc-drift.sh                deadlinks: 0
check-untracked-assets.sh         PASS: 0 untracked real assets
check-gate-satisfiable.sh         恒红: 0
make -n audit-all                 可解析, 7 条
check-fresh-build.sh --full       PASS (干净检出能构建)
cargo check -p neotrix            Finished 0 error
cargo check -p neotrix-neobot     Finished 0 error
```
