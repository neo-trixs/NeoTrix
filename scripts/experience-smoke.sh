#!/bin/bash
# experience 回归冒烟 (P4 先决条件) — 全命令真 KB 回路, 断言退出码 + 关键输出。
# 用法: bash scripts/experience-smoke.sh
# 隔离: 临时 HOME, 不碰 ~/.neotrix。
set -uo pipefail

BIN="${BIN:-./target/debug/neotrix-experience}"
export HOME="$(mktemp -d /tmp/exp-smoke-XXXXXX)"
trap 'rm -rf "$HOME"' EXIT

pass=0; fail=0
check() { # $1=描述 $2..=命令
  local desc="$1"; shift
  local out code
  if out=$("$@" 2>&1); then
    pass=$((pass + 1)); echo "ok: $desc"
  else
    code=$?; fail=$((fail + 1)); echo "FAIL($code): $desc"; echo "$out" | head -n 5
  fi
}
check_out() { # $1=描述 $2=期望子串 $3..=命令
  local desc="$1" want="$2"; shift 2
  local out
  if out=$("$@" 2>&1) && echo "$out" | grep -q "$want"; then
    pass=$((pass + 1)); echo "ok: $desc"
  else
    fail=$((fail + 1)); echo "FAIL: $desc (want '$want')"; echo "$out" | head -n 5
  fi
}

[ -x "$BIN" ] || { echo "先构建: cargo build -p neotrix --bin neotrix-experience"; exit 2; }

SESSION="$HOME/session.json"
cat > "$SESSION" <<'EOF'
{"session_id": "smoke-001", "cycle": "001", "domain": "NT-CORE",
 "content": "冒烟测试：本地知识库回归验证哨兵经验条目",
 "evidence": "scripts/experience-smoke.sh:1"}
EOF

check "snapshot" "$BIN" snapshot --cycle 001 --task "smoke" --domain NT-CORE
check_out "absorb" "smoke-001" "$BIN" absorb "$SESSION"
check_out "query 命中" "smoke" "$BIN" query --kw "冒烟测试" --limit 5
check "list" "$BIN" list --domain NT-CORE
check "stale" "$BIN" stale --domain NT-CORE
check "hub" "$BIN" hub
check "hebb" "$BIN" hebb
check "dedup dry" "$BIN" dedup --dry-run
check "distill dry" "$BIN" distill --domain NT-CORE --dry-run
check "close" "$BIN" close --cycle 001
check "sim" "$BIN" sim --a "hello world" --b "hello there" --dim 32
check "topology" "$BIN" topology --dim 16 --steps 2 --max-points 8

echo "=== pass=$pass fail=$fail ==="
[ "$fail" -eq 0 ]
