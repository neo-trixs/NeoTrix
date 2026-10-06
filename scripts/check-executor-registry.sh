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

echo "=== 执行器登记制门 ==="
echo "  条目: $n_total   声称可执行: $n_exec"

if [ "$fail" -ne 0 ]; then
  echo "  executor-registry: FAIL —— 声称与事实不符"
  exit 1
fi
if [ "$STRICT" -eq 1 ] && [ "$n_exec" -eq 0 ]; then
  echo "  executor-registry: FAIL —— 无任何 Executable 条目 ⇒ 门空转"
  exit 1
fi
echo "  executor-registry: PASS（$n_total 条全部对账一致）"
