# 交接：原子拆解 + 行为对位（建议 2 关闭）— 2026-09-30

> 窗口任务：外部技术检索 → 逆向推理 NeoTrix 审计等基础能力缺失 → 完善全域 map 每链路
> 缺陷与进化路线 → 目标能力「**原子级拆解任何技术逻辑**」+「**复现对方产品**」。
> 分支 `feat/capability-absorb-20260828`，本会话 3 笔提交（均**仅本地，未推送**）。

---

## 1. 做了什么（3 笔提交）

| 提交 | 内容 |
|---|---|
| `4f2bc704` | `--units` 逐族分诊 + 修**字节估 token**（喂给 T4 档位门槛）+ 修一个**稳定复现**的测试竞态 |
| `0459494b` | **副本漂移审计器**（`nt_fn_drift`）+ 抓到 3 个真缺陷：2 处**中文 panic** + 1 处时钟 panic |
| `1db3234f` | 登记 `map-reconcile` + TODO 记边界澄清（**并非所有能力都可复现**）|
| `e9f74b4c` | **全域 map 装上自检器**：每条「现状」带可执行断言（9/9 HOLDS）+ 3.2 边界章 |
| `144c59a8` | **行为对位通用化 + 第一个真 bug**：中文实体链接按字节算编辑距离（三处单位混用）|
| `6a4309fb` | 行为对位腿：37/37 步与 `lru@0.18.5` 一致；`ResponseCache::contains()`；P0→P1 改判 |
| `519d78b9` | `nt_decompose.py`（原子拆解 + 名字级对位）；首次跨仓对位抓到 O(n) 淘汰 |
| `6a91b77b` | ROUND23 吸收 5 源（oil-ui 已在前一笔 `8810b0dc`）；`nt_calledges --compact` |

**净能力增量**（此前只有文档、无工具）：

```
原子级拆解  = nt_decompose.py atoms  --db <edges> --root <sym>   （入口 → 原子记录 + oracle）
名字级对位  = nt_decompose.py parity --mine A --theirs B         （能力矩阵）
行为级对位  = nt_parity_ref.py --vectors <f> [--check|--list]    （采对方 oracle，注册表）
              + neotrix-core/tests/nt_capability_parity.rs      （我方侧，同一文件多能力）
```
⚠️ `response_cache_parity.rs` 已删（并入 nt_capability_parity.rs）。

---

## 2. 关键结论（都可复跑证伪）

1. **语义等价已证**：`ResponseCache` 与 MIT `lru` 在 `put/get/len/contains` 全子集
   **37 步零分歧**（oracle 由跑对方实现采集，非人手写）。
2. **复杂度不等价**：我方淘汰 `iter+filter+min_by_key+remove` = 每次写 O(capacity)
   全扫描；对方 `attach/detach/swap` = O(1) promote。⇒ **性能债，非正确性债**（P0→P1）。
3. **名字级对位会误报，已实测**：它把对方 `get` 报成「仅对方有」，我方同义方法叫
   `cache`。⇒ 文档与 task-index 的 `when_not` 都已写明「异名 ≠ 缺能力」。
4. **本仓约束下的唯一可行修法**：引第三方 `lru`（MIT，已在 registry）。自建侵入式
   链表需手写 `unsafe`，被 `#![forbid(unsafe_code)]` 直接排除 —— 这本身是架构结论。

---

## 3. 三次自证错误（全部留档，别重犯）

