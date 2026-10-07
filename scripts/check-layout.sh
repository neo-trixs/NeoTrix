#!/usr/bin/env bash
# check-layout.sh — 根目录/目录结构白名单门
#
# ## 为什么需要这道门（2026-09-29 实测）
#
# `DOCUMENTATION-MAP.md` 写了 4 条「禁止」：
#   - 根目录禁止放临时文件、会话笔记、分析报告（:39）
#   - neotrix-core/docs/ 禁止存研究笔记、分析报告（:81）
#   - .neotrix/ 下的 JSON/DB 禁止手改（:113）
#   - 文档操作禁止私自构建文件（:4）
#
# **但这四条全是散文，没有一道门执行。** 实测：在根目录放
# `SCRATCH-临时笔记.md`，`check-naming` / `check-doc-drift` /
# `check-truth-surface` **三道门全部通过**。
#
# ⇒ 规范形同虚设，下一个 agent 明天照样能在根目录扔文件。
# **一次清扫是临时的，门才是长期的。**
#
# ## 门做什么
#
# ① 根目录文件**白名单**：不在表内即红（表内每项附理由与消费者）
# ② 根目录**隐藏目录**白名单：同上
# ③ `neotrix-core/docs/` 只许日期前缀（`YYYY-MM-DD_*`）+ 固定子目录
# ④ **与 DOCUMENTATION-MAP.md 交叉校验**：两边清单不一致就红
#    —— 防止「规范腐化」本身（规范失真比没有规范更贵）
#
# ⑤ **gitignore 过滤**：`git check-ignore` 判为忽略的项不计入违规
#    （完整口径见下方 is_git_ignored）—— 但仍单列报告，不隐藏视野。
#
# ## ⛔ 为什么不一上来就阻断
#
# 本仓已有既存违反（`neotrix-core/docs/plans/` 5 个文件正是
# DOCUMENTATION-MAP:81 明令禁止的形态）。若直接 exit 1 ⇒ **恒红**，
# 而恒红的门比没有门更坏（它训练人忽略红色）。
# 故：advisory 起步 + 账本棘轮，拦新增、既有记账。
#
# 用法：bash scripts/check-layout.sh [--strict] [--update-baseline]
#   默认 advisory（恒 exit 0）
#   --strict           仅当有**新增**违规时 exit 1
#   --update-baseline  把当前存量写入账本（棘轮）

set -uo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT" || exit 2

BASELINE="scripts/layout-baseline.txt"
DOCMAP="DOCUMENTATION-MAP.md"
STRICT=0
UPDATE=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    --update-baseline) UPDATE=1 ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

# ── 根目录允许清单（每项必须有真实理由，不是「先放着」）────────────
# 格式：文件名|类别|理由（理由要指向真实消费者，否则就是待清理物）
ALLOW_FILES="
README.md|入口|项目入口，GitHub 展示面，DOCUMENTATION-MAP:14
AGENTS.md|agent 守则|agent 守则中枢，.githooks/{pre-commit,prepare-commit-msg} + 94 处引用
RUST-STANDARDS.md|规范|Rust 编码标准正典，cargo xl 风格命令出处
CONTRIBUTING.md|流程|贡献指南
REVIEW.md|流程|审查策略正典（passes/严重度分桶/不报清单），.opencode/agent/review.md 消费 + check-agent-config.sh 守引用完整性
CHANGELOG.md|流程|Keep a Changelog
TODO.md|任务|唯一任务清单
TODO.yml|任务|neotrix todo sync 生成，git_hook.rs 消费，禁手改
DOCUMENTATION-MAP.md|规范|文档标准地图 —— 本门与之交叉校验
ARCHITECTURE-MAP-ROADMAP-V2.md|台账|模块台账（R-P199 口径）
LICENSE|法务|MIT，Cargo.toml license-file 引用
Makefile|构建|Makefile 被 ci.yml + Makefile 的 build 目标消费
Cargo.toml|构建|workspace 根 manifest
Cargo.lock|构建|7 个 CI workflow 消费，deny.yml 强制入库
clippy.toml|构建|cargo clippy 配置（工具按约定名自动读取）
deny.toml|构建|cargo-deny 门配置，deny.yml 消费
results.tsv|账本|进化实验账本：nt_evolution_exp.rs 写入 + check-evolution-ledger.sh 消费
.gitignore|工具|忽略规则
"

