#!/bin/bash
# CI 引用门 —— 拦住「job 指向 git 里不存在的目录」这类幻影门。
#
# 动机（2026-09-29 实测，见 EVOLUTION-ROADMAP-CODE-NODES-2026-09-29.md N-1）：
# ci.yml 有 4 个 job（frontend-tests / frontend-coverage / frontend-build / e2e）
# 指向 `neocodex-frontend/` 与 `e2e/`。这两个目录：
#   - `neocodex-frontend/` 在 .gitignore:136 被**整目录忽略**
#   - 两者 `git ls-files` 跟踪数都是 **0**，磁盘上也不存在
#   - `src-tauri/` 已于 5c02e738 归档
# ⇒ 这 4 个 job 在**任何干净检出上必然失败**。它们不是"偶尔红"，是恒红。
#
# 为什么这值得一道门（`awesome-dsh-plugin/.github/workflows/pr-gate.yml:44-60` 的原话）：
#   "A gate that dies before posting is indistinguishable from one that never needed
#    to run: the PR simply carries no verdict, and a green `check` beside it reads
#    as ready to merge."
# 恒红的门比没有门更坏 —— 它训练人忽略红色。
#
# 本仓已因此付出过代价：提交 3edf3be7
# 「发布链路: 移除幻影CSS门禁 (脚本不存在, 构建恒失败)」修的是同一类问题，
# 修在 package.json；2026-09-29 复发在 CI 层。
#
# 检查项：
#   1) WORKING_DIR   job 的 `working-directory:` 指向的目录必须被 git 跟踪
#   2) PATH          upload-artifact 的 `path:` 同理（对目录前缀判定）
#   3) CACHE_DEP     `cache-dependency-path:` 指向的文件必须被 git 跟踪
#
# 豁免：路径含 `${{` 的（GitHub 表达式，运行期才求值）——但会打印出来供人核对。
#
# 用法：
#   bash scripts/check-ci-refs.sh              # advisory, exit 0
#   bash scripts/check-ci-refs.sh --strict     # 有未豁免的坏引用则 exit 1
#
# Note: bash-3.2-safe style (macOS system bash). `bash -n` before commit.

set -uo pipefail

STRICT=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

WF_DIR=".github/workflows"
[ -d "$WF_DIR" ] || { echo "no $WF_DIR — nothing to check"; exit 0; }

# git 跟踪的文件集合（一次性，避免每个引用都跑一次 git）
TRACKED=$(mktemp)
EXPR=$(mktemp)
trap 'rm -f "$TRACKED" "$EXPR"' EXIT
git ls-files 2>/dev/null | sort -u > "$TRACKED" || : > "$TRACKED"

BAD=$(mktemp)
ART=$(mktemp)
trap 'rm -f "$TRACKED" "$EXPR" "$BAD" "$ART"' EXIT

# 判定：$1 是工作树中的路径（目录或文件）。
# 目录 ⇒ 只要 git 跟踪了其中**任意一个**文件就算「存在」。
# 这比判定目录本身被跟踪更准：`git ls-files` 列的是文件，不列目录。
tracked_somewhere() {
  local p="$1"
  p="${p%/}"                       # 去掉尾斜杠
  [ -n "$p" ] || return 0
  # 前缀匹配（目录）：tracked 里任何以 p/ 开头
  if grep -q "^$p/" "$TRACKED" 2>/dev/null; then return 0; fi
  # 精确匹配（文件）
  if grep -qxF "$p" "$TRACKED" 2>/dev/null; then return 0; fi
  return 1
}

check_ref() {
  # check_ref <kind> <file> <lineno> <value>
  local kind="$1" file="$2" lineno="$3" val="$4"
  val="${val%\"}"; val="${val#\"}"; val="${val%\'}"; val="${val#\'}"
  val="${val%/}"
  # 空值没有可校验的对象，直接放过（不是违规）
  [ -n "$val" ] || return 0
  case "$val" in
    /*|./*|~*) echo "EXPR $file:$lineno $kind=$val" >> "$EXPR"; return 0 ;;
  esac
  tracked_somewhere "$val" && return 0
  echo "$file:$lineno $kind=$val" >> "$BAD"
}

# 本门只管「**必须在干净检出里存在**」的引用，即 job 实际要 cd 进去 / 要 checkout
# 出来 / 要当缓存键的路径。
#
# **不管**的是「构建产物」：upload-artifact 的 path 指向的常常是本次 job 刚生成的
# 文件（lcov.info / sbom / *.json 报告 / target/criterion），它们**理应**不在 git 里。
# 判据是**语义**而非「是否被跟踪」：产物路径本身就是「这次跑出来的」。
#
# 但这带来一个真实风险：`upload-artifact` 的 path 指错时 `if-no-files-found: ignore`
# 会**静默跳过上传**而不报错 —— 那和 phantom job 同一类病。
# ⇒ 所以产物路径另列一档（`--check-artifacts` 才查），默认不计入 FAIL。
ARTIFACT_DEFAULT='^(target/|.*/target/)|(^|/)(lcov\.info|sbom|scan-results\.txt|gitleaks-report\.json|criterion)$|\.vitepress/'
is_artifact_path() {
  printf '%s' "$1" | grep -qE "$ARTIFACT_DEFAULT"
}

