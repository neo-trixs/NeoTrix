#!/bin/sh
# nt_smoke.sh — NeoBot 桌面壳的**单命令验收门**。
#
# ═══════════════════════════════════════════════════════════════════════
# 它做什么（每步都单独计时、单独记账，最后打一张表）
# ═══════════════════════════════════════════════════════════════════════
#   0. 内存门（nt_mem_gate.sh）—— 只**报告**并**尊重**，绝不绕过
#   1. 核心测试  cargo test -p neotrix-neobot --lib
#   2. 壳测试    cargo test -p neobot-desktop          （新增：IPC 参数绑定层）
#   3. 前端类型  npm run typecheck
#   4. 前端自测  npm run selftest
#   5. 编译检查  cargo check -p neobot-desktop --all-targets
#   6. IPC 键名  python3 scripts/ops/nt_ipc_keys.py（前端 invoke 键 ↔ Rust 形参）
#
# ═══════════════════════════════════════════════════════════════════════
# 它**刻意不覆盖**什么（别拿「脚本绿了」当「App 没问题」的证据）
# ═══════════════════════════════════════════════════════════════════════
# - **不启动 Tauri 运行时**。没有真窗口、没有 webview、没有 `invoke` 的
#   序列化往返。参数**绑定**被验了；参数**跨进程 JSON 编解码**没被验
#   （见下方「已知缺口」）。
# - **不测需要网络的命令**：`neobot_channel_probe`（探活必然连公网）、
#   `neobot_core_*`（晶体核心）、`neobot_models/providers`（有网时才有意义）。
#   本脚本全程离线。
# - **不跑真引擎**：`neobot_run` 会起一轮真对话；冒烟里用
#   `NEOBOT_ENGINE=echo` 钉死回显引擎，绝不碰 HTTP/CLI/opencode。
# - **不测 Git 那组**（`neobot_git_*`）：它们要真 git 仓库 + 提交历史，
#   且 `neobot_git_commit` 会执行仓库里的 `.git/hooks/*`（真实代码执行）。
#   在冒烟里跑它等于「测试时执行任意仓库代码」，不进这道门。
# - **不测前端渲染**：没有 Playwright/Puppeteer（本项目没有浏览器测试设施，
#   引入属于超范围）。typecheck 只保证类型对，不保证 DOM 里有那个元素。
# - **不测 IPC 通道本身**：`tauri::ipc::Channel` 流式事件
#   （`neobot_run_stream`）需要真实 AppHandle 才造得出来。
#
# ═══════════════════════════════════════════════════════════════════════
# 已知缺口（源级复核，非执行覆盖）—— 2026-09-28
# ═══════════════════════════════════════════════════════════════════════
# ~~前端 invoke 键名 vs Rust 形参名没人守~~ → **已关闭（2026-09-28，接入第 6 步）**。
# 曾经这条写着「本脚本验不了，只能靠人读」—— 现在由 `scripts/ops/nt_ipc_keys.py`
# 静态核对并作为第 6 步进门：97 声明 / 97 注册 / 0 键名错配。
#
# 仍**不覆盖**的两点，别把「第 6 步绿了」当成端到端通了：
# - **只到键名，不到类型**：`{ taskId: 123 }` 键名对但类型错，静态核对**不报**
#   （要报就得做类型推导，超出静态范围）。
# - **只到静态，不到运行时**：Tauri 的 serde 转换、camelCase 边界、
#   `Option` 缺省行为仍要真进程往返才验得出。第 6 步绿 = 名字对上了，不是「传对了」。
#
# 更彻底的正解（仍待做）是把 invoke 收进 typed wrapper
# （`invoke<T>(name, args)` + 命令名字面量联合类型），让键名错在**编译期**。
#
# ═══════════════════════════════════════════════════════════════════════
# 用法
# ═══════════════════════════════════════════════════════════════════════
#   sh scripts/ops/nt_smoke.sh              # 内存门 BLOCKED 则只报告并退出 2
#   FORCE_SMOKE=1 sh scripts/ops/nt_smoke.sh   # 明知内存紧张仍要跑（自己负责）
#
# 退出码：0 全绿；1 有步骤失败；2 内存门 BLOCKED 且未 FORCE；9 环境不对。