# ── 根目录允许的隐藏目录 ────────────────────────────────────────
ALLOW_DIRS="
.github|工具|CI workflows（12 个）
.githooks|工具|hooks 路径（core.hooksPath）
.cargo|工具|cargo config + audit.toml
.neotrix|状态|layer-map / task-index / capability registry（唯一真源）
.project-map|状态|codemap 缓存
.blueprint|状态|manifest
# ⛔ 2026-09-29 已删 .claude/ —— 1 个 7 月会话残留（71 行），零消费。
#    曾考虑删 .neotrix-absorb/，实测其 20 条里 2 条尚未入 KB ⇒ 保留。
.opencode|状态|opencode agent 定义
.neotrix-absorb|状态|吸收轮快照
.worktrees|运行期|worktree 目录（已 gitignore，允许存在）
config|构建|真配置：cliff.toml / .gitleaks.toml / .env.example（根配置已收敛到本目录）
apps|源码|桌面 App（apps/neobot-desktop：tauri.conf.json + frontend/ + icons/）
crates|源码|workspace 9 个 crate（neotrix-types / -sysctl / -consciousness / -reasoning / -gateway / -multi-agent / -neobot / -audit / nt-core-capability-tree）
docs|文档|文档主目录（architecture/ 标准 / plans/ 方案 / adr/ 决策 / standards/ 规范 / api/）
models|数据|模型权重 + 训练脚本（gitignored，删了只能重下）
neotrix-core|源码|核心 crate，L0–L6 七层 + neotrix/ 第二棵树
scripts|工具|门脚本（21 个 check-*.sh）+ ops/（nt_*.sh / nt_*.py）+ 账本
sessions|过程|交接件 handoff-*.md（.gitignore 白名单放行）
skills|agent|Agent 技能树（skill_loader.rs 运行时读 CWD 的 skills）
target|构建|cargo 生成物（gitignored，已被 check-disk.sh 盯体积）
.cache|运行期|nt_feature_matrix.json 等工具缓存（tracked=0，允许存在）
evals|数据|gaia_mini 评测任务与基线（tracked=0）
"

# ── 已知残留（committed 已移除、磁盘仍有 untracked 残留）────────────
# ⛔ 这些**不是合法目录**，是归档后的磁盘残留。列在此处只为「记账不阻断」——
# 门仍会报它们（见下方 RESIDUAL 输出），但标为 known ⇒ 不计入新增违规。
# 处置：确认无消费者后 `rm -rf`（R-DISK-1：它们是生成物/已删源码，删前确认 tracked=0）。
# 2026-09-29：apps/ 与 src-tauri/ 已移出到
#   /Users/neo/Downloads/Neo/neotrix-archive/desktop-residual-20260929/
# （mv 非 rm，58/58 SHA-256 校验一致；含 2,496 行 git 已删的冒烟测试，
#   详见该目录 README.md 的恢复说明）⇒ 本表当前为空。
  # ⚠️ 2026-09-30 决策反转：apps/ 已回到 ALLOW_DIRS（见上方），桌面 App 回流本仓。
  #   `d5413335` 当初的「移出」裁决被推翻 —— 理由见
  #   `docs/architecture/APPS-DESKTOP-DECISION-2026-09-30.md`。
  #   src-tauri/ 仍在归档区未回流 ⇒ 本表对其保持空。
RESIDUAL_KNOWN=""

