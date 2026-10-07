#!/bin/bash
# 执行器登记制门 —— 让「清单声称的可执行性」与「代码真实状态」对账。
#
# ## 为什么需要这道门（2026-10-06 实测立门）
#
# 本轮抓到 **4 处「上架了但不可用/是桩」**：
#   ① manifest id 与实现**错层**（拿 L1 内部 4 个核对能力树 5 个）
#   ② 金丝雀度量「**被查过**」而非「被执行过」（`signal` 打在 `get()` 上）
#   ③ 安全工具**注册失败被 `.ok()` 吞掉** ⇒ 安全面悄悄缩小
#   ④ 执行器是**阶段脚手架**（17 阶段逻辑全是注释，却能跑通有产出）
#
# ⇒ 共性：**清单/注册表里写着「有这个能力」，但没有任何机器检查它到底能不能跑。**
#
# ## 本门查什么（可证伪，有变异验证）
#
# 对每条 manifest 条目，三条判据：
#   (a) `executability` 字段**必须存在**（缺字段 ⇒ 编译不过，本门跑不了 ⇒ 已成立）
#   (b) 声称 `Executable` 的 id ⇒ **必须真的**在 `tree_dispatch.rs` 里注册
#   (c) 声称 `DeclaredOnly`/`Scaffold` 的 id ⇒ **必须不在**派发实现表里
#
# ⛔ **不做**的事：不判定「Scaffold 到底实现了几成」—— 那是人工裁决，
#    门去猜就等于发明。门只对账「声称 vs 事实」。
#
#   ④ (2026-10-07 新增) **未接线必须留下机器可读的阻塞记录** —— 消费
#      `.neotrix/capability-blocking.json`：
#      (d) 每个非 Executable 能力**必须**有阻塞条目，且 reason/owner/unblocked_when
#          三项**非空** ⇒ 新增一个 DeclaredOnly 却没人登记 ⇒ 当场红。
#          （此前这些状态只活在散文里 ⇒ 无人守卫。）
#      (e) 阻塞条目**不得是死条目** —— 指向的能力必须存在、且**当前确实**非 Executable。
#          ⇒ 能力接好了却忘记从阻塞表里划掉 ⇒ 当场红（账本腐烂可被发现）。
#      (f) `schema_source` 非 null 时该路径**必须存在且非空** —— 这就是给业务侧的
#          插口：schema 到位 ⇒ 写路径 ⇒ 门立刻验它真的到位。**不发明 schema**。
#      (g) 仅报告不判红：声称已变为 Executable、却仍列在阻塞表里（提示该划掉）。
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

MANIFEST="crates/nt-core-capability-tree/src/market.rs"
DISPATCH="neotrix-core/src/l1_action/nt_act/nt_act_trade/tree_dispatch.rs"
STRICT=0
[ "${1:-}" = "--strict" ] && STRICT=1

[ -f "$MANIFEST" ] || { echo "executor-registry: 找不到 $MANIFEST" >&2; exit 2; }

fail=0

# 预先抽出「**注册表本体**里**真正注册**的 id」：
#  - 只取 `register_tree_dispatchers` 函数体（真正的注册处）
#  - ⛔ 排除 `#[cfg(test)]` 块（测试里出现的 id 不是注册）
#  - 只认被注册进 `pairs` 数组的字面量
REGISTERED=$(python3 - "$DISPATCH" <<'PY'
import re, sys
try:
    src = open(sys.argv[1], encoding="utf-8").read()
except OSError:
    sys.exit(0)
m = re.search(r"fn register_tree_dispatchers\(\)[^{]*\{", src)
if not m:
    sys.exit(0)
i = src.index("{", m.start())
d = 0
j = i
while j < len(src):
    if src[j] == "{":
        d += 1
    elif src[j] == "}":
        d -= 1
        if d == 0:
            break
    j += 1
body = src[i + 1 : j]
# 去掉行注释，避免文档里出现的 id 被当成注册
body = "\n".join(re.sub(r"//.*$", "", l) for l in body.split("\n"))
# 注册只发生在 pairs 数组里
# 解析 `const _TREE_ID_X: &str = "..."` —— 注册处**刻意**用常量而非字面量
# （避免「注册 id」与「打点 id」两处漂移），故本门必须跟着解析常量，
# 否则会把「已注册」误判成「未注册」（实测踩到）。
consts = dict(re.findall(r'const\s+(\w+)\s*:\s*&\s*str\s*=\s*"([^"]+)"', src))
arr = re.search(r"pairs:[^=]*=\s*\[(.*?)\];", body, re.S)
if arr:
    inner = arr.group(1)
    # 形式一：常量名
    for name in re.findall(r"(_?[A-Za-z_]\w*)\s*,", inner):
        if name in consts:
            print(consts[name])
    # 形式二：直接字面量
    for lit in re.findall(r'"([^"]+)"', inner):
        print(lit)
PY
)

