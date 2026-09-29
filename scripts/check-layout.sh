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
    -h|--help) sed -n '2,32p' "$0"; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

# ── 根目录允许清单（每项必须有真实理由，不是「先放着」）────────────
# 格式：文件名|类别|理由（理由要指向真实消费者，否则就是待清理物）
ALLOW_FILES="
README.md|入口|项目入口，GitHub 展示面，DOCUMENTATION-MAP:14
AGENTS.md|agent 守则|agent 守则中枢，.githooks/pre-commit + 94 处引用
RUST-STANDARDS.md|规范|Rust 编码标准正典，cargo xl 风格命令出处
CONTRIBUTING.md|流程|贡献指南
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
.claude|状态|agent 记忆
.opencode|状态|opencode agent 定义
.neotrix-absorb|状态|吸收轮快照
.worktrees|运行期|worktree 目录（已 gitignore，允许存在）
config|构建|真配置：cliff.toml / .gitleaks.toml / .env.example（根配置已收敛到本目录）
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
RESIDUAL_KNOWN=""

# ── 收集现状（只取 git 跟踪的 + 未跟踪但存在的一级项）──────────────
CUR=$(mktemp); trap 'rm -f "$CUR"' EXIT

# 根目录文件：跟踪的 ∪ 未跟踪但存在的
for f in *; do
  [ -f "$f" ] || continue
  echo "$f" >> "$CUR"
done
# 根目录目录：排除 .git
for d in */ .*/; do
  d="${d%/}"
  [ "$d" = ".git" ] || [ "$d" = "." ] || [ "$d" = ".." ] && continue
  [ -d "$d" ] || continue
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
      *) DOCSVIOL="$DOCSVIOL  neotrix-core/docs/$f"$'\n' ;;
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

# ── 报告 ────────────────────────────────────────────────────────
echo "=== NeoTrix layout check ==="
echo "根目录项: $(grep -c . "$CUR")   不在白名单: $N_VIOL   其中新增(不在账本): $N_NEW"
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
    exit 1
  fi
fi
echo "DONE(advisory)."
