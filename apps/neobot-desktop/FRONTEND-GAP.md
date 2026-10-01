# 前端已知缺口（FRONTEND-GAP）

> 记录于 2026-09-30，2026-09-30 重写（旧自研 UI 退役 + 上游 96 全登记后）。
> **这份文件存在的意义是：不把「写好了」当成「能用」。**

## 状态总览

| 组件 | 状态 | 说明 |
|---|---|---|
| 架构层（自持根 / API 面板 / 契约门） | ✅ 已验证 | 契约门 110 条 PASS + 状态门 PASS |
| token 门 / 字节门 | ✅ 已验证 | 变异全部被抓 |
| 前端构建 | ✅ 已验证 | `vite build` 通过（上游基线；增量仅文案与 3 文件逻辑） |
| 界面 | ✅ 可跑（真数据） | 会话列表读 store；消息问答落库，切会话换历史 |
| Rust 薄壳 | ✅ 已编译验证 | `cargo check -p neobot-desktop --all-targets` 0 error，`cargo test` 52+3 全绿 |
| 前后端联调 | ⚠️ 部分验证 | pet 状态四件套/配置/平台命令已通；面板链缺真实骨架下发器（演示下发器已通全链） |

## §1 Rust 薄壳：**已编译验证**

`cargo check -p neobot-desktop --all-targets` 0 error，
`cargo test -p neobot-desktop` 52 passed（另 3 条进程隔离集成测试）。

⛔ 但它**曾从未被编译过一次**（2026-09-30 前）：`Cargo.toml` 存在却
没进 workspace members，`cargo check -p neobot-desktop` 报「did not match any packages」。
首轮真编译共暴露 4 类真错误（不在成员里 / `frontendDist` 路径错 / 非 async fn 里 await /
`spawn_blocking` 的 `'static` 借用）。**唯一能发现它们的是编译器。**

> 入场券（不是可选检查）：
>
> ```sh
> cargo check -p neobot-desktop --all-targets -j2
> ```

## §2 后端缺口：上游 96 已逐条登记

| 域 | 状态 |
|---|---|
| 会话/面板/API（15 `neobot_*`） | ✅ 真实现，读 store / 注册表；消息落库 + 切会话换历史 |
| 平台/配置/桌面/日志（24 上游同名） | ✅ 已接线（剪贴板/通知/自启/窗口/配置/主题/日志） |
| 本轮新增 17（配置读侧/退出/揭示/打开/桌宠状态/能力位/穿透/清单/导入/资产/预设恒空/窗口） | ✅ `src/api.rs` Implemented + 注册 + 测试 |
| 50 Stub（插件/预装/档案/核心分发/服务日志/`toggle_sidebar`） | 本仓不跑 DSH 运行时或上游本体即空操作，决定不做；`neobot_api_call` 返回结构化说明 |
| 6 Planned（桌宠预设+鼠标流/更新 3/远端） | 待接，见 STATUS §3 |

⚠️ `neobot_evidence_summary` 返回**完整报告**（`EvidenceReport`）而非只有一句话：
前端的证据块要能列出每一条问题，只回一句「证据不足」等于把「哪一句有问题」藏起来。

`nt_evidence` 有一条硬语义：`sourced_ratio` 在 0 断言时是 `None` 而非 `Some(0.0)`。
「没断言过」与「断言全无出处」是不同的两句话，混为一谈就是把「我们不知道」
说成「它不干净」。有变异专门守这条。

## §3 数据目录：**已闭合**（原 TODO 已删）

`commands.rs data_dir()` 问 `NeobotConfig::from_env()` 要口径，
与 CLI 同源（`NEOBOT_DATA_DIR` 生效）。此前硬编码 `~/.neobot` 时，
设了变量会导致「桌面建的会话 CLI 看不见」且**不报错** —— 现已无此分叉。

## §3b 不变量与其把关方式

`capability_registry` + `block_model` 的变异验证全抓到。
harness 同时跑 **tsc 与运行时自测**，因为部分不变量只有类型检查能抓
（如 `ReasoningBlock` 从联合类型移除 —— node 只擦除不检查）。

> ⛔ 只跑 `node selftest.mjs` 时，纯类型变异全报「0 失败」。
> **纯类型不变量必须由 tsc 把关，运行时自测抓不到它们。**

## §4 能力矩阵：**已闭合**（旧静态矩阵随旧 UI 删除）

`neobot_core_capabilities` 由对话区顶栏直读（模型 · 工具数），
调失败就不渲染。旧自研 UI 的 `defaultCapabilities()` 静态默认已随旧 UI 删除，
「两处默认值各自猜」的病因已连根拔掉。

## §5 测量现状（2026-09-30 更新）

- **16px 不可读**：完整卡通图标栅格化到 16px 只是一团。
  UI（侧栏/托盘/favicon）另用简化档（`mark-mono.svg` 海豚版 / `public/favicon.svg` 黑豚剪影）。
- **布局门 v2 已复活**：`scripts/ops/nt_check_layout.mjs` —— stub-boot 在纯 Chrome 里
  跑起**生产包**（40 会话/60 消息），断言分支/几何/暗色翻色/a11y/零异常/零未登记。
  旧 `--dump-dom` 挂死问题仍在，故仍用 CDP + 显式 kill。
- **暗色**：自持区 chrome 走壳语义 token，翻色由门守；气泡固定内容色（文档在 `neobot-root.tsx` 头）。
- **仍缺**：运行中 Tauri app 内（WKWebView）的点击链路 —— CDP 接不上，
  需 Web Inspector 协议或骨架侧集成测试（STATUS §5-6）。
