# handoff — neobot-desktop 重建会话（2026-09-30）

## 1. 会话标识
- 仓：`/Users/neo/Downloads/neotrix`，分支 `feat/capability-absorb-20260828`
- 主题：把 NeoBot 桌面端重建为 **dsh-harness-desktop 0.19.1 原码 + API 契约平台**
- 结束状态：应用可启动，38 个命令已实现，工作树干净（我改的文件全提交）

## 2. 目标（一句话）
用上游原码做前端主架构，先立 API 契约单一真源，再按契约补后端，最后把「后端的能力」长成界面上的东西。

## 3. 已完成

| 提交 | 内容 |
|---|---|
| `70b36420` | 前端换成 0.19.1 原码（逐字节 1:1，MIT，`VENDOR.md` 记出处） |
| `3002697e` | **API 契约平台**：`src/api.rs` 单一真源 + 五方向对账门 + 可视化面板 |
| `f5ba1c46` | 打通启动（三个真实阻断）+ 自持对话区 + 图标改海豚 |
| `7f348be8` | 优先级 1：9 个平台接线命令 + 清掉 `AGENTS.md` 污染 |
| `7e4fbe7f` | 优先级 2：核心/备份/配置 6 个命令（接线，非从零） |

中间还有若干共享 index 事故导致的**内容被他人 commit 卷走**（见教训 1）。

当前：契约 38 条全部实现 / 0 待做 / 44 个上游未展开；cargo 45 测试（app）+ 466（两 crate 合计）；release 0 warning。

## 4. 正在改的文件
无。未提交 = 0。

## 5. 下一步（按依赖排序）
1. **44 个上游未逐条展开的命令**（插件 10 / profile 8 / updater 4 / 桌宠 5 / 杂项 17）
   —— 契约里只有名字没有理由。逐条写理由才能做，否则是「一个都不漏但不知道漏了什么」。
2. **对话区仍是基础版**（`neobot-root.tsx`）：单会话、无流式、无工具块渲染。
   上一轮删掉的 `block-model/blocks.ts/panel-view` 那套类型化块流**没有恢复**。
3. `neobot_send` 仍不带 `convo_id`、消息不落库（store 无 `messages` 表）。

## 6. 阻塞点
- **`@deepseek-ai/dsh-client-ui-*` 搬不了**。它是 Harness 运行时的**客户端**，
  17 个 peer 依赖，且其中 `@deepseek-ai/dsh-compact` 在 npm 上 **E404**。
  ⇒ 对话 UI 只能自己写。这是依赖图决定的，不是取舍。
- Tauri WKWebView 接不上 Chrome CDP ⇒ 真机端到端验证仍无手段。

## 7. 给接手会话的话

**这一节是本文档最值钱的部分。**

### ① 我在这一轮把「假绿」抓到了 7 次以上，形态各不相同
STATUS §4 记了前 7 次。这一轮新增的：

| 形态 | 后果 |
|---|---|
| `var(--nb-accent)` 语义层没有该 token | tsc/build/截图全绿，**那一条 CSS 属性静默失效** |
| 我写完 `// 覆盖 4.2/2.1/5.2 的勘误表` 就去读那份已被证伪的表 | 重复了别人已修的 bug |
| 变异 harness 把**编译失败**读成「测试通过」 | 「抓到 3/4」里有一条根本没变异 |
| 变异 harness **从陈旧备份还原**，覆盖掉我刚写的改动 | 19 条测试变回 17 条，而我看到的是 `ok` |
| 门自己用**硬编码模块名黑名单**滤路径段 | 每加一个模块就报一次假警 —— **错误的修法比不修更贵** |
| 门的「自检」是「从 Set 删掉再查它不在」 | **恒真，等于没检**。写了门 ≠ 门会 fail |
| 截图服务跑在 `/tmp` 下但截图在仓内 | 产物不在 git 里，评审看到的是旧的 |

⇒ 共同根因仍是那句：**把「没观察到失败」当成「验证通过」**。

