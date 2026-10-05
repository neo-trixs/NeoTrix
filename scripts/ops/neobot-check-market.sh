# ⭐⭐⭐⭐⭐ 能力市场清单自洽门（2026-10-04）
#
# ⭐⭐⭐⭐⭐ **它防的是什么**：市场 `listable()` **只返回可上架的**，
# ⭐⭐⭐⭐⭐ 没填 `market.*` 的能力**从视图里彻底消失** ⇒「不可见」又回来了，
# ⭐⭐⭐⭐⭐ 与本轮一路治的「有能力没接线」完全同型。
#
# ⭐⭐⭐⭐⭐ **⭐⭐ 真源纪律**：⭐⭐ ⭐⭐ 「已上架」⭐⭐ 必须读 **市场自己报的清单**
# ⭐⭐⭐⭐⭐ （`market_listable`），⭐⭐ ⭐⭐ 不能拿 canary 当地图 ⭐⭐ ——
# ⭐⭐⭐⭐⭐ canary 是**监视集合**不是**市场集合**，⭐⭐ 两者混用会把分母算成
# ⭐⭐⭐⭐⭐ 「两个不同集合的和」⇒ 报出 `5/10` 这种**无意义的数**。
# ⭐⭐⭐⭐⭐ ⇒ ⭐⭐ 判据统一走**运行期探针**，⭐⭐ 与 `neobot-check-emergence.mjs` 同源。
#
# ⭐⭐⭐⭐⭐ **报告 vs 阻断**（照抄 `nt_gate_coverage.py` 教训）：
# ⭐⭐⭐⭐⭐ ⭐⭐ 「上架率低」是**演进指标** ⭐⭐ 当阻断 ⇒ 恒红 ⇒ 门被关 ⇒ 更坏；
# ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ ⭐⭐ 但「清单**不自洽**」是**缺陷** ⭐⭐ ⇒ 判红：
# ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ ⭐⭐ ① 互斥：listable ∩ unlisted = ∅
# ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ ⭐⭐ ② 完备：listable ∪ unlisted = 全部注册能力
# ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ ⭐⭐ ③ 反向一致：金丝雀主张接线的，能力树里必须有
# ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ ⭐⭐⭐⭐⭐ ⭐⭐ 拿不到证据 ⭐⭐ 也判红（⭐⭐ 不把「没跑出来」当「没问题」）
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "能力市场清单门（⭐⭐ 报告式 ⭐⭐ 演进指标不当阻断）"
echo

probe=$(timeout 1800 cargo test -p neotrix --lib 运行期探针 -- --nocapture 2>&1 \
        | grep -oE '^EMERGENCE_PROBE \{.*\}$' | head -1)
if [ -z "$probe" ]; then
  # ⭐⭐⭐⭐⭐ ⛔ 拿不到证据 ⇒ ⭐⭐ **判红**（⭐⭐ 不静默放过）
  echo "⛔ 运行期探针跑不出结果 ⇒ ⭐⭐⭐⭐ 无法核对市场清单"
  echo "   ⭐⭐ ⛔ **不以「拿不到证据」当「没问题」**"
  exit 1
fi
echo "${probe#EMERGENCE_PROBE }" > /tmp/nt-market-probe.json

python3 - <<'PY'
import json, sys
d = json.load(open('/tmp/nt-market-probe.json'))

# ⭐⭐ 真源：**市场自己报的清单**（⭐⭐ 不是 canary，⭐⭐ canary 是监视集合不是市场集合）
listed = d.get('market_listable') or []
un     = d.get('market_unlisted') or []
orphan = d.get('canary_expected_but_unregistered') or []
canary = d.get('canary') or []
total  = d.get('total')

print(f'  ⭐⭐ 能力节点总数（运行期实测）：{total}')
print(f'  ⭐⭐ 市场上架：{len(listed)}　⭐⭐ 未上架：{len(un)}　⭐⭐ 金丝雀监视：{len(canary)}')
print()
print('  ── 已上架（可上架，license/version/category 齐备）──')
for i in listed:
    print(f'    ✅ {i}')
if not listed:
    print('    （无）')
print()
print('  ── 尚未上架（⭐⭐ 静默消失的那批）──')
for i in un:
    print(f'    ⛔ 未上架 {i}')
if not un:
    print('    （无）')
print()

fail = []

# ⭐⭐ 不变量 1：listable 与 unlisted **互斥**（⭐⭐ 同一个能力不能既上架又被拦）
both = sorted(set(listed) & set(un))
if both:
    fail.append(f'listable 与 unlisted 交集非空：{both}')

# ⭐⭐ 不变量 2：**完备性**（⭐⭐ 两个集合必须无缝拼成全集）
# ⭐⭐ listable ∪ unlisted == 全部注册能力
# ⭐⭐ ⭐⭐ 第一版就是把 canary 当地图 ⇒ 分母混了不同集合 ⇒ 5/10 这种无意义的数
if total is not None and (len(listed) + len(un)) != total:
    fail.append(
        f'清单不完备：上架 {len(listed)} + 未上架 {len(un)} = '
        f'{len(listed)+len(un)} ≠ 实测总数 {total}'
    )

# ⭐⭐ 不变量 3：反向一致（金丝雀主张接了线，能力树里却没有）
if orphan:
    fail.append(f'金丝雀主张但能力树没有：{orphan}')

listed_n, un_n = len(listed), len(un)
den = total if isinstance(total, int) and total > 0 else listed_n + un_n
pct = (listed_n * 100 // den) if den else 0
print(f'  ── 上架率：{listed_n}/{den} = {pct}%（⭐⭐ 演进指标，⭐⭐ ⛔ 不判红）──')

if fail:
    for f in fail:
        print(f'\n  ⛔ {f}')
    print('\n  ⛔⭐⭐⭐ 市场清单不自洽 ⇒ 判红（⭐⭐ 报告式指标不当阻断，⭐⭐ 不自洽当阻断）')
    sys.exit(1)
print('\n  ✅ 市场清单自洽（⭐⭐ 互斥 + 完备 + 反向一致）')
PY
