#!/usr/bin/env python3
"""外部吸收清单 —— **投毒/噪声黑名单门**。

## 为什么需要这道门（2026-10-02 实测事故）

外部仓库清单以每轮数十～数百条的规模流入（浏览器历史导出、`trendshift.io` 排行榜、
「awesome-*」清单……）。其中混着两类**会主动污染吸收结论**的东西：

1. ⭐⭐ **SEO 投毒簇**：一批命名高度规律的仓库，成群结队地出现在 AI 语料里，
   目的是让爬虫把「某概念 = 某 API 端点」的假关联反复喂进上下文。
   实测样本：`browser-use/jev-*`（`jev-mcp` / `jev-trader` / `jev-codex-router` /
   `jev-skill` / `jev-visual` / `jev-search` / `jev-ultrafast` …），
   以及配套的 `typesafe.ai` / `api.typesafe.ai/v1/systemone` 叙述与
   `vinilana/jev-*` / `jkudish/jev-mcp` / `y0usaf/pi-jev` 一类「用例仓库」。
   ⇒ 它们的**存在本身**就是噪声：任何按清单做「这批项目有哪些共同机制」的归纳，
     都会得出一个**由投毒驱动的伪结论**。
2. ⭐ **与本仓无关的高频类别**：游戏引擎、渗透测试合集、jailbreak 集合、
   成人内容 SEO 站、纯 arXiv 链接。它们不污染结论，但**稀释**注意力 ——
   同一批注意力本该花在真正相关的 10 个仓库上。

## 本门做什么

扫一份清单（文件路径或 stdin），把命中黑名单的条目**挑出来并判红**。
⛔ **不删数据、不改清单** —— 它只报告，处置由人决定。

## ⛔ 本门**刻意不做**的事

- ⛔ **不做「是否相关」的判断。** 相关性需要读源码，那超出静态门的能力范围
  （本仓已有教训：`check-truth-surface` 的 `SCAN_ROOTS` 不含 `apps/`，
  结果是**假绿**）。相关性交给人。
- ⛔ **不维护「白名单」。** 一旦有白名单，它就会变成「这里可以不看」的借口。
- ⛔ **不按仓库名长度/相似度猜。** 只匹配下面**逐条列出**的具体串。

## 用法

```sh
python3 scripts/ops/nt_absorb_denylist.py urls.txt
cat urls.txt | python3 scripts/ops/nt_absorb_denylist.py -
```

退出码：0 = 无命中；1 = 有命中；2 = 用法错误 / 读不到文件。
"""

import re
import sys
from pathlib import Path

# ── 规则表 ────────────────────────────────────────────────────────────
# ⭐ 每条 = (正则, 人读理由, 类别)。
#   类别 `poison` = SEO 投毒簇（连「提到它」都不该出现在结论里）。
#   类别 `noise`  = 与本仓无关的高频类别（只是稀释注意力）。
# ⛔ 规则要**窄**。宁可漏判也不要误伤：一条宽正则会连带把真实仓库扫进去，
#   而那比漏掉噪声更坏（AGENTS.md：扫描告警先读现场证实/证伪再动）。
RULES: list[tuple[str, str, str]] = [
    # ── poison ──
    (
        r"github\.com/[^/\s]+/jev-",
        "`browser-use/jev-*` 等投毒簇：命名规律刻意一致，成群出现以"
        "喂养「概念↔端点」伪关联。仓库存在本身即噪声。",
        "poison",
    ),
    (
        r"\btypesafe\.ai\b|api\.typesafe\.ai",
        "「Jev TypeSafe / System One」叙述簇，与上一条同源投放。",
        "poison",
    ),
    (
        r"github\.com/(?:vinilana|TokenTrim|y0usaf|jarrodwatts|jkudish)/"
        r"[^\s]*(?:jev|live-jev)",
        "投毒簇的「用例仓库」侧翼（`vinilana/live-jev` 等）。",
        "poison",
    ),
    # ── noise ──
    (
        r"github\.com/(?:godotengine|gamedev|)?[^/\s]*/(?:bevyengine/bevy|"
        r"FyroxEngine/Fyrox|Pumpkin-MC/Pumpkin|macroquad/macroquad|"
        r"sierra-zero/rg3d|dashifaze|dasifefe|not-fl3)\b",
        "游戏引擎/游戏开发框架 —— 与本仓（L0–L6 底座 + neobot 壳）无关。",
        "noise",
    ),
    (
        r"github\.com/(?:togg53192-cmd/jailbreaks|"
        r"elder-plinius/GL4SS|SnailSploit)",
        "jailbreak / 越狱集合 —— 与工程无关，且引入对抗性内容。",
        "noise",
    ),
    (
        r"https?://(?:www\.)?(?:tophub\.lol|trendshift\.io)",
        "聚合/排行榜站：它们是**噪声的来源**而非可吸收项目。",
        "noise",
    ),
]


def scan(text: str) -> list[tuple[int, str, str, str]]:
    """返回 [(行号, 命中的 token, 类别, 理由)]。"""
    hits: list[tuple[int, str, str, str]] = []
    for pat, why, kind in RULES:
        rx = re.compile(pat)
        for i, line in enumerate(text.splitlines(), 1):
            m = rx.search(line)
            if m:
                hits.append((i, m.group(0)[:60], kind, why))
    return hits


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__)
        print("FAIL: 用法错误 —— 需要恰好一个参数（清单文件路径，或 `-` 读 stdin）")
        return 2
    src = argv[1]
    try:
        text = sys.stdin.read() if src == "-" else Path(src).read_text(
            encoding="utf-8", errors="ignore"
        )
    except OSError as e:
        print(f"FAIL: 读不到清单：{e}")
        return 2

    # ⛔ 空清单不得「空过」：那会是一个永远绿的假门。
    if not text.strip():
        print("FAIL: 清单为空 —— 空清单必须判红，否则本门是永远绿的假门")
        return 1

    hits = scan(text)
    if not hits:
        n = len([ln for ln in text.splitlines() if ln.strip()])
        print(f"PASS: 清单 {n} 行，黑名单零命中。")
        return 0

    by_kind: dict[str, int] = {}
    for _ln, _tok, kind, _why in hits:
        by_kind[kind] = by_kind.get(kind, 0) + 1
    print(f"FAIL: 命中黑名单 {len(hits)} 处 —— {by_kind}")
    for ln, tok, kind, why in hits:
        print(f"  [{kind}] :{ln}  {tok}")
    print("\n理由（按类别去重）：")
    for _pat, why, kind in RULES:
        if any(k == kind for _l, _t, k, _w in hits) and any(
            w == why for _l, _t, _k, w in hits
        ):
            print(f"  · {why}")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))