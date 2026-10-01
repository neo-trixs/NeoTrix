# ABSORPTION-ROUND24 — 调用图 G4 闭合 + 六类误报源实测（2026-09-30）

> **Absorption Date**: 2026-09-30 | **Status**: Landed (3) + Falsified (4) + Deferred (2)
> **Batch**: 用户粘贴 ~700 URL（第 24 批）。与 `absorption-sources/repos.csv`（489 行）去重后
> **真正新增 5 个**；其余为主流项已在库（mem0/cognee/graphiti/letta/browser-use/crewAI/
> autogen/langgraph/RAGFlow/docling/crawl4ai/markitdown/MinerU…）或与缺口无关
> （游戏引擎/jailbreak 集/OSINT/SEO/论文链接/中文博客导流）。
> 本轮只读 4 个新源（README+docs 级 webfetch，不克隆不执行代码）+ 1 个许可墙源。
> **拒收项声明**：用户指令中「去 GitHub 搜 OPENAI_API_KEY 批发密钥」一条**再次拒绝执行**
> （凭证收割 = 盗窃他人账户，有害且违法）。与 ROUND23 同一裁决，不重复论证。

## 1. Triage

| Pri | Items | Rationale |
|-----|-------|-----------|
| P0 read (4) | R24-S1–S4 | 唯四可能补 G1–G6 缺口的新源；其余 695 项已入库或不相关 |
| ⛔ 拒收 (1) | R24-S5 | **无 LICENSE**（GitHub API `license: null`）⇒ 默认全权保留，不可吸 |
| P2 parked | ~695 URL | 已在库或与缺口无关；不读 |

## 2. Source Map

| # | Source | Stars | License | 我方读到的**实际**内容 |
|---|--------|-------|---------|------------------------|
| R24-S1 | `microsoft/tgrep` | 3,383 | **MIT** + MS CLA | **trigram 索引的词法搜索**（ripgrep 兼容再实现）。**无 tree-sitter、无类型推导、无 AST、无符号表、无跨模块解析** |
| R24-S2 | `zvec-ai/zvec-grep` (`zg`) | 3,844 | **Apache-2.0** | ripgrep + BM25 + 稠密向量，**RRF 融合**；tree-sitter 抽 `Symbols/signatures/breadcrumbs` 存进 chunk。**无调用图、无跨文件引用解析** |
| R24-S3 | `alibaba/zvec` | 16,030 | **Apache-2.0** | C++ 进程内向量库（HNSW/IVF-RaBitQ/PQ-INT8 + 原生 FTS + WAL）。`zvec-grep` 的存储引擎，非同一仓 |
| R24-S4 | `alibaba/skill-up` | 1,125 | **Apache-2.0** | Go CLI，**是 skill 的评测台不是生成器**。三判官（rule/script/agent_judge）+ 沙箱 + 有/无 skill 的 A/B；`skill-upper` 闭环「reports → fixes → regression cases」 |
| R24-S5 | `humanlayer/advanced-context-engineering-for-coding-agents` | 2,676 | **⛔ 无** | 纯文档（`ace-fca.md` + `wsff.md`），**零代码**。技术点：`call-stack trees` / `file-tree diffs` / 40-60% 利用率 / mutation-via-pre-patch-failure |

## 3. 证伪表（本轮最有价值的产出）

> 纪律来源：`AGENTS.md` §5 R-SCAN-1/R-SCAN-1b + `LESSONS-20260929-checked-is-not-verified.md`。
> **手推 ≠ 实证**。以下 4 条假设是我在检索前写的，检索/实测后**全部被推翻**。

