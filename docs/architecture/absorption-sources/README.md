# 吸收源清单（2026-09-28）

> **为什么有这个目录**：2026-09-28 之前的吸收只把「109 仓 + trendshift 385 仓」当**数字**写进文档，
> **URL 本身只存在临时目录**。临时目录会被清空 ⇒ 接手窗口无法复现输入，也无法复核结论。
> 本目录把输入固化为可 diff、可审计的数据文件。

| 文件 | 内容 | 规模 |
|---|---|---|
| `repos.csv` | 全部吸收仓：`repo,source,stars,language,boards` | **483** 行 |
| `trendshift-boards.csv` | 榜单排名：`board,repo,rank,stars_gained,stars_total` | **436** 行 |
| `LICENSES.md` | 许可台账（三类 + 数字陷阱） | — |
| `PAPERS.md` | 5 篇 arXiv（含 1 篇无关项） | — |

## 采集方式与限制（复现须知）

- **用户清单**：原始 99 条 + 追加 10 条 = 109，全部经 GitHub API 核实存在
  （`OpenInterpreter/open-interpreter` 当时返回 `Moved Permanently`，仓库已改名）。
- **trendshift.io**：21 个榜单（weekly/monthly/yearly + 16 topics + github-trending），
  从 Next.js RSC 载荷解析（需先反转义 `\"`）。去重后 **384** unique。
- **⭐ 数据质量警告**（复用前必读）：
  1. yearly 榜的 `gained` ≈ 总星数（20/25 仓 r≈1.0）⇒ **不是周期增长，比值无意义**
  2. monthly 榜只有 `r ∈ [0.83, 0.99]` 段可信；前两名是垃圾（含 `gained > stars` 的数据异常）
  3. 17 个仓跨 6 榜属同一 Jev/Laya 品牌播种，**14/17 是 astroturf**
  4. `insights` / `signal` 两页解析出 **0 条**（结构不同）⇒ 无因果数据，只有星数差 + 描述
- **arXiv API 当日被封**（对照测试已做）⇒ 只能读 abs 页。

## 去重口径

`user-list` 99 + `user-list-append` 10 + `trendshift` 384 = 去重后 **483 unique**
（`repos.csv` 的 `source` 列记录了每个仓的来源，可交叉核对交集）。
