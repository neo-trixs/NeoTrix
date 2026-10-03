# handoff — social_access 缺陷修复与生产接线（2026-10-03）

## 1. 会话标识

- 日期：2026-10-03
- 主题：吸收 bird / OpenCLI / AutoCLI / Agent-Reach 技术逻辑，修复 `social_access` 模块 7 项缺陷并接入生产
- 起点：用户问「可以获取 x.com 登陆么」→「装 bird 或 opencli 是什么外部依赖」→「完全吸收技术逻辑，完善对应缺陷」
- 执行方案：用户委托我自行决策，我选了自己建议的 **A（先修缺陷，不装任何东西）**

## 2. 目标（一句话）

把 `neotrix-core/.../social_access` 从「整模块零生产接线、探测必挂死、X 端点全死、凭据是假」修成可运行、可诊断、可 CI 判定的能力，并新增 `neotrix social` 入口。

## 3. 已完成

### 3.1 缺陷清单（每条都有实测证据）

| # | 缺陷 | 证据来源 |
|---|---|---|
| D1 | **假 timeout**：spawn 线程后 `.join()` 无限阻塞 | 我编译复现程序跑 `sleep 3600`，300ms 后仍阻塞，5s 被 kill |
| D2 | **死端点**：`api.fxtwitter.com/search` | curl 实测 **404** |
| D3 | **无登录墙判别** | 缺 OpenCLI LoginWallError 同类资产 |
| D4 | `bird --version` 探测必然误判 + `requires_auth` 是死字段 | bird README 命令表只有 `help`/`whoami`/`check` |
| D5 | **假凭据** `"twitter_client_id"` 编译进二进制 | 换 token 实测 **400 Missing client_secret** |
| D6 | **整模块零生产接线** | 全仓 rg 无外部调用方 |
| D7 | 生产路径 8 处 `unwrap`/`expect`/`panic!` | `rg -c` |

### 3.2 修复内容

- **新增 `nt_payload_guard.rs`**（吸收 OpenCLI + AutoCLI）
  - `PayloadVerdict`：Json / HtmlInsteadOfJson / ErrorEnvelope / Malformed / Empty
  - HTML 判别：`Content-Type` 含 `text/html` **或** `^<(?:!doctype|html|head|body|title)(?:[\s>/]|$)`（保留终止符守卫防 `<htmlfoo` 误命中）
  - 错误信封：`json.code !== 0`（补状态码嗅探的结构性盲区）
- **重写 `extractors/twitter.rs`**
  - 先抓真实响应再写码（vxtwitter profile/status、syndication tweet 的完整字段）
  - 诚实失败：搜索/时间线**返回显式错误而非假空**（空列表会被读成「无数据」）
  - 消除 `Handle::current()` panic 与 inherent/trait `search` 命名冲突
- **修 `probe.rs` 真超时**：`try_wait()` 轮询 + `kill()` + `reap`；**并发排管**防大输出死锁
- **`channel.rs`**：删除第二份重复探测实现；新增 `Credential` 枚举（`BrowserSession`/`EnvPresent`）+ 门控（在探测**之前**执行，让免登录 fallback 仍能胜出）
- **`auth.rs`**：占位凭据删除，改 `with_x_oauth()`/`with_reddit_oauth()` 显式注入；未配置时 **fail-fast**；cookie 路线加长度校验（消除 `login_x_cookies("","")` 假成功）；bind 失败经 oneshot 回传
- **新增 `entry/social.rs` + `neotrix social` 子命令**：doctor / probe / status / auth x，退出码 0/78 可 CI 判定

### 3.3 验证结果（全部实测）

```
cargo test -p neotrix --lib    → 12736 passed; 0 failed; 41 ignored
bash scripts/check-feature-gates.sh --quick → PASS（6 个非默认 feature）
python3 scripts/ops/nt_lock_audit.py neotrix-core/src → 可疑 0 处（核实 2026-10-03 14:10）
./target/debug/neotrix social doctor → twitter 渠道 active: vx-mirror (1039ms)
退出码：probe twitter=0 / probe github=78 / bogus=78 / auth x 无凭据=78
```

