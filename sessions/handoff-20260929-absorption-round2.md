# Handoff — 第二轮外部吸收 + 幻影门拆除（2026-09-29）

> **给下一个窗口的一句话**：本轮**没改任何 `.rs`**，全部产出是 3 份文档 + 2 门改造 +
> **拆掉 5 个恒红幻影 CI 门**。已提交 `8abad2a9`。
> **动手前必读 §3 的并发实测** —— 本轮有**两个窗口同时在写**，其中一次
> 「我的改动被 pre-commit 拦下」其实是**别的窗口抢先提交**。

## 1. 交付物

| 文件 | 内容 |
|---|---|
| `docs/architecture/ABSORPTION-AGENT-ARCH2-2026-09-29.md` | 30 源吸收结论 + **5 个被证伪的提交前提** + 许可证台账 + **4 处被证伪的既有文档数字** |
| `docs/architecture/EVOLUTION-ROADMAP-CODE-NODES-2026-09-29.md` | 13 个支脉节点（N-1…N-13），每个带 `file:line` + 可执行验收 + 批次依赖图 |
| `docs/architecture/BATCH-FIX-2026-09-29.md` | 统一任务清单：✅已完成 / ⬜待做 / ⚠️并发冲突 / ⛔需裁决 |
| `scripts/check-ci-refs.sh`（新） | 拦「job 指向 git 未跟踪的路径」 |
| `scripts/check-truth-surface.sh` | class 2 扩域 + 新增 `UNREACHABLE` 类 |
| `scripts/truth-surface-baseline.txt` | 265 条存量（棘轮） |
| `scripts/check-skill-gate.sh` | 修正 3 处失效数字 + 记下复算命令 |
| `.github/workflows/ci.yml` | 删 4 个幻影 job + 接 `check-ci-refs.sh` |
| `.github/workflows/docs-deploy.yml` | **已 `git rm`** |

## 2. 本轮最重要的三个发现

### 2.1 「自进化没人证明有效」从 11 仓扩到 **30 仓**，且找到了反解法

上一轮（`EVOLUTION-ROADMAP-2026-09-28 §0.1`）说 11 个自进化仓无一测量自己的收益。
本轮把样本扩到 30 个新源，**结论不变但找到了出路**：

- `jaredpalmer/kev` `scripts/verify_claims.py` + `docs/claims.json` ——
  **印在文档上的每个数字，CI 都能追到原始结果文件**，且按**印刷精度**比对。
- `MakazhanAlpamys/Soup` `benchmarks/gate-*.md` —— 17 份门记录，
  **连失败的 gate 都当一等行发布**；13 次逐字节相同配置跑出 2.43× 吞吐差并
  分解为可加噪声；README 自承头条数字**已过期未重测**。

> **差异不是纪律，是「产出证据的机制」**：两者都把证据做成**可执行物**
> （CI 脚本 / 门记录 + harness JSON 记 `git_sha`），而不是事后整理的报告。
> 本仓的 `nt_manifest.py` 已有 `env_fingerprint()` 地基（N-11）。

**反面教材 `cline`（69.5k★）**：eval 框架是本轮见过最完整的（3 层、
`pass@k` **和** `pass^k`、`FLAKY` 一等裁决态、失败分类器带 issue 链接），
然后 `evals/ARCHITECTURE.md` 自陈 CI 被 `removed`、smoke `disabled`、
`benchmarks/tool-precision/DEPRECATED.md`。**测量基础设施被一次重构孤儿化，
而没有任何东西会告诉你。**

⇒ 本仓已有一个同源问题：`check-test-baseline.sh` 的 baseline **0 字节**
（N-3），而 CI 正在调它。

### 2.2 拆掉 5 个恒红幻影门

| job | 指向 | 实测 |
|---|---|---|
| `frontend-tests` / `frontend-coverage` / `frontend-build` | `neocodex-frontend/` | `.gitignore:136` **整目录忽略**，`git ls-files` **0 跟踪** |
| `e2e` | `e2e/` | **0 跟踪**，磁盘不存在 |
| `docs-deploy`（整个 workflow） | `docs/package.json` | **已于 `477bf669` 随 vitepress 站删除**（`git show 477bf669` 可见 `-87 docs/.vitepress/config.ts` `-15 docs/package.json`） |

**为什么重要**（`awesome-dsh-plugin/.github/workflows/pr-gate.yml:44-60` 原话）：
*"A gate that dies before posting is indistinguishable from one that never needed
to run."* —— 恒红的门**比没有门更坏**，它训练人忽略红色。

**本仓已为同一类付出过代价**：`3edf3be7`「发布链路: 移除幻影CSS门禁
(脚本不存在, 构建恒失败)」—— 那次修在 `package.json`，**这次复发在 CI 层**。

### 2.3 **212 个 `.rs` 从不被任何 target 编译**

`check-truth-surface.sh` 的动机注释（`:4-9`）描述的就是这个病
（`cargo test` 全绿而 311 个测试从未编译）。**但修复只覆盖了字面叫 `tests`
的目录**（`:69` 的 `-path '*/tests/*'`），同类问题在普通模块目录里从未被覆盖。

新方法（关键）：**从每个 crate root 出发的传递 mod 可达性**，含
`mod X;` / `mod X {}` / `#[path=]` / `include!` 四种形式。
⛔ **不用 dep-info 当 oracle** —— `target/debug/deps/neotrix.d` 只覆盖单个
target（列 2,225，磁盘 2,550），用它判会把 bin / integration-test 的文件
**大面积误报**。

