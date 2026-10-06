#!/usr/bin/env bash
# ⛔ 只读仓库 + 只写临时目录。本脚本**绝不可**碰仓库里的任何文件。
#
# 门：能力注册表快照的**跨进程确定性**与**无损往返**。
#
# ## 为什么必须是「跨进程」的门（2026-10-06 实测教训）
#
# `CapabilityNode::metadata` 是 `HashMap`，Rust 的 `HashMap` 用
# `RandomState`（每进程重新播种）。直接 `to_string_pretty(&结构体)` 会
# **照抄 HashMap 的迭代顺序** ⇒ 同一份输入，**每个进程写出的文件都不一样**。
#
# 实测：同一输入连跑 5 次 → 5 个互不相同的 md5。
#
# ⛔ **单进程内的任何测试都抓不到它** —— 同一进程里同一个 HashMap 的迭代
#   顺序是稳定的。曾经我为此写了 3 条单测，变异验证时**全部照样绿**
#   ⇒ 零区分力。唯一的判据是「起多个真进程比对字节」。
#
# ## 它还抓什么（同一个 P0 的另一半）
#
# `load_registry` 曾用 `Err(_)` 吞掉解析错误并误走「老 schema 迁移」，
# 迁移出 0 节点，而 `save_registry` 在 `run()` 末尾**无条件**执行
# ⇒ 一次写命令就把 318 节点 / 43 边覆盖成 41 节点 / 0 边，**退出码 0**。
# 本门断言节点数与边数不变，因此这一个门同时锁住「截断」与「抖动」。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="$ROOT/.neotrix/capability_registry.json"
BIN="$ROOT/target/debug/neotrix-capability"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/nt-registry-det.XXXXXX")"
RUNS=5

# ⛔ 绝不吞 stderr —— 零命中要先确认退出码 1 而非 2（见 AGENTS.md §4.2）
cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

[ -f "$SRC" ] || fail "找不到注册表 $SRC"

echo "[registry-determinism] 准备二进制…"
if [ ! -x "$BIN" ] || [ "${NT_REGISTRY_REBUILD:-0}" = "1" ]; then
  (cd "$ROOT" && cargo build -q -p nt_core_capability_tree --bin neotrix-capability) \
    || fail "构建 neotrix-capability 失败"
fi
[ -x "$BIN" ] || fail "找不到二进制 $BIN"

base_nodes="$(python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));print(len(d.get("nodes",[])))' "$SRC")"
base_edges="$(python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));print(len(d.get("edges",[])))' "$SRC")"
echo "[registry-determinism] 基准：$base_nodes 节点 / $base_edges 边；跑 $RUNS 次独立进程"

# 每次都从同一份基准复制 ⇒ 输入完全相同，输出必须逐字节相同。
# ⚠️ 写盘目标全在 $WORK 里：overlay 路径 = registry 同目录 ⇒ 也不会碰仓库。
first_md5=""
for i in $(seq 1 "$RUNS"); do
  cp "$SRC" "$WORK/r$i.json"
  # `get` 是会触发 save_registry 的只读子命令（run() 末尾无条件保存）。
  "$BIN" --registry "$WORK/r$i.json" get "NT-MEMORY::trade::trade_product_spec" \
    >"$WORK/log$i.txt" 2>&1 || true
  if grep -q 'skip edge' "$WORK/log$i.txt"; then
    echo "[registry-determinism] 注意：第 $i 次报了 skip edge（边端点缺失）"
  fi
  n="$(python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));print(len(d.get("nodes",[])))' "$WORK/r$i.json")" \
    || fail "第 $i 次输出不是合法 JSON（写盘被破坏）"
  e="$(python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));print(len(d.get("edges",[])))' "$WORK/r$i.json")"
  [ "$n" = "$base_nodes" ] || fail "第 $i 次节点数 $n ≠ 基准 $base_nodes ⇒ 往返截断（P0 数据丢失）"
  [ "$e" = "$base_edges" ] || fail "第 $i 次边数 $e ≠ 基准 $base_edges ⇒ 往返截断（P0 数据丢失）"
  m="$(md5 -q "$WORK/r$i.json")"
  if [ -z "$first_md5" ]; then
    first_md5="$m"
  elif [ "$m" != "$first_md5" ]; then
    fail "第 $i 次输出与第 1 次字节不同（$m ≠ ${first_md5}）⇒ 序列化不确定（HashMap 迭代序泄漏）"
  fi
done

echo "[registry-determinism] PASS：$RUNS 次独立进程输出逐字节一致（${first_md5}），且 $base_nodes 节点 / $base_edges 边零丢失"