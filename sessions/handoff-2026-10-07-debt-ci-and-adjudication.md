# handoff — 债务清账 / CI 真跑 / 三道裁决（窗口收工件）

> 交接件：`sessions/handoff-2026-10-07-debt-ci-and-adjudication.md`
> 分支：`feat/capability-absorb-20260828` · 收工时 HEAD：`c301a08d`
> 性质：本窗口**全程在主树工作，未开任何 worktree**，未提交改动 **0 处**。

---

## 0. 三十秒版

本窗口做了三件事，全部已提交、全部门绿：

1. **让 CI 真的跑门** —— 发现 `desktop` job 里 9 道读 dist 的门排在 `Build frontend`
   **之前**，而干净检出里 dist 根本不存在 ⇒ 那 9 道门必红 ⇒ 后续步骤（含
   `cargo test -p neobot-desktop` 89 条与 `p neotrix-neobot` 581 条）**从未执行过**。
2. **把「不在 CI」的门全接上** —— `STATUS.md` §1.5 声称的 11 道门至此全部有执行点，
   另加 `check-layout`。顺带修了 `check-layout` 的 fail-open、dot 文件盲区，
   并按裁决**移除了账本棘轮**（无理由放行口）。
3. **D3/D5/D6 三道「决策题」** —— 实测**两道是代码缺陷不是决策题**，已修；
   第三道（D5）真阻塞，改为**建通用框架**（未接线能力必须有机器可读的阻塞记录
   + 给业务侧的 schema 插口），⛔ 不发明 schema。

---

## 1. 提交清单（本窗口，16 个）

| hash | 内容 |
|---|---|
| `853f2457` | 复制按钮 22→24px（WCAG 2.2 SC 2.5.8）+ 运行状态加 `aria-live` + 修 a1 门 C3 误报 + 产物新鲜度门 |
| `777c7b0e` | D2 死配置逐字段裁决（liquid_glass 7/8 装饰字段等）+ `check-naming` 单目录试点 |
| `b00402f3` | 补上 pool 重命名的旧路径删除（见 §4 事故） |
| `4fd79509` | 轨迹页 + 诚实增量（run 生命周期信号，不用 `busy` 猜） |
| `07495e4d` | 新鲜度断言抽公用件 + 接入全部 16 道读 dist 的门 + 修窗口化门陈旧桩 |
| `9b1e741a` | 接 4 道门 + **修它们的冷启动竞态**（固定 sleep 1500ms ⇒ 轮询） |
| `cafe4f7b` | 接最后 6 道门（§1.5 的 11 道至此全覆盖） |
| `07bcfe54` | `check-layout` fail-open 修复 + `is_healthy` 裁决 + `STATUS` §1.5 对齐现实 |
| `c2e1c959` | dot **文件**入扫描（并更正我上轮「dot 目录也扫不到」的错误） |
| `e03723b1` | 移除 `check-layout` 账本棘轮 ⇒ 单一裁决机制 + 接进 CI + 删死豁免 |
| `c8ab5e48` | 删除已无主的 `layout-baseline.txt` |
| `e381324f` | 更正 `.project-map` 的「1 秒重建」（实为 2s / 30min 两级） |
| `7dd97f3b` | **D3-Phase-2** 治理容器去重（791→617 行，13629 测试全绿） |
| `9ef71c5d` | 补 `neobot-desktop` 的 `license` 字段 ⇒ cargo-deny 从红转绿 |
| `764dd16c` | D3/D5/D6 裁决终局记录（含本窗口 3 次自我更正） |
| `c301a08d` | **D5 通用框架**：`.neotrix/capability-blocking.json` + 门加 (d)(e)(f)(g) + 接进 CI |

---

## 2. 收工自查（§8 必填）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` ⇒ **rc=4**（正确报告未提交，不是门回归）：

