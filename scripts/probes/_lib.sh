#!/bin/bash
# 探针公共库 — 供 scripts/probes/*.sh 使用
#
# 契约（**三条，缺一即探针无效**）：
#   1. 注入一个**已知违规**
#   2. 跑门
#   3. 断言门**变红**（rc != 0）
#   4. **清理**（trap 保证，即使断言失败也还原）
#
# ⛔ 探针**绝不能**在失败时留下脏状态 —— 否则一次探针执行会污染仓库。
#   本库所有注入都走 `cleanup` trap。
set -uo pipefail

PROBE_FAIL() { echo "PROBE-BROKEN: $*" >&2; exit 2; }

# 注入一个 0 字节的 .rs（会触发 EMPTY 类违规）
inject_empty_rs() {
  local target="$1"
  [ -n "$target" ] || PROBE_FAIL "inject_empty_rs 需要目标路径"
  [ -e "$target" ] && PROBE_FAIL "注入目标已存在: $target"
  : > "$target"
}

# 注入一个未被任何 mod 声明的 .rs
inject_undeclared_rs() {
  local target="$1"
  [ -n "$target" ] || PROBE_FAIL "inject_undeclared_rs 需要目标路径"
  [ -e "$target" ] && PROBE_FAIL "注入目标已存在: $target"
  printf '//! probe injection\n' > "$target"
}

# 注入一条 CI job，其 working-directory 指向 git 未跟踪的路径
inject_phantom_ci_job() {
  local wf="$1" dir="$2"
  [ -f "$wf" ] || PROBE_FAIL "CI 文件不存在: $wf"
  cp "$wf" "$wf.probe.bak"
  cat >> "$wf" <<EOF

  probe-injected-phantom:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: $dir
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
EOF
}

# 注入一条明知错误的 description（触发 M3 drift）
inject_description_drift() {
  local f="$1"
  [ -f "$f" ] || PROBE_FAIL "目标不存在: $f"
  cp "$f" "$f.probe.bak"
  sed -i.bak2 's/^description: .*/description: PROBE-INJECTED-DRIFT-DESCRIPTION/' "$f"
  rm -f "$f.bak2"
}

# 注入一个缺 module doc 的 .rs（触发 doc-drift）
inject_undocumented_rs() {
  local target="$1"
  [ -n "$target" ] || PROBE_FAIL "inject_undocumented_rs 需要目标路径"
  [ -e "$target" ] && PROBE_FAIL "注入目标已存在: $target"
  printf '// probe injection without module doc\n' > "$target"
}

# 断言门变红
# ⚠️ ⚠️ **不要用 `local rc="$1"`** —— 在 `set -u` 下，
# bash 会把 `local rc="$1"` 与后续同名的 `$?` 混在一起解析，
# 实测报 `rc: unbound variable`（bash 3.2 的已知怪癖：
# `local var=$(cmd)` 会让 `$?` 拿到 local 自己的状态）。
# ⇒ 先取参，再声明。
assert_gate_red() {
  local gate="$1"
  local rc="$2"
  if [ "$rc" -eq 0 ]; then
    echo "PROBE-BROKEN: $gate 在注入违规后仍返回 0 ⇒ 门是空的（假门）" >&2
    exit 2
  fi
  # ⚠️ 必须用 `${rc}` 加花括号边界！
  # 写成 `"$rc（…"` 时 bash 会把 `$rc（` 当成**一个**变量名
  # （全角括号在变量名里合法），实测报 `rc（: unbound variable`。
  # 这是中文文案 + bash 变量展开的经典冲突。
  echo "PROBE-OK: ${gate} 注入违规后 exit=${rc}（正确变红）"
  exit 0
}
