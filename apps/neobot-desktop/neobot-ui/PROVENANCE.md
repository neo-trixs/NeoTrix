# NeoBot 自持 UI（`neobot-ui/`）—— 溯源与边界

> 商用重构产物。**零 vendored（dsh-tauri）依赖。**
> 立项依据：`docs/architecture/FRONTEND-REBUILD-2026-10-01.md`（R1/R3/R5）。

## 1. 为什么有这个目录

原状：我方 UI（`neobot-root.tsx` 等 684 行）**不是** vite 入口，入口是
vendored 的 `frontend/src/main.tsx`，我方 UI 只能经

```
上游 main.tsx → … → 上游 layout/components/webview.tsx
  → selfHosted ? <NeoBotRoot/> : <iframe/>
```

被「请」去渲染。开关 `selfHosted = !hasService` 由上游 `harness/store.ts`
计算；后端 `get_runtime_info()` 硬编码 `has_service:false`
（`desktop.rs:185` 注释明写「返回空的 service_url + has_service: false」）
⇒ 恒为真 ⇒ 走我方分支。

**⇒ 文件级 import 零上游依赖属实，但渲染路径穿过上游。**
删 vendored 树时若只改 `frontendDist`，我方 UI 会**静默变成死代码**，
界面上没有任何异常。本目录就是那条缺失的替代路径。

## 2. 目录内容与来源

| 本目录文件 | 来源 | 授权 |
|---|---|---|
| `src/main.tsx` | **全新自持**（自持入口） | 本项目 |
| `index.html` / `vite.config.ts` / `tsconfig.json` / `package.json` | **全新自持** | 本项目 |
| `src/neobot-root.tsx` | 逐字复制自 `frontend/src/neobot-root.tsx` | 本项目自持 |
| `src/api-panel.ts` | 逐字复制自 `frontend/src/api-panel.ts` | 本项目自持 |
| `src/dom.ts` | 逐字复制自 `frontend/src/dom.ts` | 本项目自持 |
| `src/ui/nb-markdown.css` | 逐字复制自 `frontend/src/ui/nb-markdown.css` | 本项目自持 |
| `src/i18n/{zh-CN,en-US}.json` | 逐字复制自 `frontend/src/i18n/locales/` | 本项目自持 |
| `src/vendor/openghost/{markdown,highlight,tex}.js` | 逐字复制自 `frontend/src/vendor/openghost/` | **MIT（无附加条款）** |
| `src/vendor/openghost/shim.ts` | 逐字复制（53 行，自持垫片） | 本项目自持 |
| `src/vendor/openghost/VENDOR-OPENGHOST.md` | 逐字复制（逐文件 md5 清单） | — |

**`openghost` 是 R5 的落点**：MIT 渲染引擎、无附加条款 ⇒ **可保留**，
但必须**物理迁出** vendored（受限）目录，否则仍被打包进受限树。
本目录即该迁出目的地。许可门已**独立**审计本目录这棵树（实测输出见 §4）。

## 3. 复制件必须保持逐字一致

复制是**刻意的**：`frontend/src/` 有他窗在途改动，直接移动会撞车；
且切换应是**改一个配置项**，不是一次高风险删除。

⇒ 用**漂移门**保证「复制」不腐化：`nt_neobot_ui_wiring.py` 比对本目录
与 `frontend/src/` 的 md5。**任一文件分叉即门红**，并指出是哪一侧变了。
他窗改完原文件后，按门提示重新同步即可。

## 4. 已实测的结论（勿当假设）

```
tsc --noEmit  strict + noUnusedLocals/Parameters   rc=0  零错误
vite build    35 modules → 249.17 kB (gzip 81.93 kB) + 2.48 kB CSS
产物纯净度     上游特征符（overlastic/tanstack/toast-provider/store/modules）= 0 命中
              我方特征（neobot_api_call/log_frontend）= 在
              openghost 全局（Markdown/Tex/Highlight/code.copy）= 在
```

## 4.1 运行时冒烟测试（构建绿 ≠ 产物可用）

`scripts/ops/neobot-ui-smoke.mjs` —— headless Chrome（系统 `channel:'chrome'`，
免 `npx playwright install` 的 150MB 下载）在 **HTTP 子路径 `/app/`** 下加载
`dist/index.html`，注入契约忠实的 Tauri IPC 桩，断言
**① `#root` 被真实填充 ② 无未捕获错误 ③ 无资源加载失败**。

**实测**：`#root` 子元素 = 1，文本长度 = 103，零错误 ⇒ **自持 UI 真的渲染**。

