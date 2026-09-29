# Handoff — MiMo-V2.6-RL-oss 训练环境吸收进晶体核心（2026-09-28）

> 窗口任务：把 `XiaomiMiMo/MiMo-V2.6-RL-oss` 落到本地并吸收融入晶体核心。
> **已完成**。承接同日 `handoff-medical-absorb-20260928.md`（医患集），
> 两者共用 `nt_hf_digest_to_cocoons` 的落盘原语与同一套验收纪律。

---

## 1. 数据源实况（与预期不同的关键点）

| 项 | 值 |
|---|---|
| id | `XiaomiMiMo/MiMo-V2.6-RL-oss`（小米 MiMo，8294 下载 / 424 likes） |
| 许可 | Apache-2.0 |
| 性质 | **Agentic RL 训练环境**（Docker 镜像 + 训练代码 + 925 可验证环境），**非文本语料** |
| 构成 | 5 个 parquet（21.3MB / 7,780 任务行）+ `general/envs/` 925 个环境目录 + `image-mapping.jsonl` |
| 落点 | `datasets/hf_raw/mimo/`（`datasets/` 已 gitignore） |

5 域：code 2698 / webdev 2093 / cyber 1000 / music 1000 / general 989。
**全部 `reward.style=rule`、全部单轮 user prompt、agent=mimo_swe_agent。**

## 2. 核心判断：按「验证器工程」吸，不按「任务实例」吸

实测证明任务实例高度同质，倾倒即注水：

- 2,698 条 code 任务的验证器形态**完全一致**
  （`bash mimo_test_command.sh` + `test_patch` + `verifier_timeout_sec` 恒 1800），
  2698 个任务只有 3 种 test_command 字符串变体。
- 2,093 条 webdev 的 `category` 只有 1 种（website）。
- 7,780 行 prompt 正文约 9.4MB，其中 1,462 条 code 任务连开头形态都不成簇。

真正独有、可迁移的知识在两处：

1. **925 份分层加权 rubric**（`verifier_meta.json`，共 **5,125** 检查项）——
   `tier`(critical/important/sanity) × `method`(llm/rule) × `pass_anchor` 内嵌 ground truth。
2. **共享判分骨架** `verify.py` —— **925 份逐字节相同**（sha256 `caa22a4c64bea979…`，
   23,526 B）。抽样 3 环境实测唯一哈希=1；正式拉取后 `verify_shared.py` 哈希与抽样值
   **完全一致**，全量级证实。

## 3. 产出（已落盘）

```
973 条记忆 / 4 新茧 / 1,899 条边 / 0.95MB
cocoons.json 51.6MB -> 52.5MB
```

| domain | 条数 | 内容 |
|---|---|---|
| `rl-domain` | 6 | 5 域锚点（任务族/验证机制/奖励形态）+ 总览 |
| `rl-verifier` | 11 | 判分器工程模式（判分锚点 + 10 条） |
| `rl-taxonomy` | 31 | 16 环境族锚点 + 音乐约束文法 + 漏洞分类轴 + 预算包络 |
| `rl-rubric` | 925 | 逐环境 rubric 摘要（tier/method/产物/判分要点） |

连边：族锚点→general 域锚点；taxonomy→各域锚点；逐环境 rubric→族锚点+判分锚点。
`connected_ratio` 0.873 → **0.875**。

### 沉淀进核心的可迁移知识（判分器工程）

- **分层 rubric**：5,125 项 critical 2,973 / important 1,855 / sanity 297
  —— critical 占 58%、「关键项必须全对」是主流设计而非平均分
- **双方法混合**：llm 4,437 项（question+pass_anchor 交模型判）/ rule 688 项（fn 交代码判）；
  667 项无 question 正是 rule 项 —— 可验证性优先于全自动判分
- **pass_anchor 模式**：4,449/5,125 项把 ground truth 写进判分问句，使 judge 有据可依
- **judge 三级 fallback 链**：responses / chat / gemini 并列
  （OpenAI 兼容层对 Gemini 组报 500 `contents is required`，必须走原生 wire）
- **key 不落盘**：`GA_JUDGE_KEY/_KEY2/_KEY3` 环境变量透传，env 内不持久化
- **证据预算 + 投票**：`_evidence_sc(budget=20000)`、`_run_llm_sc(votes=N)`
- **产出契约**：4,448/5,125 项读 `answer.md`；reward 函数名统一 `rl_grade`
- **16 个企业职能族**：会计审计 157 / 金融保险 135 / 医疗运营 107 / 咨询 89 / HR 77 /
  政务 72 / IT 72 / 教育 55 / 能源 47 / 法务 47 / 建筑 14 / 制造 14 / 电商 14 /
  酒店 9 / 物流 9 / 农业 7（en/zh 双语混合）
- **约束文法**：音乐 79 曲种 / 141 BPM(55..220) / 5 拍号 / 6 声部数；
  cyber 3 sanitizer × 19 错误型 × 176 个 C/C++ 项目（ndpi 70 / ffmpeg 55 / imagemagick 52…）

## 4. 置信度口径（与医患集刻意不同）

本集所有数字都是**我直接实测**的（真实发布 + 自测），故分档而非一刀切封顶：

