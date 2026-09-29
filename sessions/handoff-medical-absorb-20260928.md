# Handoff — 医患对话疾病集吸收进晶体核心（2026-09-28）

> 窗口任务：把 `nisten/opus5-5-doctor-patient-conversations-all-human-diseases`
> 下载到本地并吸收融入晶体核心（`~/.neotrix/crystal_core/cocoons.json`）。
> **已完成**，含一处先于本任务的前置地雷修复。

---

## 1. 数据源

| 项 | 值 |
|---|---|
| id | `nisten/opus5-5-doctor-patient-conversations-all-human-diseases` |
| 许可 | Apache-2.0 |
| 生成器 | **Claude Opus 5.5（合成，非临床 ground truth）** |
| 原始体积 | 75,312,131 B（与 HF `usedStorage` 逐字节吻合） |
| sha256 | `f828c30cee7006a3cf5b88909f9b865688f98b11d4c886108b9b9a39e5402e0b` |
| 记录数 | 2194（0 条坏 JSON） |
| 落点 | `datasets/hf_raw/opus5-5diseaseconversations.jsonl`（`datasets/` 在 .gitignore，数据不出项目） |

每条记录 = 一个疾病的 20 键结构化病历卡（aliases / icd10 / prevalence /
body_systems / related_drugs / drug_interactions / food_interactions /
pubmed_refs / common_mistakes / differential_diagnosis / related_diseases /
executive_summary …）+ 一段 ChatML 问诊对话（均 33.4 轮）。

## 2. 产出（已落盘）

```
records 2194 → memories 56297 (25.7/record) → 59 新茧
cocoons.json 4.7MB → 51.6MB
跨病种知识图边 7529 条（另有 227 条指向别病锚点，合计 7302 跨锚边）
```

`connected_ratio` 0.209 → **0.873**，越过 `Transcend` 相位四必要条件中的
`ratio > 0.6` 那条（`consciousness.rs:462-470`）。

### 记忆面（domain = `CocoonStore::recall` 的路由键）

| domain | 条数 | 面 | MemoryType |
|---|---|---|---|
| `medical-clinical` | 32710 | pitfall / differential / related | Lesson / Pattern / Causal |
| `medical-pharma` | 14811 | drug / drug-interaction / food-interaction | Causal |
| `medical-disease` | 4388 | profile（锚点）/ summary | Fact / Pattern |
| `medical-literature` | 2194 | refs（pmid 可检索） | Fact |
| `medical-consult` | 2194 | 问诊对话首尾弧 | Experience |

「融入」的关键 = **跨病种连边**：profile 是锚点，本记录各面连回锚点；
`related_diseases[].disease` 与 `differential_diagnosis[].condition` 经
**精确 + 模糊（token Jaccard ≥ 0.55）**解析到别的记录锚点（44.8% + 2.0%，
逐条抽样人工核对模糊命中全部正确），在 2194 节点上连成真实医学关系图。

### 诚实性约束（刻意的设计决定）

数据集是**合成**的。故：
- `confidence` **封顶 0.75**（合成叙述 = 可信线索，不是已验证临床事实）；
  灌成 0.9+ 等于伪造权威。
- profile / consult 面额外打 `[synthetic]` 标记，随内容一起传播。
- provenance / sha256 / license 全量落在 `datasets/hf_medical/absorb_manifest.json`。

## 3. 顺带修掉的**前置地雷**（先于本任务存在，非本次引入）

活库 4 条记忆 `memory_type: "CrossDomain"`。`CrossDomain` 是
**`ReasoningType`** 变体（`consciousness.rs:81`），而 `MemoryType`
（`consciousness.rs:39-56`）只有 8 个变体，不含它 —— 两个枚举不同命名空间。

**杀伤链**：
1. `CocoonStore::load()`（`cocoons.rs:85-91`）对 `serde_json::from_str`
   **全有全无** —— 任一条记忆解析失败即 `Err(_)` → 返回
   `StoreData{cocoons: HashMap::new(), ..}`，**不报错不告警**。
2. `nt_train_export --ingest` 是 `load() → sync_from_consciousness → save()`
   （`nt_train_export.rs:92-95`）→ 空 store 落盘 = 把 cocoons.json
   **覆盖成只有这一批**，7450 条存量全灭。

**处置**：
- 新脚本 `scripts/ops/nt_cocoons_repair_memory_type.py`：字节级定点替换
  （不整体重序列化，避免污染 4.7MB 存量文件的无关格式），白名单驱动，
  表外非法值**拒绝猜测**直接拒修。已修 4 处 `CrossDomain → Pattern`
  （融合结论 = `consciousness.rs:42`「模式 (从推理产生)」）。
  字节差 −16 = 4×(len("Pattern") − len("CrossDomain"))，证明只动了目标。