ⓘ **本测试第一版报「未渲染」，核验后确认是【桩】的错，不是产品的错**：
桩对所有命令返回 `null`，而 `neobot_root.tsx:215` 声明
`invoke<ConvoView[]>`、Rust 侧返回 `Vec` 序列化成 `[]`，**永不为 null**
⇒ `v[0]` 抛 `Cannot read properties of null`。
⇒ **冒烟测试的桩必须按声明类型建模，否则测的是桩的 bug。**
（顺带记录一处真实脆弱性：`:219` 写 `v[0]?.id` 而非 `v?.[0]?.id` ——
 可选链只护住了 `.id`，没护住 `v`。**属原件问题，未在本副本改动**，
 以免造成漂移分叉。）

### 关于 `base` 的证据边界

| 断言 | 状态 |
|---|---|
| `base:'./'`（相对）在子路径下正确加载并渲染 | ✅ **已实证** |
| `base:'/'`（根相对）在 Tauri v2 自定义协议下必白屏 | ⚠️ **未实证，已降级** |

初版对照实验把路径改写成 `/app/assets/…`（顺手"修好"了）⇒ 那不是
`base:'/'` 的真实输出，**证明不了任何事**；改为**字面**根相对后，
子路径下确实渲染失败（`#root` 子元素 = 0）。但 Tauri 把 `frontendDist`
挂在**协议根**，`/assets/…` 未必失效 —— 故「必白屏」是**手推**，已降级。
仍取 `base:'./'`：它在两种挂载下**都**正确，是不依赖未验证前提的唯一选择。

## 5. ✅ 入口切换（M6 第一步，**已执行**）

`apps/neobot-desktop/tauri.conf.json` 的 `build` 段已改为：

```json
{
  "frontendDist": "neobot-ui/dist",
  "devUrl": null,
  "beforeDevCommand": "pnpm --prefix neobot-ui run dev",
  "beforeBuildCommand": "pnpm --prefix neobot-ui run build"
}
```

ⓘ **为何用 `--prefix` 而不是裸 `pnpm run build`**：Tauri 在
`tauri.conf.json` 所在目录执行该命令，即 `apps/neobot-desktop/`，
而那里**没有 package.json** ⇒ 裸写法必然失败。

ⓘ **`neobot-ui` 需要自己的 `node_modules`**：`pnpm --prefix neobot-ui install`。
本机验证期间我曾用软链指向 `frontend/node_modules`，但 **`pnpm` 拒绝**
（`workspace hoist directory is not a real directory`）⇒ 已改为真实安装。
这正是「本地能跑」与「别人能跑」的差别，已按后者为准。

## 6. R2 外壳 + R4 i18n（**已实现并经运行时验证**）

| 文件 | 作用 |
|---|---|
| `src/i18n/index.ts` | `t()` / `setLang()` / `onLangChange()`；语言经 `set_language` **持久化到后端** |
| `src/i18n/locales/{zh-CN,en-US}.json` | **我方自有 33 键**（实测与上游 452 键**零重合**） |
| `src/shell.tsx` | 顶栏：字标 / 语言切换 / 运行日志 / 退出；`useExternalLinks` 走 `open_external_url` |
| `src/shell.css` | 中性灰阶 + 单强调色；**不引用任何 vendored 主题变量**（那是 DSH 概念） |

**实测（`neobot-ui-smoke.mjs`，headless Chrome）**：

```
外壳：wordmark="NeoBot" 语言选择器=✅ 按钮=["查看运行日志","退出"]
语言切换：documentElement.lang zh-CN → en-US · 按钮文案 "查看运行日志" → "View run logs"
日志弹窗：✅ 返回 read_run_logs 的真实内容
```

⚠️ **i18n 曾被我自己的桩测成假象**：`addInitScript(fn)` 只序列化
`fn.toString()`，Node 侧词表闭包在浏览器里是 `undefined` ⇒ 每次 invoke
抛 `ReferenceError`，而 UI 把错误 `.catch` 掉**照样渲染**
⇒ 早先一轮「渲染通过」实际测的是**错误态**。已改为
`addInitScript(fn, table)` 传参。**教训：错误被吞掉的「通过」不算通过。**

ⓘ **未迁移的部分（如实记录）**：`neobot-root.tsx` 里仍有 24 处硬编码中文
字面量，尚未换成 `t()`。迁移它需改**他窗在途的共享文件**（该文件当前
有 884 行未提交改动）⇒ 本轮不做，词条已备好（`chat.*` / `time.*` 共 33 键），
迁移是纯机械替换。

## 7. M6 第二步：删 vendored 树（**⛔ 本轮不做**）

`apps/neobot-desktop/frontend/` 尚有**他窗 884 行未提交改动 + 2 个未跟踪新文件**
（`ui/nb-markdown.css`、`ui/skills-panel.tsx`）。
⇒ 删它 = 销毁他窗在途工作，正是 AGENTS.md 记载的 2026-09-28 事故
（850 处未提交改动凭空消失、事后靠 patch 找回）。
⇒ **入口已切、vendored 树保留**，产品已走自持路径且可回滚。
待他窗收工提交后，删树即可清掉 `check-license` 的最后一条受限来源。
