# handoff — ABSORPTION-ROUND24 / 调用图 G4 + G6 + 两个 P0（一修一待签）（2026-09-30）

> 窗口任务：吸收用户第 24 批 URL（~700），给 NeoTrix/NeoBot 核心进化建议，
> 定位到 code map 支脉节点，产出任务清单并批量执行。
> 分支 `feat/capability-absorb-20260828`。

## 0. 三句话状态

1. **P0-A（发布链路幻影门）已修复** —— 含 fail-open 三处成因、`.gitignore` 令门永不可满足、
   `check-ci-refs` 覆盖漏洞。7 条路径全部实测。
2. **P0-B（许可低报）待你签署** —— 门故意 FAIL；商业/法务判断我不代签。
3. **「死代码判据」实测不可靠**（误报率轨迹 100%→67%→0%，经五版修正）——
   故最终只交出 **3 条已手验的候选**，且**明确不建议据此删除**。

## 1. 我的改动（**全部未提交**）

| 文件 | 类型 | 内容 |
|---|---|---|
| `scripts/ops/nt_callgraph.py` | 新增 | 调用图引擎 `--impact`/`--deps`/`--stats`/`--unreachable`，**闭合 G4**。内含 F1–F6 误报源与五版修正轨迹 |
| `scripts/check-license.sh` | 新增 | 外部来源与许可门，**闭合 G6** |
| `.neotrix/LICENSE-EXCEPTIONS.md` | 新增 | 许可例外**唯一**放行通道（人工签署） |
| `docs/architecture/ABSORPTION-ROUND24.md` | 新增 | 全部吸收记录 + 证伪表 + F1–F6 + 三轮执行 |
| `neotrix-core/.../nt_shield/provenance/external-inputs.example.json` | 新增 | 清单 schema 模板（`sha256` 为 `unpinned:` 占位 ⇒ 不可产出假验证） |
| `.../safety_tools/provenance_check.sh` | 改 | **fail-open → fail-closed**（三处成因）+ 修自身 usage 路径 |
| `.github/workflows/release.yml` | 改 | 修幻影脚本路径（原 `scripts/…` 从不存在） |
| `.gitignore` | 改 | 为 `nt_shield/provenance/*.json` 开解除忽略（cache/ 仍忽略）⇒ 门首次**可满足** |
| `scripts/check-ci-refs.sh` | 改 | 增第 4 类引用（`run:` 脚本路径），判据两轮修正 |
| `scripts/ops/nt_calledges.py` | 改 | 删死分支 `kind=='method'`；更正陈旧值 94%→实测 100% |
| `scripts/ops/nt_absorption_live.py` | 改 | `--graph` 第二判据（G5）+ 默认模式自述判据边界 |
| `apps/neobot-desktop/frontend/VENDOR.md` | 改 | ⛔ 更正许可字段（P0-B） |
| `.github/workflows/ci.yml` / `Makefile` / `AGENTS.md` | 改 | 挂门 + target + 决策树 |
| `docs/architecture/absorption-sources/repos.csv` | +5 -0 | 5 个新源 |
| `.neotrix/task-index.json` | +73 -0 | ⚠️ **已被他窗捎带提交**（见 §6） |

## 2. P0-A 已修复，但有一件**必须你做**的事

修复内容见 `ABSORPTION-ROUND24.md` §10.1（fail-open 三处成因 + 7 条路径实测 +
`check-ci-refs` 覆盖漏洞 + `.gitignore` 令门永不可满足）。

⛔ **我没做、也不该由我做**：填 `provenance/external-inputs.json` 的**真实内容**
（每个外部构建输入的 name/kind/version/sha256）。
**我不伪造哈希** —— 一个用我编造的数据跑通的供应链门比没有门更坏。
schema 见同目录 `external-inputs.example.json`。

⇒ **发布仍会挡在 `Verify external input provenance` 这一步，这是真实状态。**
填完清单后：① 跑一次确认非空转；② 门自然转绿。

## 3. P0-B 待你签署

`frontend/LICENSE.details` 的附加条款「No Commercial Secondary Development」
（明写冲突时优先）与 `VENDOR.md` 原记录「MIT License」不符；该树正在被持续修改。
条款原文与三点必知已写入 `VENDOR.md` 许可节。

签署：在 `.neotrix/LICENSE-EXCEPTIONS.md` 加一段
`## ACKNOWLEDGE-<n>`，`tree`/`decision`/`owner`/`date` 必填；
`accepted-with-condition` 还需 `condition` + `review_by`。
⛔ 模板所在的 ``` 围栏内容会被门跳过 —— **别把真实值填进模板**。

## 4. 「死代码」结论：不可靠，只交 3 条已手验候选

`--unreachable` 的 `suspect-dead` 桶经**五版**修正才收敛
（v1 迭代器 → v2 末位实参 → v3 嵌套括号 → v4 rustfmt 拆行 → v5 回调+领域回调），
误报率轨迹 **100% → 67% → 0%**。**3 条全部独立复核为私有 + 零引用**：

| 符号 | 位置 |
|---|---|
| `hz_to_mel_slaney` | `neotrix-core/src/l1_action/nt_media/nt_speech_transcribe.rs:76` |
| `get_geo_cache` | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_stealth_net/geo_proxy.rs:32` |
| `_now_ms` | `neotrix-core/src/l3_embodiment/nt_shield/shield_core/nt_shield_mcp_security/nt_mcp_registry.rs:203` |

