# Handoff — 晶体核心统一收口（2026-09-28 第 5 次会话）

> **本文是给「单窗口汇总所有窗口任务、统一修复」用的。**
> 本会话期间另有 2 个窗口在改 `crates/neotrix-neobot` / `apps/neobot-desktop`，
> 且已提交 `de6aa9ea`、`70356116`（动过 `TODO.md` 与 `.gitignore`）。
> 晶体侧与那些文件**无交集**，故本会话未触碰。
> 收口时请把三方任务合到一处排期，避免再出现「一个窗口等另一个窗口的门」。

## 0. 一句话状态

**数据侧已完成并实测；Rust 侧 8 个编译错误已修到只剩 1 个，那 1 个也已改完但尚未编译验证。**
**下一步只有一件事：等内存门开，跑一条命令。**

---

## 1. ✅ 数据侧：已完成，勿重做

```
活库  69,840 条 / 213 茧 / 29 域 / 56.6MB     （吸收前 64,674 / 205 / 22 / 52.5MB）
nt_graph_audit  VERDICT: PASS
7 个新域逐域验收  全 PASS
```

| 指标 | 值 | 意义 |
|---|---|---|
| 全库最大入度 | **171，吸收前后完全不变** | 新增**未**制造 monoculture |
| 新增记忆入度 | max **4** / 均值 2.38 / 零孤立 / 入度>8 者 **0** | 链式连接纪律成立 |
| `connected_ratio` | 0.885 → **0.894** | 略升 |
| glue `test` 占位标签 | **0 条** | 基准未污染 |
| Open-Jev 带真实对话 | **431/431** | §v4.2 的修复确实生效 |
| 全库 content 重复 | **0** | |
| 备份 | `~/.neotrix/crystal_core/cocoons.json.bak.hfbatch.20260928-134857` | 可回滚 |

**已提交**：`bc3bc100`（Python 1,616 行 / 6 文件）、`85a7f97d`（RFC v4 + AGENTS.md 门记录）。

### 12 数据集处置矩阵（每条有实测依据）

| 数据集 | 存储 | 处置 | 依据 |
|---|---|---|---|
| genrobot2025/Gen-HumanEgo | 62.7 TB | 跳过 | 用户指令 + **401** |
| eidon-ai/tracker-pov | 9.0 TB | 跳过 | 用户指令 |
| IFM/Code-Reasoning | 3.3 TB | 跳过 | 用户指令 |
| **Yootta/World-SimReady-Home** | 3.0 TB | **跳过（强制）** | **401 Unauthorized**。用户点名要「吸收学习其经验」，但 gated 无令牌**物理不可达**。**已于 2026-09-28 由用户指令关闭该任务**（不再作为待办） |
| malcolmrey/various | 6.08 GB | 跳过 | 实测**仅 33 行纯图像**（无文本）+ `wtfpl` 许可 ⇒ 零蒸馏价值。**实测推翻自己原计划** |
| MoreThought/Fable-…-10000x | 8.59 GB | 吸收 **816** | 吸 `full`(10000x) 而非已在库的 `lite`(5000x) |
| nyu-mll/glue | 4.01 GB | 吸收 **419** | 只吸 `train` —— 它是**基准**，吸 `test` 会真实污染 |
| ZefanCai/Open-Jev | 85.7 MB | 吸收 **431** | **cc0-1.0** |
| LocalLLaMA/typed-decisions | 3.2 MB | 吸收 **600** | |
| Anthropic/hh-rlhf | 291 MB | 吸收 **700** | 2 条打 `[redteam]` |
| FineEnvs/SmolDataEnvs | 9.7 MB | 吸收 **1200** | |
| AxiomicLabs/Tiny_Theory_of_Mind | 1.6 MB | 吸收 **1000** | |

---

## 2. ⬜ Rust 侧：精确状态（**这里最容易误判，务必读完**）

### 精确状态

```
第 1 次编译（14:32）  8 errors
  ↓ 我修 6 处
第 2 次编译（14:52）  1 error  (E0716 nt_awaken_loop.rs:252)
  ↓ 我修（let pm_kw 绑定）
第 3 次编译          ← 从未跑过
```

**第 2 次的 8→1 是已验证的事实；第 1→2 的 6 处修复是已验证的；
只有最后那 1 处（`let pm_kw`）未经任何编译验证。**
不要以为「8 个错都修完了」——修完了，但最后 1 处没见过编译器。

### 唯一需要的命令