set -u

ROOT=$(cd "$(dirname "$0")/../.." && pwd) || exit 9
cd "$ROOT" || exit 9

APP="$ROOT/apps/neobot-desktop"
FRONTEND="$APP/frontend"

# 16G 机器 + 多窗口并行是事故根因（AGENTS.md 2026-09-21）。
# jobs=2 是硬要求，不是保守——单窗口也能提到 8，但别在别人在编译时提。
CARGO_BUILD_JOBS=2
export CARGO_BUILD_JOBS

# 一次跑两个 cargo 会自己跟自己抢锁，串行。
CARGO_SERIAL=1

# ─── 临时目录：跑之前建，跑完按结果决定留不留 ───
# 用 mktemp -d 而不是固定名：并发跑两次不会互删对方的库。
# trap 挂在 EXIT 上：**失败路径也清数据目录**，不留半份 SQLite 残骸
# （残骸的代价不只是占地方：下次跑会读到旧数据，测试变成假绿）。
SMOKE_TMP=""
# 失败时把日志**搬**到仓库里保留。为什么要搬而不是就地留着：
# 日志在 mktemp 目录里，跑完没人找得到；而验收失败时唯一有价值的东西
# 就是那份日志。第一版脚本在这里犯过一个明确的错：汇总结尾打印
# 「完整日志：<tmp 路径>」，紧接着 trap 把那个目录删了 —— 打印了一个
# 指向不存在文件的指引。故：失败留、顺手删的只有数据目录。
KEEP_LOGS=""
cleanup() {
  if [ -n "$KEEP_LOGS" ] && [ -d "$LOGDIR" ]; then
    dest="$ROOT/target/nt-smoke-logs"
    mkdir -p "$dest"
    cp -R "$LOGDIR/." "$dest/" 2>/dev/null || true
    echo "[smoke] 失败：日志已保留到 $dest"
  fi
  if [ -n "$SMOKE_TMP" ] && [ -d "$SMOKE_TMP" ]; then
    rm -rf "$SMOKE_TMP"
    echo "[smoke] 已清理临时目录 $SMOKE_TMP"
  fi
  # 测试自身在 target/nt-smoke/ 下的那份也一并清掉（common/mod.rs 的兜底目录）。
  rm -rf "$ROOT/target/nt-smoke" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

SMOKE_TMP=$(mktemp -d "${TMPDIR:-/tmp}/neobot-smoke.XXXXXX") || exit 9
# 必须以 `.neobot` 结尾：壳的 shape 断言（commands.rs::data_dir_is_absolute）
# 钉死 `dir.ends_with(".neobot")`，命名为 `data` 会让该单测在冒烟下误红。
NEOBOT_DATA_DIR="$SMOKE_TMP/.neobot"
NEOBOT_ENGINE=echo
export NEOBOT_DATA_DIR NEOBOT_ENGINE
mkdir -p "$NEOBOT_DATA_DIR"
LOGDIR="$SMOKE_TMP/logs"
mkdir -p "$LOGDIR"
echo "[smoke] 临时数据目录 $NEOBOT_DATA_DIR"

# ─── 记账 ───
# 三份**并行的临时文件**（名字/状态/秒），而不是三个 shell 变量。
# 变量方案要靠 `sed -n` 做行号换算，那玩意儿错一次就是静默错位的汇总表 ——
# 一张不可信的汇总比没有汇总更坏：它会让人以为每步都跑过了。
# 记账文件放**临时目录**而不是 `target/`：脚本退出时 trap 会把整个
# `SMOKE_TMP` 删掉，记账数据随之消失——不给「上次跑的结果」留一份
# 会被人误当成「本次结果」的半成品。
SUMMARY_DIR="$SMOKE_TMP/summary"
mkdir -p "$SUMMARY_DIR"
NAMES_FILE="$SUMMARY_DIR/names"
STATES_FILE="$SUMMARY_DIR/states"
SECS_FILE="$SUMMARY_DIR/secs"
: >"$NAMES_FILE"; : >"$STATES_FILE"; : >"$SECS_FILE"
FAILED=0

record() { # name state secs
  printf '%s\n' "$1" >>"$NAMES_FILE"
  printf '%s\n' "$2" >>"$STATES_FILE"
  printf '%s\n' "$3" >>"$SECS_FILE"
  [ "$2" = "FAIL" ] && FAILED=1
  return 0
}

# 结尾那张表。为什么要单独一个函数：中途 BLOCKED 与跑完全程
# 两条路径都要打这张表，复制两份的脚本必然有一天只改了一份。
summarise() {
  echo "══════════════════════════════════════════════════════════"
  echo "[smoke] 汇总"
  paste "$NAMES_FILE" "$STATES_FILE" "$SECS_FILE" | while IFS='	' read -r n s t; do
    case "$s" in
      OK)   mark="✅ OK" ;;
      FAIL) mark="❌ FAIL" ;;
      WARN) mark="⚠️  WARN" ;;
      *)    mark="⏭️  $s" ;;
    esac
    printf '  %-42s %-8s %ss\n' "$n" "$mark" "$t"
  done
  echo "  ─────────────────────────────────────────────────────"
  if [ -n "$KEEP_LOGS" ]; then
    echo "  完整日志：$ROOT/target/nt-smoke-logs （失败保留，target/ 下，cargo clean 可清）"
  else
    echo "  完整日志：随临时目录一并清理（本次全过，无需留档）"
  fi
  if [ "$FAILED" -ne 0 ]; then
    echo "  结果：❌ 有步骤失败"
  elif grep -q BLOCKED "$STATES_FILE" 2>/dev/null; then
    # 内存门 BLOCKED 时**一步都没跑**，此时打「全绿」是撒谎：
    # 第一版脚本就在这里犯过 —— 退出码 2，表格却写「✅ 全绿」。
    # 读表的人不会去看退出码，只看那三个字。故此处必须显式区分「没跑」与「跑过了」。
    echo "  结果：⏸️  未执行（内存门 BLOCKED，什么都没跑 —— 不构成任何通过证据）"
  else
    echo "  结果：✅ 全绿"
  fi
  echo "  提醒：绿灯只覆盖本脚本头注释「它做什么」那一节列出的部分，"
  echo "        第 6 步只验 invoke **键名**静态一致，不代表跨进程序列化与类型正确（见头注释）。"
  echo "══════════════════════════════════════════════════════════"
}