⛔ **即便这 3 条我也不建议直接删**：残余风险是「未知族仍存在」而非零
（宏注册 / 生成代码 / 外部 trait 接线都可能看不见）。这正是 G7 的实证。
**若要删，逐条读代码 + 删后跑全量测试**，别信任何自动清单。

## 5. 门的当前状态

| 门 | rc | 判定 |
|---|---|---|
| `check-ci-refs.sh --strict` | **0** | ✅ **绿** —— P0-A 真修好了，不是把门改绿 |
| `check-license.sh` | **1** | ✅ **正确红**：待 §3 签署 |
| `check-truth-surface.sh --strict` | 1 | ⚠️ **他窗 WIP**（4 条 `UNCOMMITTED_DEP` 全在 `neobot-*`）；干净检出应为 0 |
| `check-layer-deps.sh --strict` | 0 | ✅ 绿 |
| `check-untracked-assets.sh --strict` | 1 | ✅ **正确**：我的新文件尚未 `git add` |

## 6. 与另一窗口的交界

- 另一窗口在改 `crates/neotrix-neobot/`、`apps/neobot-desktop/`、`nt_media/`
- 其提交 **`519d78b9`** 把我未提交的 `.neotrix/task-index.json` 一并带入
  （内容无损，纯增量 +73）。**这是本轮唯一一次**共享 index 事故。
  ⛔ 我一度误判为「4 次」—— 起因是我自己为测门做了 `git add -N`（intent-to-add），
  随后用 `git ls-files --error-unmatch` 读它，**intent-to-add 会让该命令成功**，
  于是被我当成「已被提交」。**`git show HEAD:<path>` 才是权威判据。**
  ⇒ 教训：不要用 `git ls-files` 判断「是否已进 HEAD」，它读的是**索引**不是 HEAD；
  用完 `git add -N` 必须 `git rm --cached` 还原，否则它会被别人的提交捎带进去
  （本轮我已还原，索引现为干净）。
- 该提交落地了 `nt_decompose.py`（CAPABILITY-GAP 建议 2 的原子拆解器），
  与我的 `nt_callgraph.py` **互补**：它拆条目，我算可达性
- **我未触碰**上述目录、`Cargo.toml`/`Cargo.lock`、`nt_check_*.mjs`、`nt_decompose.py`

## 7. 未做

| 项 | 理由 |
|---|---|
| T2 `--tests` 通道（补 312 条 test-only） | 需 `--all-targets` ~30min；他窗仍在改 neobot，AGENTS.md 禁止并行全量。**等他窗收工再做** |
| T5 188 条 textual-prod 归因 | 已证实多为 F2 同名碰撞 + 散文命中，收益低于成本 |
| `cargo test` / feature-gates | 本轮零 `.rs` 改动（只改 `.sh`/`.py`/`.md`/`.json`），按 R-SCAN-3 可沿用门记录 |
| 填 `external-inputs.json` | §2，需真实数据 |

## 8. 下一步最优顺序

1. **填 `external-inputs.json`**（解开发布阻断，唯一必须你做的技术活）
2. **签署 P0-B**（解 `check-license` 红）
3. **T2 `--tests`**（等他窗收工）
4. 那 3 条死链候选：读代码后再决定
5. 提交本轮 15 个文件（须 `git commit --only <路径>`）

## 9. 本窗口踩到的坑（都已修，供后人）

| 坑 | 代价 / 教训 |
|---|---|
| `--stale` 第一版**从不检查却恒报「0 过期」** | 无声的安心最贵。改用边表 `span` 真实路径后：2161 检查 / 4 真过期，与 `git status` 完全重合 |
| 例外记录**格式模板长得像真签署** | 门输出「已有**人工签署**」而实际无人签。修：占位符 + 解析器跳过围栏 |
| `$VAR` 紧跟多字节字符被 bash 3.2 并入变量名 | 只在签署路径触发 —— **测出来的** |
| `json.dump(indent=2)` / `csv.writer` 整体重排 | 违反 R-P16。已还原重做为 +73/-0、+5/-0 |
| **「漏判属安全方向」是错的** | `from_fn_with_state(state.clone(), h)` 漏判 ⇒ **活的** middleware 被判死 = 危险方向。**判据方向性不能靠推测，要看漏判后果落在哪边** |
| F3/F6 探测器五版才收敛 | 每修一族暴露新一族 ⇒ 「零调用⇒死」不可能一次做对 ⇒ 桶名从 `dead-private`（假称高置信）改为 `suspect-dead` |
| `check-ci-refs` 的 run-script 判据两次假阳性/漏报 | `tools/` 出现在组件中间；收窄版漏 `foo/bar/x.sh`。定稿：须含 `/` + 组件边界 + 跳 `${{`，实测 13 个真 workflow 零噪声 |
| `provenance_check.sh` 三处「检测写了但不生效」 | `error()` 不 exit · 进程替换退出码不传播 · 空输入检测用 `error` ⇒ **供应链门报告通过而实际什么都没查** |

---

*交接人：NeoTrix 主开发代理 · 2026-09-30 · 未推送，仅本地*
