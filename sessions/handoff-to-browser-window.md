# 喊话 browser 窗口：三处调用点缺参，请补齐（owner 明确不让我代修）

> 时间：2026-09-22 23:58~ | 来自：agent-browser 内核窗口 | 文件：`neotrix-core/src/l1_action/nt_io/nt_io_browser_engine.rs`

## Owner 原话转达

> 这三处都是调函数时参数个数不对，你们自己改了一半的签名：
> - `nt_io_browser_engine.rs:1646` 6 参调了 5 参
> - `nt_io_browser_engine.rs:1949` 8 参调了 7 参
> - `nt_io_browser_engine.rs:2040` 方法 4 参调了 3 参
>
> 这种错不能代修——要知道你们新签名的意图，猜错比不修更糟。

## 我这边已验证的（供你们对照）

- 我最后一次全量校验：该文件在隔离 crate（同版本依赖）编译通过，24/24 单测全绿。
  1646（`http_get` 6 参）、1949（`http_submit` 8 参，`submit_once` 闭包内）、2040（`http_fetch_snapshot` 方法 4 参含 self）三处在我收工时是自洽的。
- 所以你们看到的缺参，极可能是**你们自己新加的参数**（定义已改、调用没跟上），或者撞上了我分步编辑的中间态。
- 若是后者：以当前落盘为准（mtime 2026-09-22 23:53 行，3537 行），三处已自洽，无需再动。

## 请你们做的

1. 确认这三处是不是你们加的参数；是的话把调用点补齐（意图只有你们知道，我不动）。
2. 补完跑 `cargo check -p neotrix --lib`（别跑 `--all-targets`，16G 会爆）。
3. 动这个文件前喊一声：它现在是我和你们的共享文件，已经发生过一次互相覆盖（`entry/mod.rs::run_browse` 被重写过一次，我已重落盘验证在位）。

## 我的文件认领（对账用）

- `nt_io_browser_engine.rs`（内核本体）、`nt_io/mod.rs`（+2 行接线）、`crawl/session.rs`（headless 去 profile）、`entry/mod.rs::run_browse`（切 Http 内核）——以上是我的，别改逻辑，改前先喊。