```
路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
/Users/neo/Downloads/neotrix/.worktrees/evo     | d524e278 | HEAD | 0 | 3562M | 3480M | no
/Users/neo/Downloads/neotrix/.worktrees/merge-b | 1a48ecd3 | HEAD | 3 |   66M |    0M | no
/Users/neo/Downloads/neotrix/.worktrees/nt-v2   | 6b57fe08 | HEAD | 0 |   81M |    0M | no
主树：29 处未提交 | target 100371M（只报告，不影响退出码）
worktree=3 个 | 合计 3709M | target 占 3480M | 带未提交改动: 1 个（merge-b）
```

**本窗口新建的 worktree：0 个。** 全程在主树工作，用 `git commit --only <显式路径>`
逐批提交（共享 index 下 `暂存区核对` 与 `提交` 不原子，⛔ 全程未用 `-A`）。

| worktree | 归属 | 状态 |
|---|---|---|
| `.worktrees/evo` | **他窗** | 干净，但 `target` 3480M，门判「零风险可回收」（`... clean`）。⛔ 非我所有，未动 |
| `.worktrees/merge-b` | **他窗** | **3 处未提交**。⛔ 非我所有，**未动** —— 需其主人 `prune` 或自行提交 |
| `.worktrees/nt-v2` | **他窗** | 干净。⛔ 非我所有，未动 |

### 8.2 未提交改动的去向

**本窗口结束时未提交改动 = 0 处。** 逐个核对过本会话改过的 17 个文件，全部 `git status` 为空。

主树剩余 29 处未提交**全部属于他窗**，本窗口**一件未碰**：
`.neotrix/capability_{overrides,registry}.json` · `Cargo.lock` ·
4 份 `ABSORPTION-*.md` · `nt_core_event_bus.rs` · `nt_judge.rs` ·
`nt_file_ability/visual/visual.rs` · `nt_io_neocodex/agent/nt_agent_exec.rs` ·
`kb_search.rs` · `check-arch-rules.sh` · `gate-registry.tsv` ·
`nt_tui_e2e.py` · `nt_worktree_gate.sh` · `probes/check-arch-rules.sh` ·
`probes/check-fake-signal.sh` · 1 份 handoff · 13 个未跟踪（含 `.freebuff/`）。

ⓘ 收工时 `check-layout --strict` **曾一度 rc=1**，唯一原因是 `.freebuff/`（19:36 由**他窗**
新建的根目录）—— 这正是本窗口把 dot 目录纳入扫描想要的效果：**新根目录会被判红**。
**本窗口刻意没有替它加 `ALLOW_DIRS` 白名单**（那不是我拥有的东西，代加就是替别人掩盖）；
他窗随后自行把它写进 `.git/info/exclude` ⇒ 归入「git 不管」单列，门自动恢复绿。
⇒ 这件事验证了设计的意图：白名单要么**带理由**留下，要么被明确声明为 git 不管，
**没有第三条无理由放行的路**（账本棘轮已在本轮移除）。

### 8.3 门状态

| 门 | rc | 说明 |
|---|---|---|
| `check-layout --strict` | 0 | ⛔ 收工一度为 **1**：他窗 19:36 新建根目录 `.freebuff/`，被本窗口的 dot 扫描判红。**他窗随后把它写进 `.git/info/exclude`** ⇒ 归入「git 不管」单列，门自动恢复绿（**本窗口未代加任何白名单**） |
| `check-executor-registry --strict` | 0 | 本窗口扩展过 (d)(e)(f)(g) |
| `check-dead-config-flag --strict` | 0 | 新增 0，基线 677 |
| `check-arch-rules --strict` | 0 | （他窗已修） |
| `nt_gate_coverage.py`（元门） | 0 | 无死豁免 |
| `nt_check_status.mjs` | 0 | |
| `nt_check_bytes.mjs` | 0 | 4770 文件 / 0 处 U+FFFD |
| `cargo deny check licenses` | 0 | 本窗口从红转绿 |
| 16 道 UI 门（已接新鲜度） | 16/16 绿 | |
| `cargo test -p neotrix --lib` | 0 | **13631 passed / 0 failed** / 38 ignored |
| `cargo test -p neotrix-neobot --lib` | 0 | 581 passed / 1 ignored |

提交前跑过 `cargo check -p neotrix --lib` 与 `cargo test -p neotrix --lib`。

