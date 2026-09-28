#!/bin/bash
# API surface inventory — NTS-D10 advisory implementation.
# Counts Tauri commands + Axum routes, checks for an OpenAPI spec.
# Without a spec there is nothing to diff (oasdiff) or fuzz (schemathesis):
# spec-first is step zero of the bake plan.
#
# 2026-09-28 修死引用（原 :12/:15 硬编码 src-tauri/src）：
#   桌面端已随 5c02e738 归档（599 files），但两处 rg 仍带该路径 ⇒
#   `rg` 对不存在路径 **exit 2** 并写 stderr，而 `2>/dev/null` 把它吞掉，
#   `awk` 仍照常输出 `0` ⇒ 脚本打印 **`Tauri commands: 0`** ——
#   一个**看起来正常的假数字**。读它的人会得出「本仓没有 Tauri 命令面」，
#   而真实原因是「我压根没扫那个面」。
#
#   **路径不存在 ≠ 该面为空** —— 这是扫描器的一等失败模式。
#   修法：① 删死引用 ② 面缺失时**显式报错**（`surface_missing`），绝不打 0。
#   同型问题的通用守门：`python3 scripts/ops/nt_scan_surface.py`
#
# 面状态（2026-09-28 实测）：
#   Tauri 命令面 = 0，因为**桌面端已不在本仓**（neobot 侧承接），
#   这是**事实**而非扫描失败。脚本显式区分二者。
#   Axum .route( ceiling = 277（overcounts nested/test code）
#   OpenAPI spec = 0（bake plan step 0 待做）
#
# 用法: bash scripts/check-api-surface.sh
# Note: bash-3.2-safe style. `bash -n` before commit.
set -uo pipefail

echo "=== NeoTrix API surface inventory (NTS-D10) ==="

# ── 面清单：每项显式声明，且**存在才计入**，不存在必须报出来 ──
# 格式:  名称|路径|存在时用什么 rg 模式
SURFACES=(
  "tauri-commands|neotrix-core/src|#\[tauri::command\]"
  "axum-routes|neotrix-core/src|\.route\("
)

surface_missing=""
# 只输出数字到 stdout；诊断一律走 stderr 且**不进管道**。
# 2026-09-28 首版把诊断 `echo` 在函数里、又被 `$(...)` 捕获 ⇒ 诊断混进返回值，
# 造成 `[: : integer expression expected` 且 stderr 泄漏到终端。
count_in() {
  # $1=模式 $2=路径。路径不存在 ⇒ 返回 2（**绝不返回 0**）
  _pat=$1; _root=$2
  if [ ! -d "$_root" ]; then
    printf '%s\n' "⛔ SURFACE-MISSING: '$_root' 不存在 —— 该面**未扫描**，不是「扫了为 0」" >&2
    return 2
  fi
  _n=$(rg -c "$_pat" --glob '*.rs' "$_root" 2>/dev/null | awk -F: '{s+=$2} END {print s+0}')
  if [ -z "$_n" ]; then
    printf '%s\n' "⛔ SCAN-FAILED: rg 扫 '$_root' 无输出 —— 结果不可信" >&2
    return 3
  fi
  printf '%s\n' "$_n"
}

# 用**退出码**判失败，不用输出值 —— `$(...)` 在函数失败时给空串，
# 对空串做 `[ "" -eq 2 ]` 会报 "integer expression expected"（首版真踩了）。
TAURI_ROOT="neotrix-core/src"
TAURI=$(count_in '#\[tauri::command\]' "$TAURI_ROOT")
if [ $? -ne 0 ]; then
  surface_missing="${surface_missing} tauri-commands"
  TAURI="?"
fi
echo "Tauri commands: $TAURI"

ROUTES=$(count_in '\.route\(' "$TAURI_ROOT")
if [ $? -ne 0 ]; then
  surface_missing="${surface_missing} axum-routes"
  ROUTES="?"
fi
echo "Axum .route( hits: $ROUTES (overcounts nested/test code; treat as ceiling)"

SPEC=$(rg --files . 2>/dev/null | rg -v "^./target/|/\.git/" | rg -i "openapi|swagger" | head -n 3)
if [ -z "$SPEC" ]; then
  echo "OpenAPI spec: MISSING (bake plan step 0: export spec from Axum/Tauri surface)"
else
  echo "OpenAPI spec:"
  echo "$SPEC"
fi

# ── 桌面端去向说明：把「为什么 tauri=0」讲清楚，而不是留一个裸 0 ──
if [ ! -d "src-tauri" ] && [ ! -d "apps/neobot-desktop" ]; then
  echo "Note: 桌面端不在本仓（5c02e738 归档 599 files）⇒ tauri-commands=0 是**事实**不是扫描失败。"
  echo "      现由 crates/neotrix-neobot 承接；本仓只余 Axum 面。"
fi

echo "Bake plan: export spec -> oasdiff breaking gate (fail-on-ERR) -> schemathesis 4 checks."
echo "DONE(advisory). Baseline: 0 tauri（本仓已无） / 277 routes-ceiling / 0 spec."

if [ -n "$surface_missing" ]; then
  echo "⛔ SURFACE MISSING:$surface_missing —— 上面标 ? 的数字**不可解读**，别当 0 用" >&2
  echo "   守门: python3 scripts/ops/nt_scan_surface.py" >&2
  exit 3
fi
exit 0
