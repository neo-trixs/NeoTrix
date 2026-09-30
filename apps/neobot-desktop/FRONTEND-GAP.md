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

## §1 Rust 薄壳：**已编译验证**（2026-09-30，内存闸 OPEN 后）

`cargo check -p neobot-desktop --all-targets` 0 error，
`cargo test -p neobot-desktop` 3 passed。

⛔ 但它**在此之前从未被编译过一次**，且不是「差一点」——
   `apps/neobot-desktop/Cargo.toml` 存在却**没进 workspace members**，
   于是 `cargo check -p neobot-desktop` 报「did not match any packages」。
   首轮真编译共暴露 4 类真错误：

| # | 错误 | 为什么之前没发现 |
|---|---|---|
| 1 | 不在 workspace 成员里 | 从没编过；**写好了 ≠ 接上了** |
| 2 | `frontendDist: "../frontend/dist"` 解析到不存在的 `apps/frontend/dist` | 同上 |
| 3 | `await` 出现在非 `async fn` | 同上 |
| 4 | `context` 借用跨不过 `spawn_blocking` 的 `'static` 边界（E0597） | 同上 |

> 这四条都是「看起来完全正常」的代码。**唯一能发现它们的是编译器。**
> 故下面这条命令是本 app 的入场券，不是可选检查：
>
> ```sh
> cargo check -p neobot-desktop --all-targets -j2
> ```

## §1b 已失效（保留作记录）

`.neotrix/patches/neobot-uncommitted-rs-20260930.patch` 里的 3 个 `.rs`
（`main.rs` / `lib.rs` / `build.rs`）**已入库**，patch 兜底不再需要。
它当初存在是因为内存闸 BLOCKED；闸 OPEN 后补编才发现上述 4 类错误。

## §2 后端缺口：**已补齐**（2026-09-30）

用户决定把两个能力搬进本仓，neotrix 成为唯一真源：

| 命令 | 落点 | 状态 |
|---|---|---|
| `neobot_evidence_summary` | `crates/neotrix-neobot/src/nt_evidence.rs`（新建） | ✅ 8 测试 + 6 变异全抓到 |
| `neobot_send` | `apps/neobot-desktop/src/main.rs` | ✅ 空消息明确报错 |

⚠️ `neobot_evidence_summary` 返回**完整报告**（`EvidenceReport`）而非只有一句话：
前端的证据块要能列出每一条问题，只回一句「证据不足」等于把「哪一句有问题」藏起来。

`nt_evidence` 有一条硬语义：`sourced_ratio` 在 0 断言时是 `None` 而非 `Some(0.0)`。
「没断言过」与「断言全无出处」是不同的两句话，混为一谈就是把「我们不知道」
说成「它不干净」。有变异专门守这条。

## §3 数据目录未与 CLI 对齐

`main.rs` 的 `data_dir()` 目前按 `~/.neobot` 拼，**未核对 CLI 的实际解析**
（可能支持 `NEOTRIX_HOME` 之类覆盖变量）。已用 `TODO` 显式标注。

两处不一致的后果是「桌面建的会话，CLI 看不见」—— 极难查。
**这是当前最该先验的一条**，因为它不报错。

## §3b 新增不变量与其把关方式（2026-09-30 吸收 workdsh/cua 后）

`capability_registry` + `block_model` 现有 **13 组**变异验证，全抓到。
harness 同时跑 **tsc 与运行时自测**，因为部分不变量只有类型检查能抓：

| 变异 | 抓到它的关卡 |
|---|---|
| resolvedBy 空串放行 / 不 trim / actor 缺省填假身份 | 运行时自测 |
| `setActor` 不校验就写入 / 自委派放行 / 空 requestId 放行 | 运行时自测 |
| **`ReasoningBlock` 从联合类型移除** | **tsc（运行时擦除类型，抓不到）** |
| `ToolStep.outputs` 退化为 `unknown` | **tsc** |
| 比较型不要求出处 / 允许非 http(s) 出处 / 版本不拦 / 失败步不进摘要 | 运行时自测 |

> ⛔ 只跑 `node selftest.mjs` 时，上面两条 tsc 才抓的变异**全报「0 失败」**。
> node 对 TS 只擦除不做检查 —— 这是与「被掩盖的门」同类的假绿。
> 所以：**纯类型不变量必须由 tsc 把关，运行时自测抓不到它们。**

## §4 能力矩阵仍是静态的

`defaultCapabilities()` 给了一份默认矩阵。`neobot_core_capabilities` 命令已就位
但前端**尚未消费**。接上之后「界面为什么没有某个功能」才有单一答案；
现在是两处默认值在各自猜。

## §5 已知测量限制

- **16px 不可读**：完整卡通图标栅格化到 16px 只是一团。对照组：neotrix 自家
  `nt-core-icon-16.png` 同样不可读（这族图标 16px 是凑档位，不是使用尺寸）。
  UI（侧栏/托盘/favicon）另用 `mark-mono.svg` 简化档，该档在 16px 仍读得出轮廓。
- ✅ **headless 渲染验证已补**（2026-09-30）：`scripts/ops/nt_check_layout.mjs`。
  此前 `--dump-dom` / `--screenshot` / `--headless=new` 在本机**全部挂死**
  （各烧掉一次 30 分钟超时），故改用 **CDP**（`--remote-debugging-port`）——
  实测 1s 就绪，且生命周期由脚本显式 `kill`，不靠浏览器自己退出。
  Node 22+ 原生 WebSocket ⇒ **零依赖**。
  视口用 `Emulation.setDeviceMetricsOverride` 设（`--window-size` 只改窗口，
  headless 下内容视口不跟着变，实测 820 → innerHeight 仍 413）。
  判据五条：高度链每环 `min-height=0` · 注入 40 会话/60 消息后真的可滚 ·
  token 在**计算样式**里非空 · 输入框在视口内 · 无横向溢出。
  变异验证 **5/5**，含**本仓历史上那次真实 bug**（摘掉 `.convs` 的 `min-height:0`
  ⇒ 门报 `.convs 的 min-height=auto`）。