for wf in "$WF_DIR"/*.yml "$WF_DIR"/*.yaml; do
  [ -f "$wf" ] || continue
  grep -nE '^[[:space:]]*(working-directory|cache-dependency-path|path):' "$wf" 2>/dev/null |
  while IFS= read -r line; do
    ln=${line%%:*}
    rest=${line#*:}
    key=$(printf '%s' "$rest" | sed -E 's/^[[:space:]]*([A-Za-z-]+):.*/\1/')
    val=$(printf '%s' "$rest" | sed -E 's/^[[:space:]]*[A-Za-z-]+:[[:space:]]*//' |
          sed -E "s/[[:space:]]+#.*$//" | sed -E 's/[[:space:]]+$//')
    printf '%s\t%s\t%s\n' "$key" "$ln" "$val"
  done > /tmp/.ci_refs.$$
  # while-in-pipe 在子 shell，文件重定向才能传出结果
  while IFS=$'\t' read -r key ln val; do
    case "$val" in
      '${{'*) echo "EXPR $wf:$ln $key=$val" >> "$EXPR"; continue ;;
    esac
    # 块标量 `path: |` 的值是 "|"，后面跟的是多行列表 —— 逐行单独处理
    if [ "$val" = "|" ] || [ "$val" = ">" ] || [ "$val" = "|-" ] || [ "$val" = ">-" ]; then
      continue
    fi
    # 产物路径另列，不计入 FAIL（除非 CHECK_ARTIFACTS=1）
    # 注意：判定前先去掉尾斜杠，与 check_ref 内部保持同一套规范化，
    # 否则 `sbom/` 与 `sbom` 会被判成两类 —— 这正是本门第一版的 bug。
    if is_artifact_path "${val%/}"; then
      echo "$wf:$ln $key=$val" >> "$ART"
      continue
    fi
    check_ref "$key" "$wf" "$ln" "$val"
  done < /tmp/.ci_refs.$$
  rm -f /tmp/.ci_refs.$$
done

sort -u "$BAD" -o "$BAD" 2>/dev/null || : > "$BAD"
sort -u "$EXPR" -o "$EXPR" 2>/dev/null || : > "$EXPR"
sort -u "$ART" -o "$ART" 2>/dev/null || : > "$ART"

N_BAD=$(grep -c . "$BAD" 2>/dev/null); N_BAD=${N_BAD:-0}
N_EXPR=$(grep -c . "$EXPR" 2>/dev/null); N_EXPR=${N_EXPR:-0}
N_ART=$(grep -c . "$ART" 2>/dev/null); N_ART=${N_ART:-0}

echo "=== CI reference gate ==="
echo "workflows scanned: $(ls "$WF_DIR"/*.yml "$WF_DIR"/*.yaml 2>/dev/null | wc -l | tr -d ' ')"
echo "bad refs (untracked target): $N_BAD    runtime-expression refs (not checked): $N_EXPR    artifact paths (not gated): $N_ART"

if [ "$N_ART" -gt 0 ] && [ "${CHECK_ARTIFACTS:-0}" -eq 1 ]; then
  echo "--- artifact paths (--check-artifacts): these are produced by the job, NOT in git ---"
  echo "    verify by hand that each is actually written; upload-artifact with"
  echo "    if-no-files-found:ignore fails SILENTLY when the path is wrong."
  cat "$ART"
fi

if [ "$N_EXPR" -gt 0 ]; then
  echo "--- runtime expressions (verify by hand; gate cannot evaluate these) ---"
  cat "$EXPR"
fi

if [ "$N_BAD" -gt 0 ]; then
  echo "--- offenders: path not tracked by git => job fails on every clean checkout ---"
  cat "$BAD"
  if [ "$STRICT" -eq 1 ]; then
    echo "FAIL(strict): $N_BAD CI reference(s) point at paths git does not track."
    echo "  Fix: point at a tracked path, or delete the job."
    echo "  A gate that is always red trains people to ignore red."
    exit 1
  fi
fi

echo "DONE(advisory)."