| # | 错误 | 怎么发现的 | 处置 |
|---|---|---|---|
| 1 | `ls \| tail` 误导我以为最大轮次是 ROUND9，`write` 直接**覆盖了他人的 `ABSORPTION-ROUND20.md`** | 写入后 `git status` 显示 `M` 而非 `??` | 立即 `git checkout` 恢复（已验证零 diff），改写为 ROUND23。**`ls` 取尾不可靠，写前必须全量确认** |
| 2 | `contains` 测试我**手推错了** LRU 顺序（把 `insert a;insert b` 后的 LRU 写成 `b`） | 实跑失败 | 读实现 + 外部 oracle（c2 淘汰 `a`）确认**实现对、期望错**；改期望并把这段写进注释。又一次「手推 ≠ 实证」 |
| 3 | 用 Python `json.dump(indent=2)` 重写 `.neotrix/task-index.json` ⇒ **全文件重排 816 行** | `git diff --stat` | 改用 `edit` 工具按原 1 空格缩进外科式插入（+47 行） |
| 4 | 我写的 CJK 链接测试**不判别**：阈值取默认 0.6 时两种语义都合 ⇒ 恒绿 | 回退修复后测试仍绿 | 阈值改 0.7 才判别 |
| 5 | 同一测试选的「知识库/知识库务」是**子串对** ⇒ `names_match` 在包含分支就 return true，**走不到 Levenshtein** | 同上 | 换「知识库/知识阁」（非子串、差末字） |
| 6 | diff 报错信息把 flat 索引当 case 名（`[7]`） | 读输出对不上 case | 修成 (case 名, 该 case 内步号) |
| 7 | 采集器把「仅元数据变化」报成「oracle 说谎」 | 加 capability 字段后 `--check` 误报 | 区分观测漂移 / 元数据漂移（IDENTICAL） |
| 8 | `nt_map_reconcile` v1 用**行内正则**匹配谓词 ⇒ 全仓 **18 条假阳性**（把 `package.json` 的 `test:headed` 脚本名当断言） | 跑出来 18 条 FAIL，逐条看上下文才发现全是假的 | 改为只认 ```assert 围栏块 |
| 9 | 我写的 `nlit:…@本文件` 是**自指断言**：为解释改动必须引用旧措辞 ⇒ 永远失败 | 工具自己报「literal present」 | 删该断言 |
| 10 | 我写的 `cmd:nt_map_reconcile.py`（自检调用正在自检的工具）⇒ **无限递归**，120s 超时 | 工具自己超时 | 删该断言 |
| 11 | `nt_fn_drift` 输出把 tuple 解包成 `(mod,path,body,line)` 却按 `(mod,path,line,body)` 用 ⇒ `%d` 收到 str 首跑即崩 | TypeError | 改正解包顺序 |
| 12 | 把「字节估 token」的缺陷方向**算反**（说早 2/3 篇幅升档，实为该升没升） | 实测 bytes×0.3 中 bytes=chars×3 与 0.3 抵消 | 三处文档按实测更正 |
| 13 | `evm` 竞态的**第一次加固无效**：在 uses_default 开头 `remove_var`，实测仍 6/6 红 | remove 的仍是共享变量 | 消除共享（换变量名+换 chain）⇒ 8/8 绿 |

---

## 4. ⚠️ 共享工作树事故（`519d78b9`，必须让其他窗口知道）

**我的 commit 把另一窗口的 `.neotrix/task-index.json` 内容提交了，我自己的 3 条索引丢失。**

- 成因：`git commit --only <path>` 取的是**工作树状态**。我加完条目并验证
  `nt_find` 能命中；提交前另一窗口并发改写该文件（加 `nt-callgraph-impact` 等）
  ⇒ 我提交了他们的内容。
- 与 `sessions/handoff-commit-only-20260929.md` 是**不同**事故：那次是「暂存区核对与
  提交不原子」，这次是「**共享单文件本身被并发改写**」。
- ⇒ **教训**：`--only` 隔离「哪些文件」，**不隔离「文件里是什么」**。
- ⇒ 共享单文件（`task-index.json` / baseline / `layer-map.json`）提交前必须：
  `stat -f '%Sm'` 看 mtime + 重新 `grep` 自己的条目仍在。
- 现状：他们的条目**未丢失**（在我的 commit 里，工作树干净）；我的 3 条已在 `6a4309fb`
  补回。**他们若以为还没提交，会发现 `git status` 干净** —— 需要知会。

---

## 5. 门与验证（全部本会话实跑）

| 门/验证 | 结果 |
|---|---|
| `nt_decompose.py selftest` | ✅ 6 正例 + **3 证伪** |
| `cargo test -p neotrix --lib` | ✅ **12,213** passed / 0 failed（+4 新测）|
| `cargo test -p neotrix --test nt_capability_parity` | ✅ 3 绿（lru 37 步 + levenshtein 17 步）|
| `cargo check --all-targets -p neotrix` | ✅ 0 error；**我改的 2 个文件 0 warning** |
| `nt_lock_audit.py neotrix-core/src` | ✅ 0 处 |
| `check-silent-failure.sh --strict` | ✅ PASS（OPEN CONTRACTS 0/32） |
| `check-doc-drift.sh` / `check-layout.sh --strict` | ✅ 0 死链 / rc=0 |
| `nt_map_reconcile.py --strict` | ✅ **18/18 HOLDS**；证伪：植入 3 条假声明全部被抓（实测）|
| `nt_fn_drift.py selftest` | ✅ 7 例（含 3 例证伪）；实跑 DIFFERENT 160 / IDENTICAL 29 / UNRESOLVED 24 / 形状命中 7 |
| truncate 两处修复的证伪 | ✅ 回退后分别以 `byte index 5 is not a char boundary` 与 `attempt to subtract with overflow` 转红 |
| model_router 修复的证伪 | ✅ 回退后 `left: 30 right: 25`、`left: 504 right: 560` 转红；恢复后 13 绿 |
| evm 竞态的复现与验证 | ✅ 回退我的改动 6/6 红（稳定复现）；修法 8/8 绿 |
| `check-unwrap.sh --strict` | ❌ **4 条红 = 他窗 WIP**：`apps/neobot-desktop/src/core.rs:205`、`main.rs:124`、`crates/neotrix-neobot/src/nt_pet.rs:225`/`:226`。**未代改、未代记账** |
| `check-license.sh` | ❌ rc=1 = **正确状态**（他窗待裁决 `apps/neobot-desktop/frontend` 附加条款）。⛔ 不要为了让门变绿删 deny 名单 |
| 证伪：篡改 oracle | ✅ 测试红（`step 13: ours=… reference=…`） |
| 证伪：oracle 说谎 | ✅ `--check` rc=**1** |

---

## 6. 下一步（按杠杆，建议接手者按序做）

1. **P1 性能债**：`ResponseCache` 换 `lru` crate。独立一轮，**必须带 before/after 基准**
   （本会话只证了复杂度劣势，没测绝对耗时 —— 别把「O(n) 更差」当「真的慢」）。
2. **扩行为对位覆盖面**（现在很便宜：一份 vectors + 一个 adapter 分支）：
   - 未验证且**不预设有 bug**：`search_scorer::fuzzy_match`、`writing_style::fuzzy_similarity`。
     参考 crate 需自己找（registry 里未必有 ⇒ 可能得选别的参考源，**找不到就记「无法对位」，
     不要拿手写期望值冒充对方行为**）。
   - 未覆盖：`key_for` 哈希稳定性、`prefetch` / `prefetch_lookahead`、kb_search 排序、
     outbox 毒行处置（后者已有 `nt_channel_serve` 边表可分解）。
   - ⚠️ **`--only` 提交且含删除时**：`check-commit-deletions` 读不到 `--only` 的
     message（脚本头注已记此互锁）。先把消息写入 `$(git rev-parse --git-path
     COMMIT_EDITMSG)`，再 `git commit --only ... -F <file>`。
3. **doc-claim 门新类目**：`nt_policy.rs` 单文件 **23 处** `/// Note: Real implementation
   needs —`，已逐行核实 ≥4 处是**假声明**（函数体完整）。按 G7 先确认模式够窄再考虑建门。
4. **建议 3（map 从结构升到意图）**：入口图 + doc 承诺对账，尚未做。
5. **G3 跨仓查询合并**：`nt_callgraph.py`（他窗建的）仍是单库；跨库只在 `nt_decompose parity`
   里以「两个 DB 各跑一遍」的形式成立。

---

## 7. 复跑清单

```sh
python3 scripts/ops/nt_parity_ref.py --list              # 已注册能力
python3 scripts/ops/nt_map_reconcile.py --strict            # 地图自检（0 断言 ⇒ rc=2）
python3 scripts/ops/nt_fn_drift.py selftest                 # 副本漂移审计器自证
python3 scripts/ops/nt_fn_drift.py --only-different         # 分诊单（[SHAPE] 优先）
python3 scripts/ops/nt_fn_drift.py --list-units             # 待分诊的族
python3 scripts/ops/nt_fn_drift.py --units estimate_tokens # 该族每份副本的计数单位
python3 scripts/ops/nt_decompose.py selftest
python3 scripts/ops/nt_decompose.py atoms --db .project-map/edges-neotrix-neobot.jsonl \
    --root 'nt_channel_serve::run_once' --depth 3        # 193 原子