# ── 门只管「版本控制管的东西」（2026-10-07 定）─────────────────────
# 实测误报：根目录 `notes/` 被 `.gitignore:20`（首版提交即存在的规则）
# 明确忽略、`git ls-files notes` = 0 tracked，门却按**文件系统**扫描
# 把它算成「新增违规」，`--strict` 恒红。而仓库自己的台账已判定
# 「工作区产物 ⛔ 非代码缺陷」（DEBT-LEDGER-2026-10-07.md）。
# ⇒ 恒红的门比没有门更坏（同文件 :30-31）。错的是门，不是 notes/。
#
# ⛔ 口径（为什么不反过来「把 notes/ 记账成债」）：本门约束的是
# **入库物**（DOCUMENTATION-MAP:39「根目录禁止放会话笔记/分析报告」
# 的真实意图 = 别让评审/PR 里出现这种东西）。`.gitignore` 已经
# 声明「此处 git 不管」，把 git 拒绝管辖的目录判成布局违规，
# 语义是错的：账本棘轮会把「工作区垃圾」永久记成「布局债」。
# 同仓 `check-untracked-assets.sh` 用的就是
# `git ls-files --others --exclude-standard`（= 排除 gitignore），
# 本门与之口径一致，不是新发明。
#
# ⚠️ 本门因此**看不见**这一手：「新建根目录 + 同一提交把它写进
# .gitignore」⇒ 两者都 git 不管 ⇒ 不报。诚实的说法是：那笔
# .gitignore 改动是 **tracked 文件**，必进 PR diff 由人眼/评审拦，
# 本门把该判据交给 git diff，而不是凭空再造一道新门（新依赖）。
# ⚠️ 反向也一样：tracked 的东西绝不会被本过滤放过 ——
# `git check-ignore` 默认**读索引**，已入库路径恒返非 0。实测 2026-10-07：
# 把 README.md 显式写进 .git/info/exclude 后 `check-ignore README.md`
# 仍 rc=1 ⇒ 「先入库再 gitignore」骗不过本门（它靠索引而非只看规则）。
is_git_ignored() {
  # 非 0 = 「不管」。仓库外调用 git 返 128，同样落到「不管」之外
  # ⇒ 退化成改动前的全量扫描（fail-safe：新东西照样被拦）。
  git check-ignore -q -- "$1"
}

# ── 收集现状（只取 git 跟踪的 + 未跟踪但存在的一级项）──────────────
CUR=$(mktemp); trap 'rm -f "$CUR"' EXIT
IGN=""   # 被 gitignore 排除的项（单列报告，不隐藏）

# 根目录文件：跟踪的 ∪ 未跟踪但存在的
#
# ⛔⛔⛔ **盲区：dotfile / dot-directory 完全扫不到**（2026-10-07 记录，未修）
#
# `for f in *` 在 bash 里**不匹配以点开头的项** ⇒ 本门只审「非 dot 的根项」。
# 实测：仓库根有 11 个非忽略 dot 项，**一个都进不了本门视野**：
#   .blueprint .cargo .git .githooks .github .gitignore .neotrix
#   .neotrix-absorb .npmrc .opencode
# （另有 .cache / .project-map / .worktrees 已被 gitignore，同样扫不到。）
#
# ⓘ 也就是说：**在根目录新建一个 `.随便什么` 目录/文件，本门永远不报。**
#   —— 本门不是「本仓根目录干净」的证明。
#
# ⛔ 为什么**本轮不修**：让它扫 dot 项需要先答一个政策问题 ——
#   「根目录 dot 项里哪些是合规的」。上面 11 个全是合法工具/配置目录，
#   若逐个塞进 ALLOW_FILES，白名单就从「违规清单」退化成「现状快照」，
#   下一个人加新工具时会照着塞 ⇒ 门彻底失去约束力
#   （与 `--nb-side-w` 声明了却零消费、`--nb-col-w` 同类问题）。
#   ⛔ 这是**需要人裁决**的事，不是 agent 该替他定的。
#   ⇒ 处置：先记录，等裁决。裁决方向二选一：
#     ① 显式白名单「根目录允许存在的 dot 工具项」（须写理由，禁无脑追加）
#     ② 判据改为「非 dot 根项 + 明确禁止的新增 dot 项」，即只拦增量不枚举存量
for f in *; do
  [ -f "$f" ] || continue
  if is_git_ignored "$f"; then IGN="$IGN$f"$'\n'; continue; fi
  echo "$f" >> "$CUR"
done
# 根目录目录：排除 .git
for d in */ .*/; do
  d="${d%/}"
  [ "$d" = ".git" ] || [ "$d" = "." ] || [ "$d" = ".." ] && continue
  [ -d "$d" ] || continue
  if is_git_ignored "$d"; then IGN="$IGN$d/"$'\n'; continue; fi
  echo "$d/" >> "$CUR"
done
sort -u "$CUR" -o "$CUR"

# ── 判定 ────────────────────────────────────────────────────────
is_allowed() {
  # ⚠️ 统一去尾斜杠：收集端对目录 echo "$d/"，而白名单表里不写斜杠
  local name="${1%/}" list="$2"
  echo "$list" | grep -q "^${name}|"
}