# ─── 内存门：报告 + 尊重 ───
GATE="sh $ROOT/scripts/ops/nt_mem_gate.sh"
echo "──────────────────────────────────────────────────────────────"
echo "[smoke] 0/5 内存门"
$GATE
gate_rc=$?
if [ "$gate_rc" -ne 0 ]; then
  if [ "${FORCE_SMOKE:-0}" = "1" ]; then
    echo "[smoke] 内存门 BLOCKED，但 FORCE_SMOKE=1 —— 明知故犯，继续。"
    record "0 内存门" "WARN" 0
  else
    echo "[smoke] 内存门 BLOCKED：先停重活（sidecar/晶体/App 按序），或显式 FORCE_SMOKE=1。"
    record "0 内存门" "BLOCKED" 0
    summarise
    exit 2
  fi
else
  record "0 内存门" "OK" 0
fi

run_step() { # name logfile command...
  name="$1"; log="$2"; shift 2
  echo "──────────────────────────────────────────────────────────────"
  echo "[smoke] $name"
  start=$(date +%s)
  if "$@" >"$log" 2>&1; then
    end=$(date +%s)
    echo "[smoke]   OK ($((end - start))s) 日志 $log"
    record "$name" "OK" "$((end - start))"
  else
    rc=$?
    end=$(date +%s)
    echo "[smoke]   FAIL (rc=$rc, $((end - start))s) —— 尾部日志："
    tail -40 "$log" | sed 's/^/         | /'
    KEEP_LOGS=1
    record "$name" "FAIL" "$((end - start))"
  fi
}