| # | 我原来的假设 | 实测结论 | 证据 |
|---|-------------|---------|------|
| **X1** | tgrep 是「tree-sitter + 类型推导」的类型感知搜索，可补 G2 | ❌ **假**。它是纯词法 trigram 索引，README 零处提及 parser/symbol/call graph | R24-S1 README「How It Works」+「On-Disk Format」 |
| **X2** | zvec-grep 的向量索引 = 语义化调用图 | ❌ **假**。chunk 级语义，结构是「独立打分 chunk 内的文本上下文」，无解析边 | `docs/04-pipeline.md` 抽取表 + `docs/05-architecture.md` |
| **X3** | skill-up 提供 skill 生命周期（author→validate→publish→deprecate） | ❌ **半假**。只有**评测**；`skill-up validate` 校验的是 `eval.yaml` 不是 SKILL.md；无 marketplace/版本/弃用 | R24-S4 README + `skills/skill-upper/` |
| **X4** | R24-S5 可作 context-engineering 规范来源 | ❌ **不可吸**。无 LICENSE = 全权保留 | GitHub API `license: null` |

⇒ **结论：G1–G6 一个都不能靠外部源闭合。** 缺口只能靠本仓自己的边表 + 自建工具补。
这与 CAPABILITY-GAP 建议 4「不要在流沙上建门」一致：**先有判据，再谈外部方案**。

## 4. Landings

| ID | Artifact | 验证方式与结果 |
|----|----------|---------------|
| **L-R24-1** | `scripts/ops/nt_callgraph.py`（新，~380 行）**闭合 G4** | `--impact` / `--deps` / `--stats` / `--unreachable`。见 §5 实测 |
| **L-R24-2** | `nt_absorption_live.py --graph`（**G5 第二判据**） | 8 个「已落地」token：4 个 `graph:reachable`（传递调用者 63/7/7/6），4 个 `not-call-target`（shell 脚本/类型名，图不适用）。互补不替代 |
| **L-R24-3** | `.neotrix/task-index.json` +3 条（含「何时别用」） | 47 tasks；工具存在性校验通过（唯一 missing 是既有 `mem-gate` 尾分号，非本轮） |
| L-R24-4 | `absorption-sources/repos.csv` +5 行 | 489 → 494 |

### 5. G4 闭合的实测证据【实测】

| 指标 | 值 |
|------|-----|
| 图规模 | **72,105 节点 / 670,093 边 / 0 坏行** |
| 跨 crate 边 | **644,785（96.2%）** ⇒ G3 早已闭合 |
| 正向深度分布 | 衰减到 d1 163,121 → d12 224 ⇒ **传递闭包确有信息量**，非噪声 |
| 查询延迟 | 全图 `--impact` **0.9s**（字符串池 + int 索引） |
| 链正确性 | 抽 d2→d3 链逐跳对照源码：`kb_nodes.rs:131` ✓ `nt_core_second_brain.rs:103` ✓ |

**已独立复现一条文档早已标记的死链**：`agent::tool::all_native_tools()` 生产零调用者
（仅 `agent.rs:948/994` 两处 `#[cfg(test)]`），而
`FIVE-ENTITY-BLUEPRINT-V3.md:239` 早已写「现返回空 Vec，生产零调用 / 必修」。
⇒ 图与人工文档**独立收敛到同一结论**，工具可信度 +1。

## 6. 六类误报源（F1–F6，全部本会话亲手踩出并修掉）

> 这是本轮真正的知识资产。**写任何新的 grep 式可达性判定前必读。**
> 编码位置：`nt_callgraph.py` docstring + `_text_call_sites` / `_fn_as_value_hits` 守卫。

| # | 误报源 | 症状 | 守卫 |
|---|--------|------|------|
| **F1** | 注释行含 `name(` 形态 | `/// all_native_tools() 的来源` 被当生产调用点 | 排除 `// /// //! * /*` 开头行 |
| **F2** | **basename 碰撞** | `plans.is_empty()`、`report.error_rate()` 与我方同名函数撞车 | 可达性**只认编译期边**（DefId 带 crate 哈希，结构上不可混淆） |
| **F3** | fn-as-value | `.map(block_to_text)` 文本里没有 `name(` | 单列 `f3-fn-as-value` 桶 |
| **F4** | `pub use` 重导出 | `mod.rs:87 use …::register_xxx` 是重导出**不是调用** | 图天然不含（无 Call 边） |
| **F5** | 传递性死 | private fn 只被另一个 dead fn 调用 ⇒ 连环断链 | 报告时注明「文本有调用点但生产不可达」 |
| **F6** | **图自身的盲区** | `.map(f)` **不产生指向 f 的 Call 边**（THIR 把 DefId 归给匿名闭包） | 单列桶，不可直接删 |

