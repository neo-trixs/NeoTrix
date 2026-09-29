#!/usr/bin/env bash
# check-gitleaks-version.sh — 基线/门版本对齐自门
#
# ## 为什么需要这道门（2026-09-29 实测事故）
#
# `config/.gitleaks-baseline.txt` 里的 fingerprint 是**用某个特定版本的
# gitleaks 生成的**。而 CI 装的是另一个版本（本例：基线按 8.30.1 生成，
# CI 却装 8.18.2）。
#
# 后果：8.18.2→8.30.1 之间规则集有**实质变更** ——
#   v8.25  +clickhouse / perplexity 检测
#   v8.28  +Anthropic API key + composite rules（required/proximity）
#   v8.30  +Airtable Personal Access Token + Looker client secret
# ⇒ 新版报出的指纹不在旧基线内 ⇒ **CI 首跑即红**。
#
# 而那不是真泄漏，是**版本错配**。两种坏法：
#   ① 不对齐 → CI 假红，训练人忽略红色（恒红的门比没有门更坏）
#   ② 无脑重生成基线 → 把真泄漏一起写进基线，门彻底失效
#
# ## 本门做什么
#
# 校验三处版本号一致：
#   1. .github/workflows/security-scan.yml 的 GITLEAKS_VERSION
#   2. config/.gitleaks-baseline.txt 头注声明的绑定版本
#   3. 本机 gitleaks 实测版本（若装了）—— 不一致时**只警告不阻断**，
#      因为本地版本与 CI 不同是常态（本地用于生成基线，CI 用于判定）
#
# 用法：bash scripts/check-gitleaks-version.sh [--strict]
#   默认 advisory；--strict 在 workflow 声明与基线头注不一致时 exit 1。

set -uo pipefail

STRICT=0
for arg in "$@"; do
  case "$arg" in
    --strict) STRICT=1 ;;
    -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

WF=".github/workflows/security-scan.yml"
BASE="config/.gitleaks-baseline.txt"
rc=0

[ -f "$WF" ]   || { echo "missing: $WF";   exit 2; }
[ -f "$BASE" ] || { echo "missing: $BASE"; exit 2; }

# ── 1. workflow 声明的版本 ──────────────────────────────────────────
WF_VER=$(grep -oE 'GITLEAKS_VERSION: *[0-9]+\.[0-9]+\.[0-9]+' "$WF" 2>/dev/null \
         | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')

# ── 2. 基线头注声明的绑定版本 ───────────────────────────────────────
BASE_VER=$(grep -oE '绑定 gitleaks [0-9]+\.[0-9]+\.[0-9]+' "$BASE" 2>/dev/null \
           | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')

echo "=== gitleaks version alignment (R-P79 闭环) ==="
echo "  workflow GITLEAKS_VERSION : ${WF_VER:-<未找到>}"
echo "  baseline 绑定版本        : ${BASE_VER:-<未找到>}"

if [ -z "$WF_VER" ]; then
  echo "  ⛔ workflow 里找不到 GITLEAKS_VERSION —— 版本被硬编码回 URL 了？"
  rc=1
elif [ -z "$BASE_VER" ]; then
  echo "  ⛔ 基线头注没声明绑定版本 ⇒ 版本漂移无从发现。"
  echo "     修法：在 $BASE 头注加一行「绑定 gitleaks X.Y.Z」"
  rc=1
elif [ "$WF_VER" != "$BASE_VER" ]; then
  # ⚠️ ${} 是必需的：$VAR 紧跟全角标点（，。）时 bash-3.2 的参数展开边界
  #    会吃掉下一个字节，表现为 `${VAR}` 之外的 unbound variable 报错。
  echo "  ⛔ 版本错配：CI 装 ${WF_VER}，基线按 ${BASE_VER} 生成。"
  echo "     规则集变 ⇒ 基线必重生成；**重生成前必须逐条人工判读新增项**。"
  rc=1
else
  echo "  ✓ 对齐（均为 ${WF_VER}）"
fi

# ── 3. 本机实测版本（只警告）────────────────────────────────────────
if command -v gitleaks >/dev/null 2>&1; then
  LOCAL=$(gitleaks version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
  echo "  本机 gitleaks            : ${LOCAL:-<未知>}"
  if [ -n "${LOCAL:-}" ] && [ -n "${BASE_VER:-}" ] && [ "$LOCAL" != "$BASE_VER" ]; then
    echo "  ⚠️  本机版本与基线不同 —— 若要用本机重生成基线，生成后请同步更新头注与 workflow"
  fi
else
  echo "  本机 gitleaks            : <未安装，跳过>"
fi

# ── 4. 基线自身完整性 ──────────────────────────────────────────────
N=$(grep -cE '^[0-9a-f]{40}:' "$BASE" 2>/dev/null)
echo "  基线 fingerprint 条数     : $N"
if [ "$N" -eq 0 ]; then
  echo "  ⛔ 基线为空 —— 门会变成「任何泄漏都红」或（更糟）「什么都不查」"
  rc=1
fi

if [ "$rc" -ne 0 ]; then
  if [ "$STRICT" -eq 1 ]; then
    echo "FAIL(strict): 见上方 ⛔"
    exit 1
  fi
  echo "advisory: 有问题（advisory 模式不阻断）"
fi
echo "DONE."
