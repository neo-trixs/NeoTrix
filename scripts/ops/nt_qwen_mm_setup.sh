#!/usr/bin/env bash
# nt_qwen_mm_setup.sh — Qwen-MM-Plugins 无 key 落地（体的方案）。
#
# 背景：core 7 工具原生无 key（本地模式），search/api/omni 系才要 key。
# 本脚本把"无 key 即生产"变成一条命令：sparse-checkout 源 → venv →
# pip 装无 key 依赖 → bin 垫片 → 握手自检。Rust 侧零改动：
# 垫片进 PATH 即 `LaunchVia::Path` 命中（见 nt_qwen_mm_manifests.rs）。
#
# 用法: bash scripts/ops/nt_qwen_mm_setup.sh [setup|check|update]
#   setup（默认）: 全量 provision（幂等，已有部分跳过）
#   check        : 只报告状态，不产生副作用（可放心跑）
#   update       : git pull 最新 core 源（tag 不动，只跟 main；发版钉死仍以
#                  nt_qwen_mm_manifests.rs 的 VERSION 常量为准）
#
# 落盘：${QWEN_MM_HOME:-~/.neotrix/qwen-mm}/{src,venv,bin}
#   仓外目录，不污染 git；worktree prune 安全；删除即卸载。
#
# 无 key 依赖集（上游 base 子集＋本地渲染）：
#   mcp/pydantic/pillow/anyio/docstring-parser（E2E 已验：mcp 1.30 等）
#   pypdfium2（PDF 页渲染）· nibabel+numpy（NIfTI 体渲染）
# 不装：dashscope（云 VL/Omni/生成，要 key，不在无 key 路径内）。

set -euo pipefail

ROOT="${QWEN_MM_HOME:-$HOME/.neotrix/qwen-mm}"
SRC="$ROOT/src"
VENV="$ROOT/venv"
BIN="$ROOT/bin"
PKGDIR="$SRC/src/capabilities/core/qwen_mm_plugins_core"
PY="$VENV/bin/python"

log() { printf '[qwen-mm-setup] %s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

do_clone() {
  if [ -d "$SRC/.git" ]; then
    log "src 已存在（$SRC），跳过 clone（用 update 更新）"
    return 0
  fi
  log "sparse-checkout core 源（48 py / ~1M，只要源不要字体资产）…"
  git clone --depth 1 --filter=blob:none --sparse \
    https://github.com/QwenLM/Qwen-MM-Plugins.git "$SRC"
  git -C "$SRC" sparse-checkout set --skip-checks \
    src/capabilities/core src/shared src/mcp_framework.py plugin-versions.json
  log "checkout 完成：$(find "$SRC/src" -name '*.py' | wc -l | tr -d ' ') py 文件"
}

do_venv() {
  if [ -x "$PY" ]; then
    log "venv 已存在，跳过创建"
  else
    log "建 venv…"
    python3 -m venv "$VENV"
  fi
  log "装无 key 依赖（base＋pypdfium2＋nibabel）…"
  "$PY" -m pip install -q \
    "mcp>=1.0,<2" "pydantic>=2.11,<3" "pillow<12" "anyio>=4.0,<5" \
    "docstring-parser>=0.18,<0.19" \
    "pypdfium2" "nibabel>=5,<6"
  log "pip 完成：$("$PY" -m pip list 2>/dev/null | grep -ciE 'mcp|pydantic|pillow|pypdfium2|nibabel') 个相关包"
}

do_shim() {
  mkdir -p "$BIN"
  cat > "$BIN/qwen-mm-plugins-core" <<EOF
#!/usr/bin/env bash
# 自动生成（nt_qwen_mm_setup.sh）：LaunchVia::Path 垫片，零 Rust 改动接入。
exec "$PY" "$PKGDIR" "\$@"
EOF
  chmod +x "$BIN/qwen-mm-plugins-core"
  log "垫片：$BIN/qwen-mm-plugins-core"
}

do_verify() {
  log "自检 1/3 --version：$("$PY" "$PKGDIR" --version 2>&1 | head -1)"
  log "自检 2/3 --check-system："
  "$PY" "$PKGDIR" --check-system 2>&1 | head -6 | sed 's/^/    /'
  log "自检 3/3 tools/list 握手（与 Rust 客户端同帧）："
  "$PY" - "$PKGDIR" <<'PYEOF'
import json, subprocess, sys, time
pkg = sys.argv[1]
srv = subprocess.Popen([sys.executable, pkg], stdin=subprocess.PIPE,
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
def send(o): srv.stdin.write(json.dumps(o) + "\n"); srv.stdin.flush()
send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
    "protocolVersion": "2024-11-05", "capabilities": {},
    "clientInfo": {"name": "neotrix-setup-check", "version": "0"}}})
send({"jsonrpc": "2.0", "method": "notifications/initialized"})
send({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
names = []
deadline = time.time() + 30
while time.time() < deadline:
    line = srv.stdout.readline()
    if not line:
        break
    try:
        v = json.loads(line.strip())
    except Exception:
        continue
    if v.get("id") == 2 and "result" in v:
        names = [t["name"] for t in v["result"].get("tools", [])]
        break
srv.stdin.close()
srv.wait(timeout=15)
print("    tools:", " ".join(names) if names else "NONE — 自检失败")
sys.exit(0 if len(names) == 7 else 1)
PYEOF
}

do_check() {
  local ok=1
  [ -d "$SRC/.git" ] && echo "src: OK ($SRC)" || { echo "src: MISSING"; ok=0; }
  [ -x "$PY" ] && echo "venv: OK" || { echo "venv: MISSING"; ok=0; }
  [ -x "$BIN/qwen-mm-plugins-core" ] && echo "shim: OK" || { echo "shim: MISSING"; ok=0; }
  if [ "$ok" = 1 ]; then
    echo "PATH 接入: export PATH=\"$BIN:\$PATH\"  （或设 QWEN_MM_PLUGINS_CHECKOUT=$SRC 走源码模式）"
  else
    echo "跑全量：bash scripts/ops/nt_qwen_mm_setup.sh setup"
    return 1
  fi
}

case "${1:-setup}" in
  setup)  do_clone; do_venv; do_shim; do_verify
          log "完成。接入：export PATH=\"$BIN:\$PATH\""
          log "search/omni 系仍要 key（credential 门），不在本脚本内。" ;;
  check)  do_check ;;
  update) git -C "$SRC" pull --depth 1 2>&1 | tail -2 ;;
  -h|--help) sed -n '2,20p' "$0" ;;
  *) echo "unknown arg: $1 (setup|check|update)" >&2; exit 2 ;;
esac