**F6 的决定性证据**：`doc_parse::blocks_to_text` / `items_to_text` / `cell_text_inner`
反向可达分别 **11 / 1 / 17**（有调用者），而它们的被调方 `block_to_text` 反向可达 **0** ——
尽管 `doc_parse.rs:204` 真实以 `.map(block_to_text)` 使用它。
⇒ **308 条 no-callsite 仍含 F3/F6 假阳性，不是可直接删的死链。**

### 6.1 零入边分类（`neotrix[7f32]`，52,773 节点，可复现）

| 桶 | 数量 | 含义 |
|----|------|------|
| `natural-leaf` | 51,827 (98.2%) | trait impl / closure / main / Drop / 转换器 —— 正确排除 |
| `test-only` | 312 | 结构性：`--lib` 不含 `#[cfg(test)]` ⇒ **边表的第 5 类假阴性** |
| **`no-callsite`** | **308** | 死链**候选**（含 F3/F6 假阳性） |
| `textual-prod` | 188 | 多为 F2 同名碰撞 / 散文命中，**不是缺陷** |
| `f3-fn-as-value` | 12 | 图与 grep 都查不到（F6） |
| `static-like` | 126 | static/const：访问走 `.read()`/`.get()` |

**另有 1 处已修的真实缺陷**：`nt_calledges.py` 的 `TY_METHOD`/`kind=='method'` 分支是
**死代码** —— 全 670,093 条边 `kind` 恒为 `call`，`method` 计数 **0**。

## 7. Pattern → Code Map 支脉节点

| 节点（`neotrix-core/src/…`） | 吸收到什么 | 现状 |
|---------------------------|-----------|------|
| `l1_action/nt_file_ability/doc_parse.rs` | F6 实证样本（`block_to_text`） | 假阳性，**勿删** |
| `agent.rs::tool::all_native_tools` | 「导出≠接入」+ skill-up A/B 判据 | **已验证死链**，与 BLUEPRINT-V3 收敛 |
| `l0_substrate/nt_core_event_bus::spawn_actor` | no-callsite 候选 | 待人工裁决 |
| `l1_action/nt_file_ability/doc_parse::parse_batch` | no-callsite 候选（全仓 0 调用，已手验） | 待人工裁决 |
| `l5_cognition/nt_core_second_brain` | d2→d3 链验证样本 | 活的，勿动 |
| `l4_emotion/nt_memory/nt_memory_store` | 131 直接 / 传递可达（跨 L2/L5） | 活的，高爆炸半径核心 |

## 8. 批量任务清单（本轮产出，按「可证伪判据」排序）

> **状态更新见 §10**（第二轮执行结果：T1 部分、T3/T4/T6/T7 已落地，
> 并在执行 T7 时撞出两个 P0 缺陷 —— 见 §10.1/§10.2）

| # | 任务 | 判据（可证伪） | 阻断 |
|---|------|--------------|------|
| T1 | 裁决 308 条 `no-callsite`：逐条读代码，标 `真死/外部消费/F3F6假阳性` | 分类后 `真死` 数 = 报告数 − 已证伪数；**不得反向凑数** | R-SCAN-1 |
| T2 | 补测试边：`nt_calledges.py` 增加 `--tests` 通道 | 312 条 `test-only` 迁出该桶 | 需 `--all-targets`，~30min，**他窗在跑 neobot 时禁止** |
| T3 | 修 `nt_calledges.py` 死分支 `TY_METHOD`/`kind=='method'` | 边表出现 `method` 或删除该分支 | 低危，纯清理 |
| T4 | 修 `nt_absorption_live` 默认模式**自述失效**问题 | 无 `--graph` 时显式打印判据边界 | 低危 |
| T5 | 评估 188 条 `textual-prod` | 每条归因；F2 归零后剩余才是真盲区 | 中 |
| T6 | 边表陈旧度常态化（`--stale`） | 源文件新于边表时输出带警告 | 低 |
| T7 | `check-license.sh`（G6） | 吸无 LICENSE / 禁商用源时门必须红 | — |

