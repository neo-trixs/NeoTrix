#!/usr/bin/env bash
# ============================================================
# NeoTrix Smart Commit — 智能化提取 git 变更生成结构化 commit message
# v1.0 — 能力沉淀: 替代手写 commit message 的启发式提取
#
# 用法:
#   scripts/git-smart-commit.sh              # 从 staged changes 提取 (推荐: git add 后运行)
#   scripts/git-smart-commit.sh --diff A B   # 从任意两个 commit 的 diff 提取
#   scripts/git-smart-commit.sh --stage      # 显式 staged 模式 (默认)
#   scripts/git-smart-commit.sh --last       # 从最近一次 commit 提取
#
# 输出: 结构化 commit message (含类型/scope/摘要/要点), 可用于 git commit -m
# ============================================================
set -euo pipefail

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
[ -z "$PROJECT_ROOT" ] && { echo "❌ 不在 git 仓库内"; exit 1; }

# ── 参数解析 ──
MODE="stage"
DIFF_A=""
DIFF_B=""
while [ $# -gt 0 ]; do
  case "$1" in
    --stage) MODE="stage"; shift ;;
    --last)  MODE="last"; shift ;;
    --diff)  MODE="diff"; shift; DIFF_A="${1:-}"; shift; DIFF_B="${1:-}"; shift ;;
    *) if [ -z "$DIFF_A" ]; then DIFF_A="$1"; elif [ -z "$DIFF_B" ]; then DIFF_B="$1"; fi; shift ;;
  esac
done

# ── 获取 diff ──
case "$MODE" in
  stage) DIFF_CMD="git diff --cached" ;;
  last)  DIFF_CMD="git diff HEAD^ HEAD" ;;
  diff)  DIFF_CMD="git diff ${DIFF_A} ${DIFF_B}" ;;
esac

# 辅助: 执行 diff 命令 (多词变量必须 eval, 否则词分割失败)
diff_run() {
  eval "$DIFF_CMD $1" 2>/dev/null
}

# 变更文件列表
CHANGED_FILES=$(diff_run "--name-only" | grep -v "^$" | head -30)
[ -z "$CHANGED_FILES" ] && { echo "⚠️ 无变更可提取 (staged 为空? 用 git add 或 --diff A B)"; exit 1; }

# ── 1. 类型提取 (type) ──
# 基于变更文件路径 + 内容特征
TYPE="feat"  # 默认
if echo "$CHANGED_FILES" | grep -qE "test[s]?/|_test\.|tests/|\.spec\.|__tests__"; then
  TYPE="test"
fi
if echo "$CHANGED_FILES" | grep -qE "docs/|\.md$|README|CHANGELOG|\.adoc"; then
  TYPE="docs"
fi
# 内容特征: 删除占比高 → fix/remove; 只改配置 → chore
DEL_LINES=$(diff_run "--numstat" | awk '{d+=$2} END {print d+0}')
ADD_LINES=$(diff_run "--numstat" | awk '{a+=$1} END {print a+0}')
TOTAL=$((DEL_LINES + ADD_LINES))
# 小修改 (≤3行) 且增删相当 → 修复性改动
if [ "$TOTAL" -gt 0 ] && [ "$TOTAL" -le 3 ] && [ "$DEL_LINES" -gt 0 ]; then
  TYPE="fix"
fi
if [ "$TOTAL" -gt 0 ] && [ $((DEL_LINES * 100 / TOTAL)) -gt 70 ]; then
  # 大量删除 → fix(移除坏代码) 或 remove(删除功能)
  if echo "$CHANGED_FILES" | grep -qE "src/|lib/"; then
    TYPE="fix"
  else
    TYPE="remove"
  fi
fi
if echo "$CHANGED_FILES" | grep -qE "\.(lock|toml)$|package\.json|Cargo\.toml" \
   && [ $((ADD_LINES + DEL_LINES)) -lt 20 ]; then
  TYPE="chore"
fi