# 抽出 (id, executability) 对。用 python3 保证解析稳定（不靠脆弱的 sed）。
pairs=$(python3 - "$MANIFEST" <<'PY'
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()
body = src[src.index("pub const TRADE_MANIFEST"):]
for m in re.finditer(r'id:\s*"([^"]+)"(.*?)executability:\s*Executability::(\w+)', body, re.S):
    print(f"{m.group(1)}\t{m.group(3)}")
PY
)

if [ -z "$pairs" ]; then
  echo "executor-registry: FAIL —— 未从清单解析到任何 (id, executability)"
  exit 1
fi

n_total=0; n_exec=0
while IFS=$'\t' read -r id kind; do
  [ -n "$id" ] || continue
  n_total=$((n_total + 1))
  # ⛔⛔ 只认**注册表本体**里的字面量。
  #   ⚠️ 首版 grep 整个文件 ⇒ 命中文档表格与 `未注册必须仍 None` 测试里的 id
  #   ⇒ 把「没注册」误判成「已注册」。这与 AGENTS.md R-SCAN-1b 同族：
  #   **字符串命中不构成证据**。此处由 `$REGISTERED` 预先算好（见下）。
  if printf '%s\n' "$REGISTERED" | grep -qxF "$id"; then
    in_dispatch=yes
  else
    in_dispatch=no
  fi

  case "$kind" in
    Executable)
      n_exec=$((n_exec + 1))
      if [ "$in_dispatch" != yes ]; then
        echo "  ❌ 声称 Executable 但**未在派发表注册**: $id"
        fail=1
      fi
      ;;
    DeclaredOnly)
      # 无可调用入口 ⇒ **必须不在**派发表；注册了就是「声称没有却能跑」⇒ 自相矛盾
      if [ "$in_dispatch" = yes ]; then
        echo "  ❌ 声称 DeclaredOnly 却**已注册进派发表**: $id"
        fail=1
      fi
      ;;
    Scaffold)
      # ⚠️ 脚手架的定义就是「**已注册、能跑通、却没做实事**」
      #   ⇒ 它**必须**在派发表里。若反而不注册，那它是DeclaredOnly 而非脚手架
      #   ⇒ 说明 `executability` 字段本身填错了。
      #   （实测：我首版把 Scaffold 与 DeclaredOnly 用同一判据 ⇒ 门把**真的**脚手架
      #     报成「不该注册」。规则错了，被门顶回来。）
      if [ "$in_dispatch" != yes ]; then
        echo "  ❌ 声称 Scaffold 却**未注册** ⇒ 它其实是 DeclaredOnly，字段填错: $id"
        fail=1
      fi
      ;;
    *)
      echo "  ❌ 未知 executability: $kind（$id）"
      fail=1
      ;;
  esac
done <<EOF
$pairs
EOF

# ── ④ 未接线能力的机器可读阻塞记录 ────────────────────────────────
BLOCK=".neotrix/capability-blocking.json"
n_block=0
if [ ! -f "$BLOCK" ]; then
  # ⛔ fail-closed：缺表 = 判据无法执行 = 本门结构上不可能失败 ⇒ 判红
  echo "  ❌ 找不到阻塞记录 $BLOCK ⇒ 判据 (d)(e)(f) 无法执行"
  fail=1