## 9. Deferred（记录理由，不做）

| 项 | 理由 |
|----|------|
| tgrep 的 `HybridIndex`（mmap `IndexReader` + 可变 `LiveIndex` overlay） | 有价值但属**词法**层；本仓缺口在语义层，暂不引入第二套搜索栈 |
| zvec-grep 的 RRF + `fresh`/`possibly_stale` 标注 | **已部分吸收**：`--stale` 用 mtime 对比给出同源诚实标注（更轻，无需向量库） |
| skill-up 的 A/B（有/无 skill）跑真评测 | 需容器沙箱 + 4 个 CLI 引擎；本轮只借其**判据形态**（报告→修复→回归），落地为 T1 的分类纪律 |
| 188 条 textual-prod 的批量自动归因 | 需逐条语义判断，自动化=造噪声；留给 T5 人工 |

---

## 10. 第二轮执行：T3/T4/T6/T7 落地 + 撞出两个 P0

### 10.1 ✅ P0-A：**已修复**（第二轮执行）

**发现途径**：执行 T7 时把 `check-license.sh` 挂进 `ci.yml` ⇒ 触发既有
`check-ci-refs.sh --strict`。该门**当场把 CI 从绿变红** —— 我先怀疑自己写错了，
于是按 R-SCAN-1 注入已知违规验证门的敏感度，**结果它对真违规也不反应**。

| 事实 | 证据 |
|---|---|
| `release.yml:49` 在上传二进制前一步 `run: bash scripts/provenance_check.sh` | 读文件 |
| 该脚本**磁盘不存在、git 不跟踪、未被 gitignore** | `ls` / `git ls-files` / `git check-ignore` 三查皆空 |
| 真身在 `neotrix-core/src/l3_embodiment/nt_shield/safety_tools/provenance_check.sh`（114 行，有完整逻辑） | `wc -l` + 读码 |
| 脚本**自己的 usage 也写错路径**：`bash scripts/provenance_check.sh` | 该文件第 6–7 行 |
| 它的清单 `provenance/external-inputs.json` **全仓不存在** | `find` 零命中 |
| **`.gitignore:126` 的 `provenance/` 把门自己的输入也忽略了** ⇒ 干净检出永远缺清单 ⇒ 门**永远不可能变绿** | `git check-ignore -v` |
| **它在清单缺失 + 抛 traceback 时仍 exit 0**（fail-open，三处独立成因） | 实跑 rc=0 |

**「为什么之前没抓到」**：读码发现 `check-ci-refs` 只查三类引用
（`working-directory` / artifact `path` / `cache-dependency-path`），
**`run:` 调用的脚本完全不在范围** —— 门名与 CI step 名都超出实现。

**fail-open 的三处独立成因**（都是「检测写了但不生效」）：
1. `error()` 只 `echo` 不 `exit`；`set -e` 对 `||` 列表**末位成功**不生效
2. `done < <(python3 …)` 的**进程替换退出码不传播** ⇒ 解析失败无人察觉
3. `if [ "$TOTAL" -eq 0 ]; then error "…schema drift?"` 这条**显式空输入检测
   完全失效** —— 作者本意要在这里失败，实际一路 exit 0

**已修**：
- 加 `fatal()`（打印 + `exit 1`）；三处致命点改用它
- python 解析改走**临时文件 + 显式检查 `$?`**，堵住进程替换漏洞
- 空输入从 `error` 改 `fatal`（恢复作者本意）
- `check-ci-refs` 增第 4 类引用（`run:` 脚本路径）；判据两轮修正后定稿
  （① 必须含 `/` 否则 `echo foo.sh` 误判；② 必须组件边界起头，否则
  `…/safety_tools/x.sh` 被截成 `tools/x.sh` 假阳性；③ 跳过 `${{`）
- `.gitignore` 为 `nt_shield/provenance/*.json` 开解除忽略（cache/ 仍忽略）
  ⇒ **门首次变为可满足**