```bash
sh scripts/ops/nt_mem_gate.sh; echo "gate=$?"    # 必须 0
CARGO_BUILD_JOBS=2 cargo test -p neotrix --lib -j 2 -- nt_crystal_core --test-threads=2
```

⚠️ `echo $?` **不可接管道**（会取到 `tail` 的码 —— 本会话已误报过一次 `gate=0`）。

### 未提交文件（全部）

```
 M neotrix-core/src/neotrix/nt_crystal_core/cocoons.rs          # 内容去重两阶段化 + 链落盘 + 原子 save
 M neotrix-core/src/neotrix/nt_crystal_core/mod.rs              # 注册 nt_graph_index / nt_premise_selector
 M neotrix-core/src/neotrix/nt_crystal_core/nt_awaken_loop.rs   # 前提接线 + D8/D9/coverage/饱和 + 判分
 M neotrix-core/src/neotrix/nt_crystal_core/nt_graph_index.rs
 M neotrix-core/src/neotrix/nt_crystal_core/nt_jev_calibration.rs  # grounding → coverage 改名
?? neotrix-core/src/neotrix/nt_crystal_core/nt_premise_selector.rs  # 新增 883 行
?? scripts/ops/nt_jev_live_eval.py                                  # 本轮新增，可立即提交
```

### 已知风险点（Agent 交班时标的，**尚未实证**）

1. `nt_premise_selector.rs` 的借用/部分移动模式（`Option<&Adjacency>` 缓存）
2. `cocoons.rs` 的 `StoreData.chains` / 原子 save / 内容去重
3. `nt_jev_calibration.rs` 的 `grounding → coverage` 改名是否所有消费方都改了
4. 58 个目标测试：`sync_dedup` / `chain_persist` / `nt_premise_selector`(14) /
   `verify_incentive` / `nt_graph_index`(9)

### 唯一缺的硬证据

`CocoonStore::load()` 对**新 69,840 条**核心的加载实证。
52.5MB 时证过一次，**吸收后未重证**。JSON 契约层已全过
（`StoreData` 键 / memory 契约 / M-id / 悬空 0），但「能加载」要有二进制说话：

```bash
cargo test -p neotrix --lib -- live_graph_index --ignored --nocapture
```

### 为什么一直没跑成

包 `neotrix`（= `neotrix-core/` 目录）**硬依赖 `neotrix-neobot`**，
**没有绕开它单独测晶体代码的路径**。而他窗 `neotrix-neobot` 持续在改
（本会话期间先后报 `ToolName::ReadImage` 未覆盖、`no LastStep in nt_store_tasks`，
两次都是他窗中途态，他自行修好）。内存门因此反复 BLOCKED。

---

## 3. 🔴 跨模块实测发现：`keywords()` 对中文近乎失明

**这是本会话最重要的产出，且影响面远超晶体核心。**

`CrystalConsciousness::keywords()` 按**空白/标点**切分：

```
keywords("我的支付一直失败收不到验证码") -> ['我的支付一直失败收不到验证码']   # 整句 1 个 token
keywords("支付网关")                     -> ['支付网关']                      # 永不相交
```

**且无词干还原**：`{"invoice"} & {"invoices"} == ∅` —— 纯复数差即判分器归零。

### 真实标答任务实测（`nt_jev_live_eval.py`，339 条可用 jev-choice）

| 上下文 | top-1 | MRR | 零交集 |
|---|---|---|---|
| 仅对话(state_json) | 43.9% | 67.9% | 265 |
| 仅题干 | 42.7% | 67.6% | 206 |
| **对话+题干** | **45.8%** | 69.4% | 196 |
| 无上下文(对照) | 38.1% | 63.8% | 339 |

**随机基线 38.1%。最好组合仅 +7.7 个百分点，78% 条目零词面交集。**

⇒ 不是「分数不够好」，而是「**这个任务上纯词法几乎没有信息可用**」。
**词法路线在此任务无实用价值 ⇒ judge 必要。**
数据已到位：jev-choice 431 + decision-calib 600 + agentic-trace 816。

> ⚠️ **勿与 RFC §v4.2 混淆**：「补上对话后 79,116 行不再塌缩成 ~80 条」是**区分性**
> 结论（由去重实测：2400 行中 2318 行自重复，单独成立）；上表测的是**词法判别力**。
> 两者是不同性质，**上表不能推出「对话无用」**，只说明纯词法用不上对话。
> （工具已内置该区分措辞，防止下一个人误读。）

---