VIOL=""
KNOWN=""
while IFS= read -r entry; do
  [ -n "$entry" ] || continue
  if is_allowed "$entry" "$ALLOW_FILES" || is_allowed "$entry" "$ALLOW_DIRS"; then
    continue
  fi
  if echo "$RESIDUAL_KNOWN" | grep -q "^${entry%/}$"; then
    KNOWN="$KNOWN$entry"$'\n'
    continue
  fi
  VIOL="$VIOL$entry"$'\n'
done < "$CUR"

# ── 第二类：neotrix-core/docs/ 只许日期前缀 + 固定子目录 ──────────
DOCSVIOL=""
if [ -d neotrix-core/docs ]; then
  while IFS= read -r f; do
    base="${f##*/}"
    case "$base" in
      [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]_*) ;;   # YYYY-MM-DD_ 前缀
      *)
        # 同口径：gitignore 的文件不在本门辖区（探针只对**候选**跑，
        # 合规日期前缀零开销 ⇒ 不给门加常态性能债）。
        if is_git_ignored "neotrix-core/docs/$f"; then
          IGN="${IGN}neotrix-core/docs/$f"$'\n'; continue
        fi
        DOCSVIOL="$DOCSVIOL  neotrix-core/docs/$f"$'\n'
        ;;
    esac
  done <<EOF
$(find neotrix-core/docs -type f 2>/dev/null | sed 's|^neotrix-core/docs/||')
EOF
fi

# ── 第三类：与 DOCUMENTATION-MAP.md 交叉校验 ─────────────────────
# 规范里点名了本门不认识的根文件 ⇒ 规范与现实发散了（两遍都算）
DOCGAP=""
if [ -f "$DOCMAP" ]; then
  # 抽出规范中根目录表里第一列的反引号文件名
  docfiles=$(sed -n '/^### 1\./,/^### 2\./p' "$DOCMAP" 2>/dev/null \
    | grep -oE '^\| `[^`]+`' | tr -d '|` ' | sort -u)
  for f in $docfiles; do
    [ -e "$f" ] || DOCGAP="$DOCGAP  规范点名但磁盘不存在: $f"$'\n'
  done
fi

# ── 账本棘轮 ────────────────────────────────────────────────────
if [ "$UPDATE" -eq 1 ]; then
  { echo "# 根目录/目录结构存量基线（棘轮）— 每行一项。"
    echo "# 生成于 check-layout.sh --update-baseline。"
    echo "# ⛔ 绝不要为了让门变绿而删条目 —— 那等于让违规永久合法化。"
    echo "# 处置正确姿势：把文件挪进合规目录，或把白名单扩到 ALLOW_FILES 并写明理由。"
    printf '%s' "$VIOL"
    printf '%s' "$DOCSVIOL" | sed 's/^ *//'
  } > "$BASELINE"
  echo "baseline updated: $BASELINE now has $(grep -vc '^#' "$BASELINE") entries"
  exit 0
fi

N_VIOL=0; [ -n "$VIOL" ] && N_VIOL=$(printf '%s' "$VIOL" | grep -c .)

# ⭐⭐⭐ 2026-10-07 修 fail-open：账本缺失 ⇒ 棘轮静默失效 ⇒ `--strict` 假绿。
#
# # 实测（在隔离副本里，2 个真违规）
#   有账本 → rc=1 ✅   删掉账本 → **rc=0** ⛔ 即「2 个真违规被判通过」。
#
# # 为什么这是 fail-open 而不是「无害降级」
#
# 本门的两类判据里，**只有「新增」会判红**（存量一律放过，靠棘轮记账）。
# ⇒ 账本是新增判据的**唯一依据**；账本没了，`N_NEW` 恒为 0，
#   `--strict` 就**结构上不可能失败** —— 正是 LESSONS L8 说的
#   「报 PASS 却结构上不可能失败」，比没有门更危险（它训练人忽略红色）。
# ⛔ 触发条件现实：账本是 tracked 文件，一次误删/一次 bad merge/一次
#   `git clean` 之后的 checkout 事故就能造成，**且不会有任何提示**。
#
# ✅ 修法：**fail-closed**。`--strict` 下账本缺失 = 门无法履行职责 ⇒ 判红，
#   并说清怎么补。advisory 模式只警告、不阻断（保持「只跑不判」的用法）。
if [ ! -f "$BASELINE" ]; then
  if [ "$STRICT" -eq 1 ]; then
    echo "FAIL(strict): 账本 $BASELINE 不存在 —— 棘轮无法工作，本门结构上不可能失败。"
    echo "  ⛔ 不把它当「无存量所以通过」：删掉账本会让**新增判据整体失效**。"
    echo "  修法：bash scripts/check-layout.sh --update-baseline 重建账本，然后复查账本内容。"
    exit 1
  fi
  echo "⚠ 警告: 账本 $BASELINE 不存在 —— 新增判据当前失效（本门只报存量，不判新增）。"