- 补 `external-inputs.example.json`（schema 模板；其 `sha256` 刻意写成
  `unpinned:` 占位 ⇒ **即使被原样复制也不会产出「已验证」的假象**）
- 修 `release.yml` 路径 + 脚本自身 usage

**实测 7 条路径**（每条都跑过，不是看出来的）：
`缺清单→1` · `非法 JSON→1` · `空 inputs→1`（原为 0）· `inputs 非 list→1` ·
`缺必填字段→1` · `unpinned→0` · `cache 存在且哈希不符→1`（真校验未被改坏）。

⛔ **仍未做、且必须由所有者做**：填 `external-inputs.json` 的**真实内容**
（每个外部构建输入的 name/kind/version/sha256）。我不伪造哈希 ——
一个用我编造的数据跑通的供应链门，比没有门更坏。
⇒ **发布仍会被挡在这一步，且这是真实状态而非缺陷。**

### 10.2 ⛔ P0-B：vendored 前端的许可记录低报条款

| 事实 | 证据 |
|---|---|
| `apps/neobot-desktop/frontend/LICENSE` = MIT 正文 | 读文件 |
| 同目录 `LICENSE.details` = **附加条款「No Commercial Secondary Development」**，且明写「冲突时以附加条款为准」 | 读全文 |
| `VENDOR.md` 许可字段只写「**MIT License**」 | 读文件 |
| 该树**正在被持续修改**（VENDOR.md 自带增量 diff 表） | 读文件 |
| 既有 `check-supply-iocs.sh` advisory B1 的 marker 列表**不含**该措辞，且只 WARN | 读脚本 |

⇒ 1:1 vendored 133 文件 / 15,047 行 + 14 包 / 694 ts 源码的**记录低报了实际条款**；
按条款字面，持续修改即落入「secondary development」，若商用则未授权。
**这是商业/法务判断，agent 不代签。**

**已做**：① 门 `scripts/check-license.sh`（G6）；② 更正 `VENDOR.md` 许可节，
把条款原文、三点必须知道的事实、三条处置路径写进去；
③ 建 `.neotrix/LICENSE-EXCEPTIONS.md` 作为**唯一放行通道**（人工签署，
字段齐全才算；`accepted-with-condition` 须带 `condition` 与 `review_by`）。

**为什么不用「直接删掉 deny 名单」**：删 deny = 门变绿 = 问题被隐藏，
正是本仓明令禁止的失败模式。签署把决定权**显式落到人**。
当前状态：**无任何签署 ⇒ 门 FAIL，这是正确状态。**
门在 `ci.yml` 里刻意 `continue-on-error`（可见但不阻塞），
待签署后去掉该标记。

### 10.3 T3/T4/T6 落地与实测

| ID | 结果 |
|----|------|
| **T3** ✅ | THIR 251 MB 上 `MethodCall` 出现 **0 次** ⇒ `kind=='method'` 是**结构性死分支**，已删。顺带更正 docstring 陈旧值：抽取率实测 **6241/6241 = 100%**（旧写 94%）。回归验证：边数与 CLI 输出一致 |
| **T4** ✅ | 默认模式现在每次都打印「本判据只能证明文本存在 / PASS 不是健康证明」 |
| **T6** ✅ | **`--stale` 第一版是从不检查却恒报「0 过期」** —— DefId 段是模块路径永不含 `/`，循环体一次不执行。改用边表 `span` 里的真实路径后：实测 **2161 个文件检查、4 个真过期**，且这 4 个与 `git status` 的改动文件**完全重合**（两套独立工具互证） |
| — | 另修 bash 3.2 陷阱：`$VAR` 紧跟多字节字符（`$EXCEPTIONS（`）会被并入变量名 ⇒ 改 `${EXCEPTIONS}`；该 bug **只在签署路径触发**，是测出来的不是看出来的 |

### 10.4 本轮自身踩到的两个「假绿」教训