python3 scripts/ops/nt_parity_ref.py \
    --vectors .neotrix/parity/response-cache.vectors.json --check
cargo test -p neotrix --test nt_capability_parity
```

⚠️ 边表 `.project-map/edges-*.jsonl` 是 **gitignored 低频生成物**（全量 ~30min）。
干净检出上**没有**它 ⇒ `nt_decompose` / `nt_parity_ref` 会以「no such edge db」退出。
先跑 `nt_calledges.py --crate <dir> --out <f>` 生成。

---

## 8. 收工自查（必填）

### 8.1 worktree 去向

```
[worktree-gate] worktree=3 个 | 合计 4840M | target 占 4673M
[worktree-gate] 带未提交改动: 2 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 2 个 worktree 的未提交改动**不在任何提交里**
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
[worktree-gate] ♻️  target 累计 4673M ≥ 1024M ⇒ 零风险可回收：sh scripts/ops/nt_worktree_gate.sh clean
```

| worktree | 归属 | 去向 |
|---|---|---|
| `.worktrees/merge-b` | **他窗**（脏：Cargo.lock / CODE-TOPOLOGY.md 等） | ⛔ 未动。删前须 patch 兜底 |
| `/private/tmp/nt-v9` | **他窗**（脏） | ⛔ 未动 |
| `.worktrees/nt-stop` | 干净（`b9be70d9`，落后 main） | 保留，非我建 |

**本会话未新建任何 worktree**（全程在主工作树最小改动）。`nt_sidecar` = DOWN（按需即用）。
临时目录已清：`/tmp/nt-parity`（外部 crate 副本 + 自建 cargo 工程）、
`/tmp/nt-decompose-selftest`、`/tmp/ref-backup.json` 等。

### 8.2 未提交改动的去向

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `scripts/ops/nt_decompose.py` | 原子拆解 + 对位 + selftest | ☑ 已提交 `519d78b9` |
| `docs/architecture/DECOMPOSE-PARITY-2026-09-30.md` | 工具文档 + 缺口证据 | ☑ 已提交 `519d78b9` / `6a4309fb` |
| `.neotrix/task-index.json` | +3 条索引（`nt-decompose-atoms`/`-parity`/`nt-parity-ref`） | ☑ 已提交 `6a4309fb`（+47 行，外科式） |
| `TODO.md` | P0→P1 改判 + 事故留档 + 23 处假声明 | ☑ 已提交 `6a4309fb` |
| `neotrix-core/src/.../resilience/nt_policy.rs` | 新增 `contains()`（非变更式） | ☑ 已提交 `6a4309fb` |
| `neotrix-core/tests/response_cache_parity.rs` | 行为对位测试（单目标） | ☑ 已删（内容并入 `nt_capability_parity.rs`，`144c59a8` 带 DELETION-INTENT）|
| `neotrix-core/tests/nt_capability_parity.rs` | **通用**行为对位 harness（注册表驱动） | ☑ 已提交 `144c59a8` |
| `neotrix-core/src/.../entity_linking/linker.rs` | levenshtein 改按 char + max_len 同单位 + 2 条判别测试 | ☑ 已提交 `144c59a8` |
| `neotrix-core/src/.../nt_act_code/semantic_entropy.rs` | `char_similarity` 分母改 char + 回归测试 | ☑ 已提交 `144c59a8` |
| `.neotrix/parity/levenshtein.vectors{,.reference}.json` | 第 2 个能力的 op 脚本 + 对方 oracle | ☑ 已提交 `144c59a8` |
| `.neotrix/parity/*.json` | op 脚本 + 对方 oracle | ☑ 已提交 `6a4309fb` |
| `scripts/ops/nt_parity_ref.py` | oracle 采集器 + `--check` 漂移门 | ☑ 已提交 `6a4309fb` |
| `docs/architecture/ABSORPTION-ROUND23.md` | 5 源吸收记录 | ☑ 已提交 `6a91b77b` |
| `docs/architecture/absorption-sources/{repos.csv,LICENSES.md}` | +5 源 / 许可台账 | ☑ 已提交 `6a91b77b` |
| `scripts/ops/nt_calledges.py` | `--compact` 输出 | ☑ 已提交 `6a91b77b` |
| `skills/design/ui-direction/` | oil-ui 方法论（8 文件） | ☑ 已提交 `8810b0dc` |
| **他窗** `apps/neobot-desktop/**`、`crates/neotrix-neobot/**`、`.neotrix/capability_registry.json` 等 | 非我改动 | ⛔ **未动、未暂存**（`--only` 隔离） |
| **他窗** `.worktrees/merge-b`、`/private/tmp/nt-v9` | 非我 worktree | ⛔ 未动 |

**我的改动零遗留**（`git status --porcelain -- <我的文件>` 空）。

### 8.3 交接给下一位的三条硬约束

1. **`.neotrix/task-index.json` 是共享单文件** —— 提交前 `stat -f '%Sm'` + 重 grep 自己的
   条目（见 §4 事故）。**不要用 Python json.dump 重写它**（会重排全文件，+816 行）。
2. **不要给 parity/parity 矩阵建门** —— 名字级对位已实测误报（G7 裁决）；行为级只对位
   vectors 覆盖的 op，PASS ≠ 「行为完全一致」。
3. **两个模块曾有同名不同语义的 `levenshtein`**（linker 字节版 / semantic_entropy
   字符版）。已统一为按 `char`，但**这类「副本语义漂移」是复发型问题** —— 再见到
   同名函数先对位再合并，别只做文本去重。
4. **⛔ 不要照 fn-drift 的 `[SHAPE]` 标记批量改代码** —— `--units` 也只是**候选**
   （`.len()` 可能是集合长度）。本轮已据此**判 2 族无罪并放过**
   （`truncate_chars` 的字节快路径无害、`tokenize` 规则有意不同）。
5. **⛔ 清理测试的共享全局状态（env/单例/全局计数器）时，
   必须消除「共享」本身** —— 仅 `remove_var` 开头无效（本轮实测 6/6 红）。
6. **⛔ 不要照 fn-drift 的 `[SHAPE]` 标记批量改代码** —— DIFFERENT 里绝大多数是
   「同名不同域」的合法重复（`osint::investigate` ×15）。形状标记只用于**排序优先级**。
   同理 `306 处 "Real implementation needs"`（49 文件）**不得批量删**：抽查 5 处全假，
   但其余 283 处未取证 ⇒ 局部修改会让文件一半真一半假，更难判断。
7. **地图/台账的断言只写在 ```assert 围栏块里**，且**绝不可自指**
   （不自检自己、不用 nlit 检查所在文件）。工具 `map-reconcile` 在索引里。
8. **用户指令里有一条被拒收项**：搜 GitHub 公开 `OPENAI_API_KEY` 批量密钥 —— **拒绝执行**
   （凭证收割）。已写入 `ABSORPTION-ROUND23.md` 声明。如再次出现，同样拒绝。