抽样（全 `grep -c` = 0 确认）：
- `neotrix-core/src/l1_action/nt_act/tool_registry.rs`（**770 行**）
- `neotrix-core/src/l6_meta/nt_meta/eval_engine/`（**641 行**，全仓**唯一**的
  dataset / experiment / llm-judge 抽象）
- `neotrix-core/src/l3_embodiment/nt_shield/defense/**`（21 个）

**221 条全部是 git 已跟踪文件**（干净检出同样存在）⇒ 不是任何窗口的 WIP。

⛔ **判读纪律（已写进门输出）**：「未被编译」≠「功能缺失」。三类处置不同：
- (a) 已归档旧引擎 → 删/归档
- (b) 忘了加 mod → **先问「它现在能编译吗」**
- (c) **声明被注释掉** → `nt_memory/mod.rs:54` 的 `pub mod hybrid_retrieval;`
  注「内部编译错误待修复」—— **加回去会让干净检出编不过**

## 3. ⚠️ 并发实测（本轮全程遇到，务必读）

**本轮有至少两个窗口同时在写。** 三次实测：

| 观测 | 含义 |
|---|---|
| `crates/neotrix-neobot/src/nt_types.rs` mtime 距当时刻 **36 秒** | 另一窗口在写 neobot ⇒ **N-6/N-7 故意没做** |
| `nt_mind_skill_engine/book_to_skill.rs` mtime 距检查 **5 秒**，且内容与 HEAD **完全不同** | 整文件重写中。`cargo check` 的 2 个错误（`:89` 语法错 + `E0603 format_route is private`）**属他窗，不是本轮引入** —— 本轮**未改任何 `.rs`** |
| `git commit` 被 pre-commit 拦下，但 `git log` 显示**我的提交没进去、别人的进去了** | ⛔ **不要以为「提交失败」= 「hook 拒绝」**。先 `git log --oneline -3` 确认是否被抢先 |

**pre-commit 的 scan-surface 门抓到我自己的死引用**：我在新 Python 块里写了
`src-tauri/src`（已随 `5c02e738` 归档）。已修 —— 这个门**正常工作**，值得信。

## 4. 留给下一窗口的（按建议顺序）

### 批次 A（零依赖，先做）
1. **N-3** `check-test-baseline.sh` 账本 **0 字节** ⇒ `--strict` 下零容忍。
   从**干净检出**填充（`LESSONS-20260928-fresh-checkout`），再做非空门证明。
2. **N-5** 扩 `nt_manifest.py` 查**数字**（不只查 `file:line`）。本轮已抓出
   4 处文档数字失效，验收就是把它们写进 claims 后门必须红。
3. **N-4** `SkillInvocationPolicy` 的 3 个 `visible_*` **零消费者** ⇒
   第一轮记的「核心代码」不成立，是**门**。另 `is_official_converged()` 是死代码。

### ⛔ 需裁决（本轮只取证，**没擅自动**）
- **221 个未编译 `.rs` 的逐条处置** —— 多数是已归档旧引擎，但归档/补 mod/删是
  **产品判断**；(c) 类必须先试编译
- `hybrid_retrieval/` 6 个文件：修编译错误还是删
- `nt_meta/eval_engine/` 641 行：接上（N-11/N-12 都依赖它）还是当孤儿
- `docs/` 要不要恢复 vitepress 站（110 个跟踪文件还在）
- **N-6/N-7 何时做** —— 需先与正在写 `crates/neotrix-neobot/src/` 的窗口协调

## 5. 本轮的门学到的两件事（比代码更值钱）

1. **非空门证明不可省。** `check-ci-refs.sh` 第一版是坏的：
   ```bash
   [ -n "$val" ] && return 0   # ← 把「非空」当「合法」⇒ 抓到注入的幻影反而放过
   ```
   是「注入一个违规、确认它红」这步把它抓出来的。**没有这步，我会交付一个
   永远绿的假门** —— 那正是本轮拆掉的 5 个幻影门的同一种病。
2. **门要问可满足性。** `i-have-adhd` 的发布门**永远无法通过**
   （"no blocking findings" 是绝对规则，相邻规则却是比较规则），
   **是跑出来才发现的**。本仓同源：N-3 的空 baseline。
   ⇒ N-12 要建 `check-gate-satisfiable.sh`：每道 `--strict` 门都要有
   「注入违规⇒红」**和**「存在合规输入⇒绿」两条证明。

## 6. 收工自查（§8 模板）

- **worktree**：本轮**未开任何 worktree**。`nt_worktree_gate.sh check` 实测
  现有 2 个（`merge-test` 近 3h 有活动、`ratchet` 干净）**均非本轮所开，不动**。
- **未提交改动去向**：本轮产出**全部已提交** `8abad2a9`（9 文件，+1219/−137，
  **零 `.rs`**）。工作树剩余脏文件**全部属其他窗口**（66 项，含 neobot 与
  `models/training/*`），**本轮一律未碰**。
- **未入库的已知项**：`UNCOMMITTED_DEP crates/neotrix-neobot/src/nt_qwen_mm.rs`
  是另一窗口的 `??` 在制品，**故意不写进 baseline** —— 吸收进来等于把他人
  中间态固化成「已知存量」。他自己提交时会被门正常拦下。