## 4. 正在改的文件（已改完，待提交）

| 文件 | 性质 |
|---|---|
| `neotrix-core/src/l2_perception/nt_world/social_access/nt_x_browser.rs` | **新增**（第二轮：自建检索器） |
| `neotrix-core/src/l2_perception/nt_world/social_access/nt_payload_guard.rs` | **新增** |
| `neotrix-core/src/entry/social.rs` | **新增** |
| `.../social_access/extractors/twitter.rs` | 重写 |
| `.../social_access/probe.rs` | 重写 |
| `.../social_access/channel.rs` | 修改 |
| `.../social_access/auth.rs` | 修改 |
| `.../social_access/traits.rs` | 修改（`HttpPool::try_standard`） |
| `.../social_access/mod.rs` | 修改（注册新模块 + 导出） |
| `neotrix-core/src/entry/mod.rs` | +2 行 |
| `neotrix-core/src/main.rs` | +51 行（Social 子命令 + is_ops_cmd） |

## 5. 下一步（按优先级）

1. **安装 cookie 后端**（用户本人操作）：`npm i -g @jackwener/opencli` + 在 Chrome 登录 x.com + 装 opencli 扩展。装完 `neotrix social doctor` 应显示 opencli 为 active。
2. **提交**（见 8.2）—— 必须用 `git commit --only <逐个文件>`，共享 index 下暂存区核对与提交不原子。
3. **把 `nt_x_browser` 接进 `SocialAccessManager`**：`XBrowserRetriever::query()` 已可用，但 `SocialAccessManager::get_feed` 仍是 `vec![]` 桩。未接的原因：x.com robots 是 `Disallow: /`，默认门会拒绝 —— **要不要默认关闭门、还是要求用户显式声明授权，这是产品决策不是技术决策**，故留给用户。
4. `SocialAccessManager` 本身仍无调用方（`get_feed` 是 `vec![]` 桩）。它是**设计层未完成**而非缺陷，接线前需要先有真实 adapter 注册路径。

## 6. 阻塞点

无技术阻塞。仅一处**需用户决策**：X 搜索/时间线必须靠 cookie 会话（无免登录路径），我不能代为登录。

## 7. 给接手会话的话

### 🔴 第四轮：审计自己的产出 → 发现权重表「来源对、语义错」

前几轮我一直在修**别人写的**缺陷。这轮按指令回头审计**我自己这几轮的产出**，
用 `rg` 查每个符号的外部引用，结果：

| 符号 | 外部调用方 |
|---|---|
| `SocialAccessManager` | **0**（只在自己文件里） |
| `FeedService` / `UniversalRecommender` / `PlatformAdapterRegistry` | **0** |

⛔ 即：**我修完的模块里，编排层仍然是死的** —— 同一个罪（导出≠接入），
我前三轮治好了底层，却没治编排层。

#### 顺带挖出一个**语义级**缺陷（比死代码严重得多）

`feed.rs` 注释声称「融合 x-algorithm 开源权重」。核实：
`github.com/xai-org/x-algorithm` **真实存在**（Apache-2.0），
11 个权重里 **10 个与 `home-mixer/params/param.rs` 逐字相符**
（含 `-234.0` / `-58.8` 两个特征值）。**来源是真的。**

⛔ **但用法是错的 —— 正是上游在注释里逐字点名的那一条。**
上游 `param.rs:285-292` 原文：

> Each weight multiplies the *predicted* probability … **the weights do not
> multiply raw engagement counts.** One common misinterpretation is … the
> incorrect statement that **"one report cancels 468 likes"** — this is
> incorrect because the weights apply to the predicted probabilities.

而原实现是 `weight * count`。⇒ `report(-234)×1` 恰好抵消
`like(0.5)×468`，**数值上完美复现了上游说「不要这么读」的那句话**。