## 4. 📌 可复用教训（本会话付出代价换来的）

1. **干跑覆盖不到写盘。** 取数/去重/上限都能干跑验，但写盘路径在写之前就 `return`。
   首版 `NameError: mtype`（解包绑 `mt`、字典写 `mtype`）就死在这 —— 若信了
   「干跑全绿」就会带着坏代码上线。**已抽出 `make_record()` 由 selftest 直接调用**，
   把写盘路径纳入单测覆盖。
2. **先勘察后吸收。** 78TB 的清单若直接开灌，会浪费数小时并污染 52MB 核心。
   一次只读 survey（12 个 API 调用）就把可执行性/许可/gated/体量全部定死。
3. **体检工具报错比不报更危险**（R-SCAN-3 精神的又一例）。`nt_cocoons_verify_absorb.py`
   用 `len(raw)/1e6` 报体积，而 `raw` 是已解码 `str` —— 字符数 ≠ 字节数，
   把 56,637,045 B 误报成 56.0MB（真实 56.6）。已改 `os.path.getsize`。
4. **门记录必须带核实时间戳。** AGENTS.md 写「本轮仅改文档未改码故沿用同值」，
   而本轮**确实改了 Rust 码** ⇒ 该理由不成立，已重测补记（2026-09-28 14:49，0 条）。
5. **评估必须给机会水平。** 全零交集时若「同分保序取第一个」，等于白送 index 0
   一个正确（而金标常在 index 0），准确率虚高。须按 1/n（MRR 按 H_n/n）计入。
6. **别用管道取 `$?`。** `sh gate.sh 2>&1 | tail -1; echo $?` 取的是 `tail` 的码 ——
   本会话据此误报过一次 `gate=0`（实际 BLOCKED）。
7. **异常元组要对齐真实继承树。** `http.client.IncompleteRead` 属 `HTTPException`，
   **不属** `URLError` ⇒ 漏掉它会让一次瞬时网络抖动终止整批。

---

## 5. 需要用户/主 agent 决策

| # | 事项 | 状态 |
|---|---|---|
| 1 | ~~`Yootta/World-SimReady-Home` 授权~~ | **已关闭（2026-09-28 用户指令移除该任务）**。处置见 §3 矩阵「跳过（强制）· 401」—— 那是**已做的决定**，不再是待办 |
| 2 | **32 个未跟踪的 `scripts/ops/` 脚本** | 整个目录未入库（含 `nt_graph_audit.py` / `nt_cocoons_dedupe_ids.py` 等活路径）。本会话**只提交了自己那 6 个**，未扫他人文件。是否全量入库待决 |
| 3 | hh-rlhf 红队策略 | 2 条已打 `[redteam]`。下游过滤，还是当拒答样本留用？ |
| 4 | 8 个测试失败 | **已全部登记在册**（`sessions/handoff-test-debt-20260928.md` + `TODO.md` + `scripts/test-failures-baseline.txt`），本会话实测与账本**逐条吻合** ⇒ **非新增债，不必重查** |
| 5 | 修 `keywords()` 的 CJK 失明 / 无词干 | 跨模块影响，建议独立一轮，优先级高于 judge |

## 6. 本会话产生的未提交文件汇总

- `neotrix-core/src/neotrix/nt_crystal_core/` 5 改 1 增（见 §2）
- `scripts/ops/nt_jev_live_eval.py`（新增，selftest 绿，可立即提交）
- `TODO.md`（本轮追加的「晶体核心」节，**未提交**）

## 7. 复现命令速查

```bash
# 勘察（只读，12 个 API 调用）
python3 scripts/ops/nt_hf_survey.py --selftest && python3 scripts/ops/nt_hf_survey.py

# 干跑 + 落 stage（不写活库；一轮约 25 分钟，Fable 最慢）
python3 scripts/ops/nt_hf_batch_absorb.py --selftest
python3 scripts/ops/nt_hf_batch_absorb.py --dry-run

# 消费已复核 stage 落盘（不重新联网）
python3 scripts/ops/nt_hf_batch_absorb.py --commit

# 验收
python3 scripts/ops/nt_graph_audit.py
python3 scripts/ops/nt_cocoons_verify_absorb.py --prefix jev-choice --conf-cap 0.80
python3 scripts/ops/nt_lock_audit.py neotrix-core/src

# 判别力实测
python3 scripts/ops/nt_jev_live_eval.py --selftest && python3 scripts/ops/nt_jev_live_eval.py
```