---

## 3. 本窗口的三次自我更正（比修的东西更值钱）

| # | 我的错误 | 真相 | 纪律 |
|---|---|---|---|
| 1 | 整轮报告「`check-dead-config-flag --strict` rc=0」 | 那是 **`grep` 的退出码**；真实 rc=1 且确有新增违规。⛔ 报告门状态**一律** `cmd >/tmp/x.log 2>&1; echo $?`，禁管道 | 管道返回的是最后一个命令的码 |
| 2 | 「`check-arch-rules` 是幻影门，数据源从不存在」 | 数据源在 HEAD 里是 **tracked blob**；`probes/check-arch-rules.sh` 初版 `mv`+`rm -f` 把它从**工作树**删了，我读到的「不存在」是探针自己制造的 | 用工作树读数下**关于 HEAD** 的断言前，先查 HEAD |
| 3 | 「dot 目录也扫不到」「`codemap.json` 已陈旧」 | dot 目录循环一直是 `for d in */ .*/`，本来就扫得到（盲区**只有 dot 文件**）；codemap 是我拿工具总量比一个**不完整**的文件计数 | 比对前先确认两边口径一致 |

⇒ 三次同一形状：**用局部 / 单次证据下关于整体的断言。**

另一次无效验证也已留痕：曾把门断言改成 `if (false)` 得 rc=0 就宣称「检出力通过」——
**那条断言本来就不会响，什么都证明不了**。后改用真变异（桩返回空数组 ⇒ rc=1 准确报错）。

---

## 4. 一次真实提交事故（已修，值得记）

`777c7b0e` 用 `git commit --only` 只列了**新**路径，git 于是把改名记成「新增 3 个文件」，
而旧 3 个文件的**删除仍留在暂存区** ⇒ 若不补，历史里就是**同名两份实现同时存在**。
已用 `b00402f3` 补上。

⇒ 纪律：重命名提交必须**新旧路径都列**。

---

## 5. 留给接手者（按优先级）

1. ~~**`.freebuff/`**~~ —— **已由他窗自行处置**（写入 `.git/info/exclude`），
   `check-layout` 已恢复绿。⛔ 本窗口未代加白名单；留此条是为了记录
   「新根目录会被判红」这一行为已被实际验证过一次。
2. **`.worktrees/merge-b` 有 3 处未提交** —— 转告其主人。
   `.worktrees/evo` 的 `target` 3480M 门判零风险可回收（`nt_worktree_gate.sh clean`）。
3. **D5 的 schema** —— 框架已就位：`.neotrix/capability-blocking.json` 的
   `schema_source` 字段，业务侧把权威 schema 落到文件后**只需写路径**，
   门会校验它真的到位。⛔ 不需要改门、不需要适配器、不需要 agent 参与。
4. **`tests/full_pipeline.rs:232` 的 E0308** —— 自 09-29 起存在，非本窗口引入。
5. **UI 体检剩余两条**（低优先）：根目录**零标题层级**（读屏无文档结构）；
   气泡在深浅两档**逐字节相同**（对比度 9.99 / 15.15:1 均过 AA，属**刺眼**非不可读）——
   ⛔ 后者 `ABSORPTION-2026-10-07:256` 明写「需产品判断，不该由 agent 单方面改」。
6. **`cargo deny` 两条 warning** —— `fuchsia-cprng`（传递依赖无 license 字段）、
   `Unicode-DFS-2016`（allowance 未被用到）。非 error，属独立清理。

---

## 6. 一句提醒

本窗口把 25 道门接进了 `desktop` job，**但没有一道在真实 CI runner 上验证过** ——
本地全绿只证明「在有 Chrome、有 30 分钟构建的机器上绿」。
GitHub `macos-latest` 是**冷启动**（我已把 `waitForCdp` 的固定 sleep 改成轮询来对冲，
但那是本地验证）。**首次推送后请盯一眼 CI 的实际失败步骤**，
不要假设「本地绿 ⇒ CI 绿」——这正是本窗口第 1 条修复所针对的同一形状。