# ─── 1. 核心测试 ───
# 为什么还要跑它：壳的测试断言的是「壳接对了」，而「核心是对的」是它的前提。
# 只跑壳不跑核心，等于在可能已经歪了的假设上盖房。
run_step "1 核心测试 (neotrix-neobot)" "$LOGDIR/01-core.log" \
  sh -c 'cargo test -p neotrix-neobot --lib'

# ─── 2. 壳测试（本次新增的那层）───
run_step "2 壳测试 (neobot-desktop)" "$LOGDIR/02-shell.log" \
  sh -c 'cargo test -p neobot-desktop'

# ─── 3+4. 前端 ───
# 2026-10-07 拍板：桌面壳/前端真源已迁 `~/Downloads/Neo/neobot`，本仓 smoke
# 不再自持前端门 —— 改为在 Neo/neobot 在位时委托其路径跑同两步，否则 SKIP。
NEOBOT_REPO="$HOME/Downloads/Neo/neobot"
if [ -d "$NEOBOT_REPO/apps/neobot-desktop/frontend/node_modules" ]; then
  run_step "3 前端类型 (typecheck @ Neo/neobot)" "$LOGDIR/03-fe-type.log" \
    sh -c "cd '$NEOBOT_REPO/apps/neobot-desktop/frontend' && npm run --silent typecheck"
  run_step "4 前端自测 (selftest @ Neo/neobot)" "$LOGDIR/04-fe-selftest.log" \
    sh -c "cd '$NEOBOT_REPO/apps/neobot-desktop/frontend' && npm run --silent selftest"
else
  echo "[smoke] Neo/neobot 前端未在位 —— 跳过前端两步"
  record "3 前端类型" "SKIP" 0
  record "4 前端自测" "SKIP" 0
fi

# ─── 5. 编译检查 ───
# 放最后：它最慢，而前面的步骤已经能说明「这批改动有没有把壳写坏」。
# --all-targets 是为了连带 check tests/（本轮新增的四个测试文件）。
run_step "5 编译检查 (neobot-desktop --all-targets)" "$LOGDIR/05-check.log" \
  sh -c 'cargo check -p neobot-desktop --all-targets'

# 6. 前后端 IPC 键名一致性。
#    放进这道门的原因：Rust 侧 38 个冒烟测试验的是「参数绑定层」（直接调函数），
#    而真实前端走 `invoke("cmd", { key: v })` —— Tauri 把 JSON 键映射到 Rust 形参名，
#    **键名对不上时编译期无感知、运行期静默失败**（参数变 None）。这是唯一的跨语言接缝，
#    曾经完全没有覆盖（脚本头「已知缺口」里明写着这条）。纯 python，不吃编译时间。
#    2026-10-07：桌面 nt_commands 已迁出本仓 ⇒ 仅当 Neo/neobot 在位时跑，否则 SKIP。
if [ -d "$HOME/Downloads/Neo/neobot/apps/neobot-desktop/src/nt_commands" ]; then
  run_step "6 IPC 键名核对 (nt_ipc_keys.py @ Neo/neobot)" "$LOGDIR/06-ipc-keys.log" \
    sh -c "python3 '$ROOT/scripts/ops/nt_ipc_keys.py' --root '$HOME/Downloads/Neo/neobot'"
else
  record "6 IPC 键名核对 (Neo/neobot 不在位, SKIP)" "SKIP" 0
fi

summarise
exit "$FAILED"