另外三处偏离（均由本次核实发现）：
- `reply_mutual = 20.0` 上游**无对应**：`BidirectionalFollowReplyWeightBoost`
  是 **15.0**，且是**条件修正**（只作用于 reply 这一个 head）。
  20.0 正好 = 5.0+15.0，说明是把基础权重与 boost 相加塞进一个槽位。
- `quote_dm = 5.0` 把 `ShareViaDmWeight(5.0)` 与 `QuoteWeight(5.0)`
  两个独立 head 合并了。
- 只建模 11/25 个 head，**遗漏项里有权重更大的**
  （`click 0.3 > open_link 0.2`、`quote 5.0`、`cont_click_dwell_time 0.4`）
  与**全部负权重**（`not_interested -47.52`、`block_author -31.2`）
  ⇒ 被屏蔽/不感兴趣的帖子排序虚高。

#### 修法（类型层面阻止复发）

- `actions: HashMap<String, f64>`（键=动作名，值=计数）
  → **`PredictedActions`**（枚举键 + `Prediction{0.0..=1.0}`）。
  ⭐ 越界概率**拒绝**而非 clamp —— clamp 会把 `count=468` 变成 `1.0`，
  **恰好抹掉这个 bug**。
- `unwrap_or(&0.0)` 静默计 0 → **枚举 + 穷尽匹配**，未建模 head 成为编译错误。
- `rank()` 返回 `RankOutcome::{Ranked, Unranked}` ——
  ⭐ **无预测时不得假装排过**（所有分数恒 0，顺序是任意的）。
- 补齐全部 20 个 head（含负权重）。
- 顺带修 `MAX_PER_AUTHOR` 的 off-by-one：原条件
  `count >= 2 && result.len() >= 20` 让多样性规则在**前 20 条完全不生效**
  （小结果集里同作者可占满），且用 `break` 会**丢弃后续所有帖**（含其他作者的）。

#### 新增两个 CLI 子命令（让编排层真被调用）

`neotrix social weights`（打印权重表 + 核对来源 + 未建模 head 时退出码非 0）
`neotrix social rank`（演示「权重×概率」并显式区分 ranked / 未排序）

实测输出：
```
share_copy_link 权重 20.0：
  high bob   score=10.000   (P=0.50)
  mid  carol score=4.000    (P=0.20)
  low  alice score=0.200    (P=0.01)
  ranked = true
无预测时：全部 score=0.000，ranked = false
```

### ⚠️ 第四轮的方法论教训

**核实落地率时，「搜到的 0」有两种可能**：
① 真的没落地；② **我搜的词不对**。
我在第三轮就犯过后者（搜台账措辞 `CDP契约` 而代码叫 `BackendCaps`），
第四轮复查时差点又犯 —— 最后靠 `rg -l` 列**文件**而不是数行数才发现
`entry/social.rs` 已经引用了它们。
⇒ **判断「是否死代码」要列引用文件，不看命中行数**；
且排除列表本身要先确认没有把唯一的消费者排除掉。

### 🔴 第三轮：同一对 URL 连发两次 ⇒ 我上轮只做了「实测+报告」，没真正吸收

**信号**：用户把完全相同的话又发一遍。反思后确认：上轮我停在「报告不能吸收」，
没有把**可吸收的设计**落进代码。⇒ 本轮改为直接施工。

#### 落地了什么

**新增 `nt_selector_contract.rs`（17 测试）** —— 治**我自己**的一个假成功。

背景：`nt_x_browser::parse()` 首版是「判完登录墙就 `Ok(empty_result)`」。
于是「选择器失效导致抽不到元素」与「真的没有结果」返回值完全相同 ——
**这正是本会话反复打击的失败模式，我自己又犯了一次**。

吸收的两条设计：
1. **lightpanda「声明能力面」**（设计，非代码 —— AGPL）⇒ `X_SELECTOR_CONTRACT`
   声明每个选择器的页面形态与是否 essential，`verify()` 运行时核对。
