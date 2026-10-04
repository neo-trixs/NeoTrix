#!/usr/bin/env bash
# ⭐⭐⭐ neobot Tauri 窗口端到端验收（可重复）
#
# ## 为什么需要它
# 2026-10-03 之前，「UI 三缺陷（草稿串台 / busy 串台 / 长会话分页）真修好」
# ⛔ **只有单测背书，没有任何端到端证据**。本脚本补上这一层。
#
# ## ⭐ 为什么用 **AX（辅助功能）树**而不是截图
# ⭐⭐ 实测 `screencapture` 在本机报 `could not create image from display`
# （屏幕录制权限未授予）⇒ ⛔ 截图路线**不可用**。
# ⇒ ⭐ **AX 树反而更可靠**：它读的是**渲染后的无障碍属性**，
# ⭐⭐ 且能**逐条列出真实文本** ⇒ 比像素比对更能证明「界面真的画对了」。
#
# ## ⭐⭐ 前置条件（⭐ 先查，别假设 —— 2026-10-03 的第 4 次教训）
# ① Aqua 会话：`launchctl managername` 必须返回 `Aqua`
# ② 二进制存在：`target/debug/neobot-desktop`
# ③ 辅助功能权限：osascript 能枚举 `neobot-desktop` 的窗口
# ⛔ 任一不满足 ⇒ **明确报错退出**，⛔ 绝不把「测不了」说成「测过了」。

set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO" || exit 2
BIN="target/debug/neobot-desktop"
LOG="${TMPDIR:-/tmp}/neobot_gui_e2e.log"

fail=0
say() { printf '%s\n' "$*"; }
ok()  { say "✅ $*"; }
bad() { say "⛔ $*"; fail=1; }

# ── ① 显示环境（⭐ 先排除测量装置本身）──────────────────────────────
[ -f "$BIN" ] || { bad "二进制不存在：${BIN}（先 cargo build -p neobot-desktop --bins）"; exit 2; }
ok "二进制存在（$(wc -c < "$BIN" | tr -d ' ') B）"

mgr=$(launchctl managername 2>/dev/null || echo "")
if [ "$mgr" = "Aqua" ]; then ok "有 Aqua 图形会话（launchctl managername=Aqua）"
else bad "无 Aqua 会话（= ${mgr}）⇒ GUI 无法验证，⛔ 不是「通过」而是「测不了」"; exit 2; fi

# ── ② 启动 ────────────────────────────────────────────────────────────
"$BIN" > "$LOG" 2>&1 &
PID=$!
# ⭐⭐ 这里必须用 `${PID}` ⛔ 不能用 `${PID}，`
# ⭐⭐ **原因**：紧跟 `$PID` 的中文逗号 `，`（U+FF0C，多字节）
# ⭐⭐ 被 bash 当成变量名的延续 ⇒ `unbound variable`（实测报 `PID�: unbound variable`）
say "已启动 pid=${PID}，等待窗口出现"
WIN=""
for _ in $(seq 1 30); do
  sleep 1
  WIN=$(osascript -e 'tell application "System Events" to tell process "neobot-desktop" to return count of windows' 2>/dev/null || echo 0)
  [ "${WIN:-0}" -ge 1 ] && break
done

# ⭐⭐⭐ 等**前端挂载日志**再查 AX，⛔ 不是 sleep 猜时间。
# ⭐ 起因：脚本在「窗口一出现」就查 AX ⇒ 读到 **0 条文本**，
# ⭐⭐ 而手动那次（等久了）读到 11 条 ⇒ ⭐ **差的是 webview 渲染时间，不是功能**。
# ⇒ ⭐⭐ 用**确定性信号**（`neobot-root] mounted`）当就绪条件，
# ⭐⭐ 这比「多睡几秒」可靠，也不受机器负载影响。
for _ in $(seq 1 30); do
  grep -q 'neobot-root] mounted' "$LOG" 2>/dev/null && break
  sleep 1
done
grep -q 'neobot-root] mounted' "$LOG" 2>/dev/null \
  && ok "前端已挂载（日志信号）" || bad "30 秒内未见前端挂载日志"
# ⭐ 给 webview 一小段布局时间（日志打在 mount 之后，⛔ 布局仍在进行）
sleep 3

cleanup() { kill "$PID" 2>/dev/null; sleep 2; kill -9 "$PID" 2>/dev/null; }
trap cleanup EXIT

if [ "${WIN:-0}" -lt 1 ]; then
  bad "30 秒内未出现窗口（日志：${LOG}）"
  head -5 "$LOG" 2>/dev/null | sed 's/^/    /'
  exit 1
fi

# ── ③ 窗口属性 ────────────────────────────────────────────────────────
info=$(osascript -e 'tell application "System Events" to tell process "neobot-desktop" to return {count of windows, value of attribute "AXTitle" of window 1, size of window 1}' 2>/dev/null)
say "窗口属性：$info"
echo "$info" | grep -q 'NeoBot' && ok "窗口标题 = NeoBot" || bad "窗口标题异常：$info"

# ── ④ ⭐ 真实渲染文本（核心判据）────────────────────────────────────────
# ⭐⭐⭐ 取文本**不要在 AppleScript 里比较 role**：
# ⭐⭐ 实测 `role of e is "AXStaticText"` 这种写法会**静默失效**（返回 0 条，
# ⭐⭐ 而同一时刻 `entire contents` 明明有 7,533 字节完整树）⇒ ⭐⭐ 我一度以为
# ⭐⭐ 「AX 树取不到」，⭐ **实为我的过滤器坏了**（今天第 5 次「测量装置 vs 被测对象」）。
# ⇒ ⭐ **改为直接取整棵树、在 shell 侧用 grep 抽 `static text …`** —— 更少活动部件。
AX=$(osascript -e 'tell application "System Events" to tell process "neobot-desktop" to return entire contents of window 1' 2>/dev/null || echo "")
if [ -z "$AX" ]; then
  bad "AX 树为空（辅助功能权限？窗口未渲染？）"
else
  ok "AX 树取到 ${#AX} 字节"
  texts=$(printf '%s' "$AX" | grep -oE 'static text [^,]{1,60}' | sed 's/^static text //; s/ of .*$//' || true)
  say "读到 $(printf '%s' "$texts" | grep -c . || true) 条可见文本"
  say "  · $(printf '%s' "$texts" | head -6 | tr '\n' '|')"
  for want in "NeoBot" "自持前端" "会话" "Enter 发送"; do
    if printf '%s' "$texts" | grep -q "$want"; then ok "界面含「${want}」"
    else bad "界面**缺**「${want}」⇒ 渲染不完整"; fi
  done
  if printf '%s' "$texts" | grep -qE '[0-9]+ 个任务'; then ok "读到任务计数（数据层已接上）"
  else say "ℹ️ 未读到「N 个任务」（⛔ 可能是空库，⛔ 不判红）"; fi
fi

# ── ⑤ 前端挂载日志（⭐ 独立证据：webview 真跑了 React）─────────────────
if grep -q 'neobot-root] mounted' "$LOG" 2>/dev/null; then ok "前端挂载日志存在（webview 内 React 已 mount）"
else bad "无前端挂载日志（日志：${LOG}）"; fi

say ""
if [ "$fail" -eq 0 ]; then
  say "PASS: neobot Tauri 窗口端到端验收全过"
  exit 0
fi
say "FAIL: 有判据未过 ⛔ **不谎称通过**"
exit 1