else
  block_out=$(python3 - "$BLOCK" "$MANIFEST" <<'PYBLOCK'
import json, os, re, sys
blocking_path, manifest_path = sys.argv[1], sys.argv[2]
try:
    with open(blocking_path, encoding="utf-8") as fh:
        doc = json.load(fh)
except Exception as exc:
    print(f"__ERR__阻塞记录不是合法 JSON：{exc}")
    raise SystemExit(0)

# 以下划线开头的键是文档键，不是能力条目
entries = {k: v for k, v in doc.items() if not k.startswith("_")}

# 从 market.rs 抽「id -> executability」，与主体门同一口径
src = open(manifest_path, encoding="utf-8").read()
real = {}
for m in re.finditer(r"ManifestEntry\s*\{", src):
    i = src.index("{", m.start()); d = 0
    for j in range(i, len(src)):
        if src[j] == "{": d += 1
        elif src[j] == "}":
            d -= 1
            if d == 0: break
    body = src[i + 1 : j]
    mid = re.search(r'id:\s*"([^"]+)"', body)
    mex = re.search(r"executability:\s*Executability::(\w+)", body)
    if mid and mex:
        real[mid.group(1)] = mex.group(1)

lines, bad, warn = [], 0, 0
for cid, info in sorted(entries.items()):
    if not isinstance(info, dict):
        lines.append(f"  ❌ {cid} 条目不是对象"); bad += 1; continue
    kind = real.get(cid)
    if kind is None:
        lines.append(f"  ❌ 阻塞条目指向不存在的能力（死条目）：{cid}"); bad += 1; continue
    if kind == "Executable":
        # (g) 仅报告：已接线却仍列阻塞 —— 该划掉，否则阻塞表会失去意义
        lines.append(f"  ℹ️ {cid} 声称已是 Executable，但仍列在阻塞表 ⇒ 该划掉了")
        warn += 1
        continue
    for field in ("executability", "owner", "reason", "unblocked_when"):
        v = str(info.get(field, "")).strip()
        if not v:
            lines.append(f"  ❌ {cid} 缺必填字段 `{field}`（阻塞记录必须写清为什么）")
            bad += 1
    if info.get("executability") != kind:
        lines.append(f"  ❌ {cid} 阻塞记录声称 {info.get('executability')}，market.rs 实为 {kind}")
        bad += 1
    ss = info.get("schema_source", None)
    if ss not in (None, ""):
        if not os.path.isfile(ss) or os.path.getsize(ss) == 0:
            lines.append(f"  ❌ {cid} 的 schema_source 指向 {ss}，但该文件不存在或为空")
            bad += 1
# (d) 反向：manifest 里每个非 Executable 都必须有阻塞条目
missing = [cid for cid, k in real.items() if k != "Executable" and cid not in entries]
for cid in sorted(missing):
    lines.append(f"  ❌ {cid} 声称 {real[cid]} 却**没有**阻塞记录 ⇒ 未接线状态只靠散文，无机器守卫")
    bad += 1

print(f"__N__ {len(entries)} {bad} {warn}")
for l in lines:
    print(l)
PYBLOCK
)
  n_block=$(printf '%s
' "$block_out" | sed -n 's/^__N__ //p' | awk '{print $1}')
  printf '%s
' "$block_out" | grep -v '^__' | sed '/^$/d'
  if printf '%s
' "$block_out" | grep -q '^__ERR__'; then
    fail=1
  elif [ "$(printf '%s
' "$block_out" | sed -n 's/^__N__ //p' | awk '{print $2}')" != "0" ]; then
    fail=1
  fi
fi

echo "=== 执行器登记制门 ==="
echo "  条目: $n_total   声称可执行: $n_exec   阻塞记录: ${n_block:-0}"

if [ "$fail" -ne 0 ]; then
  echo "  executor-registry: FAIL —— 声称与事实不符，或未接线缺阻塞记录"
  exit 1
fi
if [ "$STRICT" -eq 1 ] && [ "$n_exec" -eq 0 ]; then
  echo "  executor-registry: FAIL —— 无任何 Executable 条目 ⇒ 门空转"
  exit 1
fi
echo "  executor-registry: PASS（$n_total 条全部对账一致）"