2. **open-slide「约束画布而非内容」**（MIT）⇒ 不给「抽取结果」加内容校验，
   而是约束**抽取面**，用探针让越界在结构上不可能。

⭐ 顺序语义（关键）：**先判形态再验契约**。未登录页天然 0 推文，
反序会把「请登录」误报成「X 改版了」—— 后者会让维护者去改代码，
而真正的问题是去登录。

⚠️ 同时补了一个 `BackendCaps` **没有**的层：它声明「后端能否执行 JS」，
而 X 抽取真正会坏的是「JS 里的选择器还能否命中」，此前无任何机制守护。

#### ⚠️ 更正上一轮的一个错误结论

上轮我记「lightpanda 的 CDP 契约设计**未落地**，落地率 1/4」。
**这个结论本身是错的**：我按台账措辞搜 `CDP契约`，而代码里叫 `BackendCaps`
（`nt_io_browser_engine/types.rs:87-127`）—— **它早已落地**。

⇒ 教训：**核实落地率必须搜代码里的概念名，不能搜台账里的标签名。**
台账用自己的话命名，代码用另一套话命名，按标签搜必然 0 命中，
然后就会得出「文档说 ✅ 但代码没有」的错误结论。
**这本身又是一次「台账不可信，代码才是真相」。**

#### 🔴 第三轮我又犯了三个错

1. **`check()` 首版缺 `expected` 参数**：自测抓到访问 `/home`（必然有推文的 feed）
   观测到 0 条时返回 `Ok(Unknown)` ⇒ 调用方照常报 `total: 0`。
   **假成功只是换了个位置。** 加 `expected` 后，形态不可判定时报**歧义错误**
   并点明「(a) 真无数据 / (b) 选择器漂移」，把判断权交还调用方。
2. **判据用错字段**：拿 `document.title` 去比对登录标记。实测 x.com 未登录页
   title 是 `X / 用户名`，`Log in to X` 在**正文**里 ⇒ 该匹配永远不命中。
   改为 `body_markers` 与 `doc_title` 拆成两个参数。
3. **两个测试自相矛盾**：改完语义后，`tweets_present_means_feed` 仍传
   `["Log in to X"]` 标记却断言非 LoginWall。**改代码时忘了改旧测试的期望** ——
   这是「改语义」的典型副作用，编译器不会提醒。

### 🔴 第二轮：用户质问「不是要自我构建么，为何还要 opencli」

**这个质问是对的，我的第一轮方案有根本缺陷**：我修好了地基（超时/端点/判别/门控），
却把**数据面能力**外包给 `opencli`/`bird` 两个外部 CLI —— 那不叫自我构建，
它把可用性押在「用户愿意装 Node 工具链」上。

⇒ 第二轮新增 `nt_x_browser.rs`：**自建浏览器驱动检索器**，零外部 CLI。
关键发现是本仓底座早已齐全，只是没人用：

| 组件 | 位置 | 说明 |
|---|---|---|
| `chromiumoxide 0.7` | `Cargo.toml:173` | 已在依赖里（`stealth-net`） |
| `UniversalBrowser` | `nt_io/universal_browser.rs` | 启动/stealth/新页面 |
| `CookieStore` | 同上 | cookie **持久化**到 `~/.neotrix/cookies/twitter.json` |
| `login_manual()` | 同上 | **人工登录一次即复用** |
| `page.evaluate()` | chromiumoxide | `.value()` 直接给 `serde_json::Value` |

⛔ **上一轮没自建成的真正原因**：`UniversalBrowser::eval()` 把 JS 返回值**丢弃**了
（只回传 `page.content()` 的 HTML）。结构化抽取必须拿到返回值。
⇒ `nt_x_browser.rs` 不复用那个 `eval()`，自己走 `page.evaluate()` 取值。

**技术选型（研究三个项目后）**：选 **DOM 路线**而非 GraphQL ——
bird/OpenCLI 的 GraphQL 必须维护旋转 `queryId`（每次调用去 GitHub 现取，**无磁盘缓存**），
等于引入一个**外部运行时依赖**；AutoCLI 的 DOM 路线无此依赖。