### ② 三次被系统性问题绊倒，都不是技术问题
- **共享 index**：`git add -A` 之后被别人的 `git commit` 卷走两次；
  `commit --only` 又让删除声明失效（门从 `COMMIT_EDITMSG` 读，而 `--only` 不写它）。
  ⇒ 修法：声明**由暂存区自动生成**（手写 16 条、实际 47 条，当场被门拦下），
  并先把消息写进 `COMMIT_EDITMSG`。
- **vendored 的 `AGENTS.md`** 覆盖宿主项目规范。它要求「0 注释」，
  与本仓「把为什么记在代码里」直接冲突。
  ⇒ **一条写错的、被当成规范执行的指令，比没有指令更危险** —— 它会让人主动删掉正确的东西，
  而且看起来是在遵守规范。已改名 `AGENTS.upstream.md`。
- **1:1 复制时手挑文件列表** ⇒ 漏了 `tailwind.plugins.config.js` ⇒ Tailwind 产出空 CSS。
  ⇒ 改成「全量拷入 + 脚本逐项比对」。

### ③ 定位这类问题的方法，都比答案更可复用
- **「命令不存在」与「ACL 不允许」是两个完全不同的病因。** 前者查命令注册表，
  后者查 capabilities。前者症状是 `Command x not found`，后者是
  `Command x not allowed by ACL` —— **措辞差异就是线索**。
- **用编译器问 API**，比翻依赖源码快：
  `let _ = img.width;` 一次就问出「私有字段，方法同名要加括号」。
  我为一个方法名翻了 6 轮源码，编译器一轮就说清了。
- **不适用 ≠ 填假值。** 上游 `HarnessCore` 有 8 个字段，neotrix 只适用 3 个；
  编造 version/tag 会让界面显示一个不存在的「已安装 v0.2.0」。
  ⇒ 留空 + 测试守住。
- **「在干净环境下相等」证明不了「在覆盖变量下也相等」。** 数据目录那个 bug
  我第一版测试测不出来，因为干净环境下两种写法本来就一样。

### ④ 关于「保留上游所有特性」
我一度把 29 个命令标成 `Stub`（不做），理由是「它们管理的运行时我们不跑」。
用户随后要求「保留其所有特性」—— 这个纠正是对的：
**标 `Stub` 时必须写清理由**（有测试守），否则那些命令会一直躺在「不做」里，
而没人知道那是决定还是欠账。现在 38 个已实现，剩 44 个待逐条评估。

## 8. 收工自查

| 项 | 状态 |
|---|---|
| worktree 门 | `nt_worktree_gate.sh check` 已跑。**报 2 个 worktree 带未提交改动 —— 都不是本会话开的**（`/private/tmp/nt-v9`、`.worktrees/merge-b`、`.worktrees/nt-stop`）。本会话**未创建任何 worktree**，故无可 prune 者。⛔ 不要替别人 prune。 |
| 目标目录无未提交 | ✅ `git status --porcelain -- apps/neobot-desktop` = 0 |
| 他窗 WIP | 未触碰。剩余脏文件（`neotrix-core/**`、`.neotrix/*.json`、`results.tsv` 等）全是他窗的 |
| 后台进程 | ✅ 已 `pkill` 应用、截图工具、headless Chrome，残留 0 |
| 门 | ✅ API 门 PASS（契约 38/38）· 字节门 4532 文件 0 处 U+FFFD · release 0 warning |
| 测试 | ✅ app 45 passed · 上一轮两 crate 合计 466 passed / 0 failed |
| 未提交改动去向 | 全部已提交（`7e4fbe7f`）。无 patch 兜底需求 |

**⚠️ 交接时必须知道的一件事**：`apps/neobot-desktop/frontend/` 是 **vendored 第三方代码**
（dsh-harness-desktop 0.19.1，MIT）。改它之前先读 `frontend/VENDOR.md`。
里面有我改过的 2 处（`store.ts` 的 selfHosted 分支、`webview.tsx` 的渲染分支）、
删掉的 `AGENTS.md`，以及「为什么不取 `src-tauri/` 与 `source/`」的理由。