1. **例外记录的格式模板长得像真签署** —— 模板里填了真实路径 + `owner:` + `date:`，
   于是 `ack_for` 判为已签署，门输出「已有**人工签署**」而实际无人签。
   **一个宣称「有人签过」的门，比没有门危险。** 修：模板改占位符 + 解析器跳过 ``` 围栏。
2. **`json.dump(indent=2)` 把 `task-index.json` 整体重排**（813 增 / 727 删），
   违反 R-P16「禁整文件覆写」。原文件是 `indent=1`。已 `git checkout` 还原后重做，
   现为 **73 增 / 0 删** 的纯增量。

### 10.5 T1 的真正解法：不是「裁决 308 条」，是**证明这个判据做不到**

我原计划「逐条读代码裁决 308 条」。实际做下来发现更好的目标不是裁决，而是
**先量出判据的误报率**。做法：给 `--unreachable` 加**可见性**判据
（`pub fn` vs `fn`，从模块路径定位源文件后匹配定义行），
把 no-callsite 切成 private / public。

第一版结果：**36 条 private「高置信死链」**。**我随手抽 3 条手验 —— 3 条全错**：

| 符号 | 真实接线 | 我第一版为何没看见 |
|---|---|---|
| `auth_middleware` | `middleware::from_fn_with_state(state.clone(), auth_middleware)` | 注册式 API 传 fn 值 |
| `health_handler` | `.route("/health", get(health_handler))` | 同上 |
| `models_handler` | `.route("/v1/models", get(models_handler))` | 同上 |

于是把 F3/F6 探测器修了**五版**，每一版都是被手验推翻的：

| 版 | 判据 | 仍漏什么 | 桶内剩余 | 手验误报率 |
|---|---|---|---|---|
| v1 | 迭代器适配器 | axum/clap 注册式（`get(h)`） | 36 | **100%** |
| v2 | fn 紧跟 `(` | `from_fn_with_state(state, h)`（末位实参） | 8 | 67% |
| v3 | `[^)]*` 跨实参 | `state.clone()` 的 `)` 截断 | 8 | — |
| v4 | 一层括号配平 | rustfmt 拆行（需 `rg -U`） | 7 | — |
| v5 | + 回调组合子 + 领域回调 | 未知 | **3** | **0%（3/3）** |

**最终 3 条，全部经独立复核**（只在自己文件出现、`fn` 私有、全仓零引用）：

| 符号 | 位置 |
|---|---|
| `hz_to_mel_slaney` | `l1_action/nt_media/nt_speech_transcribe.rs:76` |
| `get_geo_cache` | `l3_embodiment/nt_shield/nt_shield_stealth_net/geo_proxy.rs:32` |
| `_now_ms` | `l3_embodiment/nt_shield/shield_core/nt_shield_mcp_security/nt_mcp_registry.rs:203` |

⛔ **但结论不是「3 条可删」**，而是：

> **「零调用 ⇒ 死」在本仓不可能一次做对** —— 每修一族就暴露新的一族
> （注册式 → 末位实参 → 嵌套括号 → 拆行 → 回调 → 领域回调…）。
> **残余风险是「未知族仍存在」，不是「零风险」。**
> 这就是 `CAPABILITY-GAP` G7「不要给死代码建门」的**实证**，
> 也是本仓 `nt_coverage_gaps.py` 自述「是筛选器，不能证明哪里没问题」的量化版。

⇒ 故桶名从 `dead-private`（**假称高置信**）改为 **`suspect-dead`（疑似，未证）**，
并在脚本 docstring 里写死这条五版轨迹，防止下一个 agent 把它当成已解决的工具。

### 10.6 T1/T2/T5 最终状态

| # | 状态 | 说明 |
|---|------|------|
| T1 | **换目标完成** | 原目标「裁决 308 条」作废（判据本身不可靠）；新目标「量出误报率 + 收窄到 3 条已手验」已完成 |
| T2 | **未做** | 需 `--all-targets` ~30min；另一窗口仍在改 `crates/neotrix-neobot/`，AGENTS.md 禁止并行全量 |
| T5 | **不做** | 已证实 188 条 textual-prod 多为 F2 同名碰撞 + 散文命中（`throughput` 命中一句英文文档）；逐条归因收益低于成本 |


---

*End of Absorption Round 24*