### 🔴 第二轮：用户给了两个 URL，实测结论是「都不能按字面吸收」

| URL | 实测结论 |
|---|---|
| `lightpanda-io/browser` | ⛔ **不可作 chromiumoxide 后端**：无 `Emulation.setAutomationDisabled`（源码零引用）、`--load-resources` 默认不加载、`product="Gecko"` 与 Chrome UA 不自洽（反检测反噬）、**AGPL-3.0 vs 本仓 MIT**。实测 `Page.navigate` 返 `BrowserContextNotLoaded`；抓 x.com **只得到登录页** |
| `open-slide/open-slide` | ⚠️ **与 `openslide/openslide` 是两个无关项目**。前者是 React 幻灯片框架（8,718★，MIT）；后者是数字病理 C 库（518★，**LGPL-2.1**，此前若记为 MIT 需更正）。前者 star 数 17 倍 ⇒ 任何「搜 openslide」都会先命中错的 |

⚠️ **lightpanda 本机已装**（`~/.local/bin/lightpanda`），以上是实测不是推测。

### 🔴 发现：本仓台账早就判过 lightpanda 不可吸收，但落地率是 1/4

`docs/architecture/absorption-sources/LICENSES.md` 已记 AGPL 不可吸收 + 「4 条设计可移植」。
用 `rg` 逐条查代码核实：

| 设计 | 落地 |
|---|---|
| robots 开关 | ✅ `nt_politeness.rs:42-45` 真解析真阻断 |
| CDP 契约 | ❌ 0 命中 |
| PandaScript | ❌ 0 命中 |
| MCP 会话隔离 | ❌ 0 命中 |

⛔ **台账写 ✅ 不等于代码里有** —— R-P79 在**文档层**的形态。
已在 LICENSES.md 补记复核结果 + lightpanda 实测边界 + openslide 澄清。

### 🔴 合规发现：x.com 的 robots.txt 是 `Disallow: /`

实测抓取 `https://x.com/robots.txt`：`User-agent: * / Disallow: / / Disallow: /i/u`
（对 Google-Extended/FacebookBot/Discordbot 是 `Disallow: *`）。

⇒ `XBrowserRetriever` **默认强制 robots 门**，复用本仓既有的
`parse_robots_disallows` / `robots_denied`（**不另写一套**，避免判定分叉），
门在**启动浏览器之前**（被拒不付浏览器启动代价）。
留 `without_robots_gate()` 但其 doc 明写「关闭即在抓一个明确禁止自动化的站点」。

回归测试直接用**实测抓到的真实 robots.txt** 作夹具，
且含一条**真实网络测试** `live_gate_rejects_x_com_search` 断言门会拒绝。

### ⚠️ 我自己犯的错（第二轮）

1. **`multi_byte_text_truncation_is_char_safe` 断言写错**：我以为「中文字幕测试内容」是 7 字，
   实测 8 字。是**我的断言错、代码对**。这类错若不去实测就会反过来改坏正确代码。
2. **`assert_robots_allows` 一度掉出 `impl`**：我的 edit anchor 丢了属性导致
   `async fn` 变成自由函数，编译器报 "await is only allowed inside async" ——
   靠读错误信息定位，没猜。
3. **假设了 `parse_robots_disallows(body, "*")` 两参签名**：实际是单参，
   且 `robots_denied(disallows, path)` 第二个参数顺序也不同。
   ⛔ 我又是先写假设再被编译器纠正 —— **应先读签名再写码**。

### ⚠️ 最重要的一条经验（仍然成立且更深）

**`cargo check` 全绿 ≠ 代码在跑。** 第一轮该模块 `dead_code` 告警为 **0**
（`pub` 在 `pub mod` 内不触发该 lint），12k+ 测试全绿而整个模块一行没跑过。
第二轮在**文档层**重现同一形态：LICENSES.md 写「✅ 4 条设计可移植」，实际只有 1 条落地。
⇒ **告警与台账都不能替代 `rg` 实测 + 端到端运行。**