| 类别 | conf | 理由 |
|---|---|---|
| 实测统计（计数/分布/哈希） | 0.90 | 我自己数的，可复现 |
| 结构设计事实（验证机制） | 0.85 | 读代码/读 schema 得出 |
| 推断性教训（`[verifier-lesson]`） | 0.70 | 推断，不冒充实测；且 MemoryType 降为 `Lesson` |

医患集是 Opus 合成 → 一律封顶 0.75 + `[synthetic]` 标记；
本集是真实发布 + 自测 → 可高。**凡推断一律不冒充实测。**

## 5. 读输出读出来的三个 bug（都已修 + 已钉 selftest）

预演 dump 全文人工审阅时抓到的，都是「静默错数据」类：

1. **假的安全分类轴（最严重）**：`[taxonomy:cyber]` 原用**全局** `err.most_common(1)`
   填进每个 sanitizer，产出「MemorySanitizer→heap-buffer-overflow 235」这种
   **假映射**（实测 MSan 96% 是 `use-of-uninitialized-value`）。
   三条轴被抹平成同一个值 —— 而「各 sanitizer 有特征主错误类」正是本数据集的
   分类轴本体。改为逐 sanitizer 取自身 top3。
2. **族锚点的 lang 是全局的**：16 个族锚点全写同一句 `lang=en:504/zh:421`
   （那是全库口径，不是该族）。改为逐族统计（如 accounting_audit_tax en:94/zh:63）。
3. **权重口径含混**：`权重和` 原按**逐项**统计（跨度 0..32）却在文案里说成
   「权重和」，易误读。改为明确的**逐环境**权重和（跨度 0.00..120.00，
   恰为 1.0 的仅 45/925 个）。

另有一个**锚点归域 bug**：23 个锚点全被塞进 `rl-domain`（应为 rl-verifier 1 /
rl-taxonomy 16 / rl-domain 6），预演输出里 `rl-domain 23` 一眼看出不对。

## 6. 工具（全部 selftest 自证 + 零构建可跑）

| 脚本 | 作用 |
|---|---|
| `nt_rlenv_fetch_rubrics.py` | 拉 925 份 rubric + 1 份共享判分骨架。8 线程温和并发 + 3 次重试 + 合法性闸（拒错误页/错类型）。实测 925/925 成功、0 失败、268s |
| `nt_rlenv_distill_to_cocoons.py` | 四层蒸馏 → 直灌茧。**刻意不用前缀和**：1k 量级记忆全驻内存、编号一次成型，从根上消除 count/actual 漂移那一类 bug（医患集踩过） |
| `nt_cocoons_verify_absorb.py` | **通用**验收器（按 domain 前缀，不绑定数据集），替代「每集写一个验收器」的路数 |

`nt_cocoons_verify_absorb.py` 交叉复验医患集亦全绿（56,297 条 / 0 违例 / 0 悬挂 / 置信封顶 0.75 未越界）。

验收器自身也踩过一个坑并修掉：`mems` 用 `setdefault`（首现）而 `bmem` 用 dict
推导（末现），导致既存 1,645 个重号 id 被判成「存量被改写」。两边统一 first-wins。

## 7. 当前晶体核心总账

```
cocoons=207  memories=64,720  unique=62,040
illegal memory_type=0
connected=56,607 (87.5%)
```

| 来源 | 条数 |
|---|---|
| 医患集（本日第 1 次吸收） | 56,297 |
| MiMo RL 环境（本日第 2 次吸收） | 973 |
| 既存 | 7,450（含 1,645 个重号 id，见下） |

## 8. 遗留（承接上一轮，未变）

1. ~~**1,645 个重号 M-id / 2,680 条被遮蔽**~~ → **已修复**（2026-09-28 第 3 次会话）。
   详见 `handoff-cocoons-health-20260928.md`。今日两次吸收当时即零重号。
2. **Rust 侧实证仍缺**：`nt_mem_gate.sh` 全程 BLOCKED
   （free_pages 20248~57027 < 100000，swap 1.0~1.3G），按 AGENTS.md 未起任何 cargo。
   门开后跑任一走 `CocoonStore::load()` 的 bin，确认 `stats().total_memories == 64720`
   （若为 0 即复现 CrossDomain 静默归零地雷）。

## 9. 门记录（本窗口实测，非沿用旧值）

- `nt_lock_audit.py neotrix-core/src` → 0 处可疑（2026-09-28）
- `nt_mem_gate.sh` → BLOCKED（2026-09-28 多次实测）
- `nt_sidecar.sh status` → DOWN
- 未触碰他窗改动；本轮新增 3 个 ops 脚本

## 10. 复现命令

```bash
python3 scripts/ops/nt_rlenv_fetch_rubrics.py --selftest
python3 scripts/ops/nt_rlenv_distill_to_cocoons.py --selftest
python3 scripts/ops/nt_cocoons_verify_absorb.py --selftest

python3 scripts/ops/nt_rlenv_fetch_rubrics.py          # 925 份 rubric
python3 scripts/ops/nt_rlenv_distill_to_cocoons.py --dry-run
python3 scripts/ops/nt_rlenv_distill_to_cocoons.py --commit

python3 scripts/ops/nt_cocoons_verify_absorb.py --prefix rl- \
    --before ~/.neotrix/crystal_core/cocoons.json.bak.rlenv \
    --queries "rl-rubric:rubric,rl-verifier:pass_anchor,rl-taxonomy:sanitizer"
```
