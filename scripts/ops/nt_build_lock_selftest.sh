#!/usr/bin/env bash
# nt_build_lock.sh 自证：把 6 类关键行为固化成可回归用例。
#
# 为什么要自证：锁类脚本的失败模式都很隐蔽（死锁、误删他人锁、竞态不闭合），
# 而这些**在正常使用中几乎不会暴露** —— 只在「有人 kill 进程」「两个窗口同时跑」
# 时才发作。没有自证 ⇒ 下一个改动很容易把它改坏而无人发现。
#
# 用法: bash scripts/ops/nt_build_lock_selftest.sh

set -uo pipefail
S="$(cd "$(dirname "$0")" && pwd)/nt_build_lock.sh"
PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); echo "  PASS $1"; }
bad()  { FAIL=$((FAIL+1)); echo "  FAIL $1 -> $2"; }

# 每个用例用独立锁目录，互不干扰（也避免污染真实锁）
L="$(mktemp -d)/lock"
export NT_BUILD_LOCK="$L"
export NT_BUILD_LOCK_TIMEOUT=0
cleanup() { rm -rf "$(dirname "$L")"; }
trap cleanup EXIT

# ① 空闲时可拿锁并执行
out="$(bash "$S" -- echo hi 2>/dev/null)"
[ "$out" = "hi" ] && ok "① 空闲时执行" || bad "① 空闲时执行" "got '$out'"

# ② 退出码透传
bash "$S" -- sh -c 'exit 42' >/dev/null 2>&1
[ $? -eq 42 ] && ok "② 退出码透传(42)" || bad "② 退出码透传" "got $?"

# ③ 执行完锁必须被释放（否则下一次会自锁）
[ -d "$L" ] && bad "③ 执行后释放锁" "锁目录仍在" || ok "③ 执行后释放锁"

# ④ 陈旧锁（持有者已死）自动回收 —— 防止「进程被 kill 后永久锁死」
mkdir -p "$L"; echo 999999 > "$L/pid"; date +%s > "$L/since"
out="$(bash "$S" -- echo reclaimed 2>/dev/null)"
[ "$out" = "reclaimed" ] && ok "④ 回收陈旧锁(死进程)" || bad "④ 回收陈旧锁" "got '$out'"
rm -rf "$L"

# ⑤ 超龄锁（持有者活着但超时）自动回收 —— 防止 pid 复用导致永久锁
mkdir -p "$L"; echo $$ > "$L/pid"; echo $(( $(date +%s) - 4000 )) > "$L/since"
out="$(bash "$S" -- echo aged 2>/dev/null)"
[ "$out" = "aged" ] && ok "⑤ 回收超龄锁" || bad "⑤ 回收超龄锁" "got '$out'"
rm -rf "$L"

# ⑥ --strict 超时 ⇒ 放弃且返回 EX_TEMPFAIL(75)；**绝不能无限等待**
mkdir -p "$L"; echo $$ > "$L/pid"; date +%s > "$L/since"
out="$(bash "$S" --strict --timeout 0 -- echo SHOULD_NOT_RUN 2>/dev/null)"; rc=$?
if [ "$rc" -eq 75 ] && [ "$out" != "SHOULD_NOT_RUN" ]; then
  ok "⑥ --strict 超时放弃(rc=75)"
else
  bad "⑥ --strict 超时放弃" "rc=$rc out='$out'"
fi
rm -rf "$L"

# ⑦ 互斥：两个并发必须串行（这是锁存在的**全部理由**）
LOG="$(dirname "$L")/log"; : > "$LOG"
# ⛔ 必须给足超时：本脚本开头把 NT_BUILD_LOCK_TIMEOUT 设为 0 以便快速测
# 「超时放弃」分支；若 ⑦ 也用 0，等待方会走**非 strict 的兜底穿透**
# （设计意图：不因锁而永久阻塞）⇒ 表现为「重叠」，那是**预期行为**，
# 不是互斥失效。我第一版正是踩了这个坑：用例设计与工具语义不匹配。
export NT_BUILD_LOCK_TIMEOUT=60
bash "$S" -- bash -c "echo \"A-start \$(date +%s)\" >> $LOG; sleep 2; echo \"A-end \$(date +%s)\" >> $LOG" >/dev/null 2>&1 &
P1=$!
sleep 0.3
bash "$S" -- bash -c "echo \"B-start \$(date +%s)\" >> $LOG; sleep 1; echo \"B-end \$(date +%s)\" >> $LOG" >/dev/null 2>&1 &
P2=$!
wait $P1 $P2
a1="$(grep '^A-end'   "$LOG" | awk '{print $2}')"
b0="$(grep '^B-start' "$LOG" | awk '{print $2}')"
if [ -n "$a1" ] && [ -n "$b0" ] && [ "$b0" -ge "$a1" ]; then
  ok "⑦ 并发互斥(A 结束后 B 才开始)"
else
  bad "⑦ 并发互斥" "A-end=$a1 B-start=$b0 （重叠 ⇒ 锁失效）"
fi

# ⑧ 非 strict 超时 ⇒ **穿透执行**（设计意图：锁绝不永久阻塞工作流）
# 与 ⑦ 对照：⑦ 给足超时 ⇒ 互斥；⑧ 超时为 0 ⇒ 穿透。两者都不是 bug。
mkdir -p "$L"; echo $$ > "$L/pid"; date +%s > "$L/since"
out="$(NT_BUILD_LOCK_TIMEOUT=0 bash "$S" -- echo PENETRATED 2>/dev/null)"
[ "$out" = "PENETRATED" ] && ok "⑧ 非strict 超时穿透(设计意图)" || bad "⑧ 超时穿透" "got '$out'"
rm -rf "$L"

echo "[nt_build_lock] selftest $((PASS)) passed / $((FAIL)) failed"
[ "$FAIL" -eq 0 ] || exit 1