### 方法论

- 研究同期的价值 > 修复本身：**AutoCLI 的 README env-var 表整表过时**
  （文档写 `OPENCLI_*`，实现零命中，真值 `AUTOCLI_*`，端口 19925 非 19825）。
- **文档不可信，实现才是真相。**
- 三次「先假设后被纠正」的教训（borrow 错误、字符数、函数签名）都指向同一件事：
  **写码前先跑一次最小验证。**

## 8. 收工自查

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 输出：

```
[worktree-gate] worktree=3 个 | 合计 167M | target 占 0M
[worktree-gate] 带未提交改动: 2 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 2 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /private/tmp/nt-v9
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/merge-b
```

本会话**新建** worktree 1 个，用于 clean-HEAD 对照测量：

| worktree | 用途 | 去向 |
|---|---|---|
| `/tmp/nt-verify` | 验证 `重复bootstrap幂等不掉数` 失败是否 pre-existing | ✅ 已 `git worktree remove --force` 移除 |

上表 2 个带未提交改动的 worktree **不是我开的**（`/private/tmp/nt-v9`、`.worktrees/merge-b`），按并发公约未触碰。

### 8.2 未提交改动的去向

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `social_access/nt_selector_contract.rs` | **新增选择器契约**（第三轮） | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/nt_x_browser.rs` | **新增自建检索器**（零外部 CLI + robots 门 + 契约门） | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/nt_payload_guard.rs` | 新增载荷判别层 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `docs/architecture/absorption-sources/LICENSES.md` | lightpanda 实测边界 + openslide 澄清 + 落地率复核 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `entry/social.rs` | 新增 CLI 入口 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/extractors/twitter.rs` | 重写为实测端点 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/probe.rs` | 真超时 + 并发排管 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/channel.rs` | 去重探测 + 凭据门控 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/auth.rs` | 去假凭据 + fail-fast | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/traits.rs` | `HttpPool::try_standard` | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `social_access/mod.rs` | 注册新模块 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `entry/mod.rs` | +2 行 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |
| `main.rs` | Social 子命令 | ☐ 已提交 ☐ **待用户 `git add`** ☐ 弃用 |

⛔ **未提交。用户未要求提交，我不擅自 commit。** 若接手者丢弃，这些改动全部消失。**建议尽快 `git commit --only <上述10个文件>`**（共享 index 下必须逐个指定路径）。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**4**
  ⛔ 原因：**他窗**的 2 个 worktree（`/private/tmp/nt-v9`、`.worktrees/merge-b`）有未提交改动不在任何提交里。**不是本会话引入**（本会话唯一 worktree `/tmp/nt-verify` 已移除）。按并发公约未替他人 prune。
  - ⚠️ 首次运行时 rc=0（彼时那 2 个 worktree 尚未被写入），复测变 4 ⇒ **门记录必须带核实时间戳**（R-SCAN-3），本条以复测为准。
- `cargo xl`（= `cargo test -p neotrix --lib`）：☐ **是** —— **12774 passed / 0 failed**（nt_x_browser 17 + nt_selector_contract 17）
- `cargo check -p neotrix --lib`：☐ **是** —— 无 error
- `check-feature-gates.sh --quick`：☐ **是** —— **PASS**
- `nt_lock_audit.py neotrix-core/src`：☐ **是** —— **0 处**（核实 2026-10-03 14:10）
- 另注：clean HEAD (`ec83de30`) **自身编译不过**（`pipeline.rs` 两处 `crate::neotrix::nt_core_event_bus` 解析失败）⇒ **「HEAD 是可信地面真相」在本仓库当前不成立**，不要用 clean checkout 推断 CI 状态。
- 另注：修复过程中 `consciousness_runtime.rs`（**他窗**改动）与 `headless.rs`（**他窗**改动）一度出现编译错误，均已由那些窗口自行修好；与本会话 diff 无交集（已用 `git diff` 逐文件核对）。