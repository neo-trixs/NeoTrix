# Handoff — HF 12 候选数据集吸收（2026-09-28）

## 结论

活库 **52,379,849 B / 64,674 条 → 56,637,045 B / 69,840 条**，新增 **5,166 条**（+7.40%）。
图体检 **PASS**，7 个新域逐域验收 **全 PASS**。

## 任务前提被勘察推翻（这是本轮最重要产出）

用户给出 12 个 HF 数据集要求"融入晶体核心"。先跑 `nt_hf_survey.py`（只读）：

- **总存储 78,066.92 GB ≈ 78 TB**。核心是单文件 JSON，1h tick 全量载入并**常驻内存**。
  TB 级数据集只可能采样，不可能整体灌。
- **9 个已在核心里**：2026-09-24 `nt_hf_rows_download.py` 各抓 100 行落在 `hf-rows` 域。
  照单重灌 = 直接制造重复。

### 处置矩阵（每条有实测依据）

| 数据集 | 存储 | 处置 | 依据 |
|---|---|---|---|
| genrobot2025/Gen-HumanEgo | 62.7 TB | 跳过 | 用户指令：TB 级跳过 + **401** |
| eidon-ai/tracker-pov | 9.0 TB | 跳过 | 用户指令：TB 级跳过 |
| IFM/Code-Reasoning | 3.3 TB | 跳过 | 用户指令：TB 级跳过 |
| Yootta/World-SimReady-Home | 3.0 TB | **跳过（强制）** | **401 Unauthorized**。用户要求"cc-by-nc-sa 吸收学习其经验"，但 gated 无令牌**物理不可达** —— 已向用户明说，非本脚本的选择 |
| malcolmrey/various | 6.08 GB | 跳过 | 实测仅 **33 行**且为 `image` 字典（**无文本**）+ 许可标 **wtfpl**。6GB 换 33 个图片 URL，蒸馏价值为零。**实测推翻自己原计划** |
| MoreThought/Fable-…-10000x | 8.59 GB | 吸收 816 | 吸 `full` split(=10000x) 而非 `lite`(=5000x)，因后者已有 100 行在库 |
| nyu-mll/glue | 4.01 GB | 吸收 419 | 用户"其他的直接做"。但它是**基准** → 只吸 `train`，**永不吸 `test`** |
| ZefanCai/Open-Jev | 85.7 MB | 吸收 431 | **cc0-1.0**；带标准答案的多选判分数据，正对已实测的"词法判分器方差上限 0.02"天花板 |
| LocalLLaMA/typed-decisions | 3.2 MB | 吸收 600 | `label_agreement.total_variation` 直接喂 `nt_jev_calibration` |
| Anthropic/hh-rlhf | 291 MB | 吸收 700 | 首行即有害请求 → 打 `[redteam]` 供下游过滤，**不丢弃** |
| FineEnvs/SmolDataEnvs | 9.7 MB | 吸收 1200 | 带 `reward_mode`/`atol` 的可验证 QA |
| AxiomicLabs/Tiny_Theory_of_Mind | 1.6 MB | 吸收 1000 | `activity_label` 编码 knowledge/ignorance × order × difficulty |

## 吸收结果

| domain | 条数 | 接入边 | 入度 max | 孤点 |
|---|---|---|---|---|
| grounded-qa | 1200 | 3381 | 4 | 0 |
| theory-mind | 1000 | 2797 | 4 | 0 |
| agentic-trace | 816 | 1630 | 2 | 0 |
| pref-pair | 700 | 1396 | 2 | 0 |
| decision-calib | 600 | 1192 | 2 | 0 |
| jev-choice | 431 | 1054 | 4 | 0 |
| nli-benchmark | 419 | 836 | 2 | 0 |

全库 213 茧 / 29 域 / 74,663 边 / `connected_ratio` 0.885→**0.894**。
**全库最大入度 171，吸收前后完全不变** —— 新增未制造 monoculture。
新增记忆入度分布 `{1:21, 2:3383, 3:1549, 4:213}`，均值 2.38，**入度 > 8 者 0**。

## 三条纪律（都来自本仓已修过的真实事故）

1. **不吸 test/ood split** —— Open-Jev 有 `test`/`ood`，glue 有 `test`，吸了就是基准污染。
2. **不做 hub-and-spoke** —— `nt_cocoons_prune_monoculture.py` 刚清掉入度 935 的锚点。
   故组内用**链式**连接（成员 i 连 i±1）。实测新增入度 max 4，多键成员各贡献 ≤2。
3. **对活库已有 content 精确去重** —— 实测跳过的重复：Open-Jev 571、Fable 184、
   hh-rlhf 2、Smol 1、glue 1。

## 干跑抓到的 4 个真实 bug（都先复现再修，R-SCAN-1/2）

1. **`stride = cap*2` 越过数据集末尾** —— ToM 仅 2000 行但步长 2000，第 2 个窗口就空，
   2000/1000 行的 split 只取到首批 100 条。改为**由 `num_rows_total` 导出步长**。
2. **cap 只在窗口后判定** —— hh-rlhf 溢出到 796/700、glue 500/420。改为行内判定。
3. **`IncompleteRead` 不在 except 元组内** —— Fable 返回大体积 agentic 轨迹时抛
   `http.client.IncompleteRead`（属 `HTTPException`，**不属** `URLError`），
   一次瞬时网络抖动直接终止整批。收敛为 `NET_ERR=(OSError, ValueError,
   http.client.HTTPException)` + 429 长退避。
4. **`NameError: mtype`（写盘路径）** —— 解包绑 `mt`、字典写 `mtype`。干跑在写盘前就
   return，**结构上覆盖不到**。已把记录构造抽成 `make_record()` 并由 selftest 直接调用，
   锁死该回归。

> 教训：干跑能覆盖取数/去重/上限，**覆盖不到写盘**。写盘路径必须有单测。

## 顺带修掉的体检工具失真

`nt_cocoons_verify_absorb.py:88` 原本用 `len(raw)/1e6` 报体积，而 `raw` 是**已解码
str** —— 字符数 ≠ 字节数。中文语料下把 `56,637,045 B` 误报成 `56.0MB`（真实 56.6MB）。
已改为 `os.path.getsize`，并同时打印字节数与字符数。
**体检工具报错体积比不报更危险**（R-SCAN-3：宁可精确，也不要陈旧/失真的门记录）。

## 验证

- `nt_graph_audit.py` → **VERDICT: PASS**（内容重复/非法类型/悬空/非M-id/超限茧/键不符 全 0，
  规范往返字节一致 True）
- `nt_lock_audit.py neotrix-core/src` → **0 处**，退出码 0
- 7 个新域逐域 `nt_cocoons_verify_absorb.py --prefix <域>` → **全 PASS**
- 备份：`~/.neotrix/crystal_core/cocoons.json.bak.hfbatch.20260928-134857`
- 两阶段提交产物：`datasets/hf_batch/.stage.json`（已复核的干跑预演，保留供回滚）
- manifest：`datasets/hf_batch/absorb_manifest.json`

## 未完成

- **Rust `CocoonStore::load()` 活库实证未跑**：`nt_mem_gate.sh` 返回 **2 (BLOCKED)**，
  另有 5 个 cargo/rustc 进程在跑（他窗 `neotrix-neobot` 仍在改）。门开后再跑：
  `cargo test -p neotrix --lib -- live_graph_index --ignored --nocapture`
- 推理链 58 个测试仍未编译（同一阻塞原因）。