- **根因**：`nt_hf_smelt_chains.py` 的 `FUSIONS` 4 条误用推理类型名
  （同表 `fuse-gate` 用了合法的 `Causal`，说明是漏改非有意）。已改
  `Pattern`，并在 `emit()` 加白名单闸 —— 未来再写错类型在**写盘前**就 abort。
  融合层置信 0.65 的原意按「层」保留（不再靠类型名分支）。

## 4. 本次自曝并修掉的自身缺陷

`pubmed_refs[].year` 是 **int**，而蒸馏器复用的 `sget()` 只收 `str` →
**6377 条 PMID 年份全静默丢失**（渲染成 `(?)`）。首轮提交后自查发现，
已加 `snum()`（收 int/float/str，拒 bool）+ selftest 钉住，
并**回滚到 `.bak.medical` 重跑**（非就地打补丁），复验 0 条 `(?)`。

## 5. 工具（全部 selftest 自证 + 零构建可跑）

| 脚本 | 作用 |
|---|---|
| `scripts/ops/nt_medical_distill_to_cocoons.py` | 蒸馏 → 直灌茧。`--dry-run/--commit/--cap/--limit`，流式两遍，**复用** `nt_hf_digest_to_cocoons` 的 `cocoon_entry`/`splice`/备份轮转（格式单一事实源，禁复制粘贴） |
| `scripts/ops/nt_medical_verify_cocoons.py` | 独立验收器（不复用生产代码任何函数，只按 `Memory`/`PersistentCocoon`/`StoreData` 契约重新判定）。10 项，含**全量**（非抽样）锚点语义校验 |
| `scripts/ops/nt_cocoons_repair_memory_type.py` | 毒记忆修复器 |

验收结果：10 项全过，**唯一 FAIL 是既存的 2680 个重复 M-id**（用户裁定本轮
不碰，见 §6）。

## 6. 遗留（明确交给下一刀，非本轮）

1. ~~**2680 个重复 M-id**~~ → **已修复**（2026-09-28 第 3 次会话，见
   `handoff-cocoons-health-20260928.md`）。查明并非冗余，而是 id 碰撞在吃掉
   **不同**记忆（1,645 组内容全不相同）；已重编号 + 堵复发路径。
   当时本轮新增的 56297 条即零重复（现已折叠 2 条源数据重复 → 56295）。
2. **Rust 侧实证缺口**：`CocoonStore::load()` 能否真解析 51.6MB 的盘，
   目前只有**静态分析 + 独立验收器**（JSON/契约/枚举/连边全绿），
   **未跑二进制证实** —— `nt_mem_gate.sh` 全程 BLOCKED
   （free_pages 20248~53832 < 100000 阈值），按 AGENTS.md 禁止起 cargo。
   门开后跑：`cargo run -p neotrix --bin nt-train-export -- --help`
   或任一走 `CocoonStore::load()` 的 bin，看 `stats().total_memories`
   是否为 63747（若为 0 即复现地雷）。
3. 那 4 条修好的记忆 `connections` 指向 `M-4794xx`（旧编号空间，盘中不存在），
   属既存悬空边，不阻断解析（`recount_connections` 只数非空，不解引用）。

## 7. 并发与门记录（本窗口实测，非沿用旧值）

- `nt_lock_audit.py neotrix-core/src` → **0 处可疑**（2026-09-28 本轮实测）
- `nt_mem_gate.sh` → **BLOCKED**（2026-09-28 本轮多次实测，free_pages
  20248~53832，swap 1318M）→ 故本轮**未起任何 cargo**
- `nt_sidecar.sh status` → DOWN（按需，非本轮内存占用源）
- 未触碰工作区他窗改动（`git status` 显示大量他人修改文件，本轮只新增
  3 个 ops 脚本 + 改 1 个 ops 脚本）

## 8. 复现命令

```bash
# 1) 体检（先自证验收器本身）
python3 scripts/ops/nt_medical_distill_to_cocoons.py --selftest
python3 scripts/ops/nt_medical_verify_cocoons.py --selftest
python3 scripts/ops/nt_cocoons_repair_memory_type.py --selftest

# 2) 干跑投影（不动盘）
python3 scripts/ops/nt_medical_distill_to_cocoons.py --dry-run

# 3) 验收活库
python3 scripts/ops/nt_medical_verify_cocoons.py \
    --before ~/.neotrix/crystal_core/cocoons.json.bak.medical

# 4) 限量重灌（若要改口径）
python3 scripts/ops/nt_medical_distill_to_cocoons.py --commit --limit 200
```
