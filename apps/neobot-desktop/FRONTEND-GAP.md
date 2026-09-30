# 前端已知缺口（FRONTEND-GAP）

> 记录于 2026-09-30。**这份文件存在的意义是：不把「写好了」当成「能用」。**

## 状态总览

| 组件 | 状态 | 说明 |
|---|---|---|
| 架构层（token / 契约 / 门控） | ✅ 已验证 | `src/selftest.ts` 20 项全过 + 变异验证 |
| token 门 / 字节门 | ✅ 已验证 | 三个变异全部被抓 |
| 前端构建 | ✅ 已验证 | `vite build` 通过，产物含全部组件层 |
| 界面 | ✅ 可跑（演示数据） | `MOCK_CONVOS` / `MOCK_MEMBERS` 占位 |
| Rust 薄壳 | ⛔ **未编译验证** | 见下 §1 |
| 前后端联调 | ⛔ **未验证** | 见下 §2 |

## §1 Rust 薄壳未编译验证

`Cargo.toml` / `build.rs` / `src/lib.rs` / `src/main.rs` 写入时：

```
sh scripts/ops/nt_mem_gate.sh  →  exit 2（BLOCKED）
```

按仓库纪律（三道闸 §4.1）**禁止起 cargo 构建**。且 pre-commit 门对任何 `.rs`
跑 `cargo check --tests -p neotrix` —— 验的是 `neotrix-core`，**不是本 app**，
用它给本 app 背书不成立（handoff `nt_...20260928` §6 阻塞 2 已记过这条）。

⇒ `main.rs` 的正确性来自**逐个核对 `crates/neotrix-neobot` 的真实签名**，
不是编译器。已核对：

| 前端调用 | 库函数 | 签名核对 |
|---|---|---|
| `neobot_agent_run` | `nt_core::agent_run(&store, goal, Option<&str>)` | ✅ `nt_core.rs:273` |
| `neobot_convo_group` | `store.create_conversation(kind, title, &[String])` | ✅ `nt_store_convos.rs:47` |
| `neobot_convo_dm` | `store.get_or_create_dm(me, peer)` | ✅ `nt_store_convos.rs:414` |
| `neobot_core_capabilities` | `nt_core::capabilities_or_default` → `CapabilitiesInfo` | ✅ `nt_core.rs:254, 80` |

**闸解除后必做**：

```sh
cargo check -p neobot-desktop --all-targets -j2
```

## §2 前端调了后端没有的命令

`src/main.ts` 引用了这两个名字，**本仓的库不提供**：

| 前端调用 | 状态 | 处置 |
|---|---|---|
| `neobot_evidence_summary` | 本仓**无证据模块**（在另一仓 Neo/neobot 侧） | 未注册。前端会拿到失败原因并显示，不是静默无反应 |
| `neobot_send` | 本仓**无发送入口** | 同上 |

这是**故意**的：注册一个点了必然失败的命令，会给人「看起来在工作」的假象，
比明确报错更贵。前端已把这两种情况都渲染成可见错误。

**待定**：是把证据/发送能力搬进本仓的库（则 neotrix 成为唯一真源），
还是这个 app 继续依赖另一仓的库。**这是产品/架构决策，不该由我替你定。**

## §3 数据目录未与 CLI 对齐

`main.rs` 的 `data_dir()` 目前按 `~/.neobot` 拼，**未核对 CLI 的实际解析**
（可能支持 `NEOTRIX_HOME` 之类覆盖变量）。已用 `TODO` 显式标注。

两处不一致的后果是「桌面建的会话，CLI 看不见」—— 极难查。
**这是当前最该先验的一条**，因为它不报错。

## §4 能力矩阵仍是静态的

`defaultCapabilities()` 给了一份默认矩阵。`neobot_core_capabilities` 命令已就位
但前端**尚未消费**。接上之后「界面为什么没有某个功能」才有单一答案；
现在是两处默认值在各自猜。

## §5 已知测量限制

- **16px 不可读**：完整卡通图标栅格化到 16px 只是一团。对照组：neotrix 自家
  `nt-core-icon-16.png` 同样不可读（这族图标 16px 是凑档位，不是使用尺寸）。
  UI（侧栏/托盘/favicon）另用 `mark-mono.svg` 简化档，该档在 16px 仍读得出轮廓。
- **无 headless 渲染验证**：本轮 `--dump-dom` 两次挂死（各 30 分钟），
  故界面**未做实测盒模型校验**，只做了静态校验（token 解析、组件进包、
  typecheck、架构自测）。`scripts/check-layout.mjs` 那套实测门**尚未移植过来**。