# ── 2. scope 提取 (基于路径推断模块) ──
# 优先核心模块 (core/neotrix 实现), 非 cli/scripts 包装层
SCOPE="core"  # 默认
for file in $CHANGED_FILES; do
  case "$file" in
    neotrix-core/src/core/*)              SCOPE="core" ;;
    neotrix-core/src/neotrix/l1_body_impl/nt_io_provider/*)  SCOPE="nt-io" ;;
    neotrix-core/src/neotrix/l3_memory_impl/*)               SCOPE="nt-memory" ;;
    neotrix-core/src/neotrix/l2_world_impl/*)                SCOPE="nt-world" ;;
    neotrix-core/src/neotrix/l8_autonomic_impl/*)            SCOPE="nt-mind" ;;
    neotrix-core/src/cli/*)               SCOPE="cli" ;;
    src-tauri/*)                          SCOPE="desktop" ;;
    web/*|src/*.tsx|src/*.ts)             SCOPE="frontend" ;;
    docs/*)                               SCOPE="docs" ;;
    scripts/*)                            SCOPE="scripts" ;;
    *.sh)                                 SCOPE="scripts" ;;
  esac
  # 若命中核心实现模块 (core/neotrix 非 cli), 优先采用并停止
  case "$file" in
    neotrix-core/src/core/*|neotrix-core/src/neotrix/*) SCOPE="core"; break ;;
  esac
done

# ── 3. 摘要提取 (从 diff 头 + 文件名) ──
# 提取新增/修改的核心实体 (函数/结构体/模块名) — 过滤纯代码行
HINTS=$(diff_run "" | grep -oE "^[+-].*fn [a-z_]+|^[+-].*pub struct [A-Z][a-zA-Z]+|^[+-].*pub enum [A-Z][a-zA-Z]+" \
  | head -8 | sed -E 's/^[+-]//; s/^[[:space:]]*//' || true)

# 优先提取 diff 中的中文注释 (语义信号最强) — 只匹配 // 注释行, 过滤代码/HTML
CN_HINTS=$(diff_run "" | grep -oE "^\+[^+].*//.*[一-龥]{4,}.*" | head -5 \
  | sed -E 's/^\+//; s/^[[:space:]]*//' \
  | sed -E 's/^\/\/[[:space:]]*//; s/\/\/.*$//; s/<[^>]*>//g; s/^[[:space:]]*//' \
  | grep -vE "^$" | cut -c1-50 | head -3 || true)

# 提取主要变更文件的路径基名 (排除 test)
MAIN_FILES=$(echo "$CHANGED_FILES" | grep -vE "_test\.|/tests/|test[s]?/" | head -3 \
  | sed -E 's#.*/([^/]+)$#\1#; s/\.rs$//; s/\.tsx?$//; s/\.sh$//')

# 组装摘要: 优先中文注释 → 函数签名 → 文件名
if [ -n "$CN_HINTS" ]; then
  # 精炼: 取注释中冒号/破折号前的简短主题, 限 45 字符
  SUMMARY="$(echo "$CN_HINTS" | head -1 | sed -E 's/[:：—].*$//; s/[[:space:]]+$//' | cut -c1-45)"
  [ -z "$SUMMARY" ] && SUMMARY="$(echo "$CN_HINTS" | head -1 | cut -c1-45)"
elif [ -n "$HINTS" ]; then
  FIRST_HINT=$(echo "$HINTS" | head -1)
  SUMMARY="$(echo "$FIRST_HINT" | sed -E 's/^fn |^pub (struct|enum) //' | cut -c1-60)"
else
  # 无注释/签名信号时: 用文件名 + 语义动词
  SUMMARY="$(echo "$MAIN_FILES" | head -1 | cut -c1-45)"
  case "$TYPE" in
    fix) SUMMARY="${SUMMARY:-变更} 修复" ;;
    feat) SUMMARY="${SUMMARY:-变更} 增强" ;;
    docs) SUMMARY="${SUMMARY:-文档} 更新" ;;
    *) SUMMARY="${SUMMARY:-变更}" ;;
  esac
fi

# ── 4. 要点提取 (从 diff 上下文, 提取注释) — 过滤代码行 ──
BULLETS=$(diff_run "" | grep -oE "^\+[^+].*//.*[一-龥A-Za-z].*" | head -8 \
  | sed -E 's/^\+//; s/^[[:space:]]*//' \
  | sed -E 's/^\/\/[[:space:]]*//; s/\/\/.*$//; s/<[^>]*>//g; s/^[[:space:]]*//' \
  | grep -vE "^use |^import |^#|^$|^[a-zA-Z_][a-zA-Z0-9_]*\(|^(else|if|return|const|let|function|pub|fn|async|await|for|while)\b" \
  | cut -c1-70 | head -5 || true)

# ── 输出 ──
echo ""
echo "=============================================="
echo "📦 Smart Commit 提取结果"
echo "=============================================="
echo ""
echo "▶ 类型 (type):   $TYPE"
echo "▶ 范围 (scope):  $SCOPE"
echo "▶ 摘要 (summary): $SUMMARY"
echo ""
echo "▶ 变更文件 ($(echo "$CHANGED_FILES" | wc -l | tr -d ' ') 个):"
echo "$CHANGED_FILES" | sed 's/^/    /'
echo ""
echo "▶ 推断要点:"
if [ -n "$BULLETS" ]; then
  echo "$BULLETS" | sed 's/^/    - /'
else
  echo "    (无注释要点, 基于 $MAIN_FILES 推断)"
fi
echo ""
echo "▶ 建议 commit 命令:"
echo "    git commit -m \"$TYPE($SCOPE): $SUMMARY\""
echo ""
echo "=============================================="