fi

N_NEW=0
if [ -f "$BASELINE" ] && [ "$N_VIOL" -gt 0 ]; then
  N_NEW=$(comm -23 \
    <(printf '%s' "$VIOL" | grep . | sort -u) \
    <(grep -v '^#' "$BASELINE" 2>/dev/null | sort -u) | grep -c .)
fi
[ -z "$N_NEW" ] && N_NEW=0
N_DOCS=0; [ -n "$DOCSVIOL" ] && N_DOCS=$(printf '%s' "$DOCSVIOL" | grep -c .)
# docs 类同样走账本棘轮：既存债记账，只拦新增
N_DOCS_NEW=0
if [ -f "$BASELINE" ] && [ "$N_DOCS" -gt 0 ]; then
  N_DOCS_NEW=$(comm -23 \
    <(printf '%s' "$DOCSVIOL" | sed 's/^ *//' | sort -u) \
    <(grep -v '^#' "$BASELINE" 2>/dev/null | sort -u) | grep -c .)
fi
[ -z "$N_DOCS_NEW" ] && N_DOCS_NEW=0
N_DOCGAP=0; [ -n "$DOCGAP" ] && N_DOCGAP=$(printf '%s' "$DOCGAP" | grep -c .)

N_IGN=0; [ -n "$IGN" ] && N_IGN=$(printf '%s' "$IGN" | grep -c .)

# ── 报告 ────────────────────────────────────────────────────────
echo "=== NeoTrix layout check ==="
echo "根目录项: $(grep -c . "$CUR")（另 $N_IGN 项 git 不管，见下方单列）   不在白名单: $N_VIOL   其中新增(不在账本): $N_NEW"
echo "neotrix-core/docs/ 违规: $N_DOCS   其中新增: ${N_DOCS_NEW}"
echo "规范 vs 现实发散: $N_DOCGAP"

if [ "$N_VIOL" -gt 0 ]; then
  echo "--- 不在白名单的根目录项（存量 ${N_VIOL} / 新增 ${N_NEW}）---"
  printf '%s' "$VIOL" | sed 's/^/  /'
fi
if [ -n "$KNOWN" ]; then
  echo "--- 已知残留（committed 已移除、磁盘 untracked 残留；处置后从本表删）---"
  printf '%s' "$KNOWN" | sed 's/^/  /'
fi
if [ "$N_IGN" -gt 0 ]; then
  echo "--- git 不管（.gitignore 已声明 ⇒ 非布局违规，列出以免视野被缩）---"
  printf '%s' "$IGN" | sed 's/^/  /'
fi
if [ "$N_DOCS" -gt 0 ]; then
  echo "--- neotrix-core/docs/ 违反 DOCUMENTATION-MAP:81（禁研究笔记）；存量 $N_DOCS / 新增 ${N_DOCS_NEW} ---"
  printf '%s' "$DOCSVIOL"
fi
if [ "$N_DOCGAP" -gt 0 ]; then
  echo "--- 规范点名但磁盘不存在（规范腐化）---"
  printf '%s' "$DOCGAP"
  echo "  ⇒ 修规范或补文件。二选一，别让两边发散。"
fi

if [ "$N_NEW" -gt 0 ] || [ "$N_DOCS_NEW" -gt 0 ] || [ "$N_DOCGAP" -gt 0 ]; then
  if [ "$STRICT" -eq 1 ]; then
    echo "FAIL(strict): 新增违规 ${N_NEW} / docs 新增违规 ${N_DOCS_NEW} / 规范发散 ${N_DOCGAP}"
    echo "  修法三选一：① 把文件挪进合规目录；② 确有理由则扩 ALLOW_FILES 并写明消费者；"
    echo "            ③ 确认是既存债则 --update-baseline 记账（棘轮）。"
    echo "            （若该项属 .gitignore 已声明不管者 ⇒ 本门不判违规，见上方单列）"
    exit 1
  fi
fi
echo "DONE(advisory)."
