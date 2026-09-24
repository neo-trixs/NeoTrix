#!/bin/bash
# GUIXU SHOT 审计闭环（含 OOM 重试）
# 背景：16G 机 Metal 显存压力下偶发 abort（同二进制重跑即过，属环境非代码），
# 故每场景最多试 3 次，以 PNG 落盘为准，不以字节比对为准（时钟/粒子本就逐跑不同）。
set -u
BIN="/tmp/nt-guixu-target/debug/neotrix-guixu"
[ -x "$BIN" ] || { echo "MISS $BIN — 先 cargo build -p neotrix-guixu"; exit 1; }
pass=0; fail=0
for s in title play; do
  out="/tmp/guixu_$s.png"
  ok=0
  for i in 1 2 3; do
    rm -f "$out"
    NEOTRIX_GUIXU_SHOT="$s" NEOTRIX_GUIXU_SHOT_OUT="$out" "$BIN" >"/tmp/guixu_$s.log" 2>&1
    if [ -f "$out" ]; then ok=1; break; fi
    sleep 2
  done
  if [ "$ok" -eq 1 ]; then
    echo "PASS $s $(wc -c <"$out" | tr -d ' ')B"
    pass=$((pass+1))
  else
    echo "FAIL $s :: $(tail -1 "/tmp/guixu_$s.log")"
    fail=$((fail+1))
  fi
done
echo "=== $pass pass / $fail fail ==="
[ "$fail" -eq 0 ]
