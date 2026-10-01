# NeoBot 前端自研重构清单（商用合规路径 B）· 2026-10-01

> 触发：项目所有者确认 **商用** ⇒ 上游 `deepseek-harness-desktop` 的
> 「No Commercial Secondary Development」附加条款**生效** ⇒ 若继续使用
> 1:1 vendored 且已修改的前端，即构成未授权的商用二次开发。
> 本文件给出**路径 B（自研重构）**的逐能力裁决与工作量。
> 路径 A（取得上游书面授权）见 §6。

## 0. 一句话结论

**不是「重写 86K 行」，而是「把 684 行自持代码长成完整产品」。**

两条实测支撑：

1. 我方自有前端只有 **684 行**，依赖只有 `react` + `@tauri-apps/api/core`
   + 自有 shim，只调 2 个 Tauri 命令，对上游 `store`/`hooks` **零依赖**。
2. ⭐ **vendored 前端的 62 处后端调用，全部指向我们刻意不移植的 DSH 功能**
   （48 个命令，契约表 48/48 已登记为 Stub/Planned，42 条注 `DSH_ONLY`）。
   ⇒ 那 88,600 行**本来就大面积跑不通**。真正在跑的只有我方 684 行。

⇒ 所以商用重构的实质是**换掉一个半残的上游壳**，不是重写一个产品。
详见 §1.0（含契约门的三方一致性实测）。

## 1. 事实基线【实测，2026-10-01】

### 1.0 ⭐ M3 已完成：契约三方一致，且暴露一个**改变战略判断**的事实

`scripts/ops/nt_api_contract.py`（新增门）实测结果：

```
契约表 115 条   Implemented=60  Planned=5  Stub=50
invoke_handler 注册 60 个 · 前端 invoke 89 个唯一命令
✅ A 契约标 Implemented 但未注册 : 0
✅ B 已注册但契约表无条目        : 0
✅ C 前端调了且契约无记录        : 0
ℹ️  C-info 前端调了标 Stub/Planned 的命令 : 48
```

⇒ **契约表 / 注册 / 前端实调三方完全一致。契约已冻结且可自证。**

**但那个 48 条 C-info 是本文件最重要的发现**：

| 分类 | 数量 |
|---|---|
| vendored 前端调用、后端未注册的命令 | **48**（62 处调用点，19 个上游文件） |
| 其中契约表已显式标 `Stub`/`Planned` | **48 / 48（100%）** |
| 其中 note 标注 `DSH_ONLY` | **42** |
| **我方自有 684 行的未注册调用** | **0** |
| 真正无记录的漂移 | **0** |

⇒ **vendored 前端的 62 处调用，全部指向我们刻意决定不移植的 DSH 功能。**
换句话说：那 **88,600 行 vendored 代码本来就大面积跑不通**（对着我们的 Rust 后端）。

**这改变了两件事：**

1. **合规成本被高估了。** 我们要删的不是一个能用的产品，
   而是一个**大部分功能在我们的后端上无法工作**的上游 UI。
   真正在跑的只有我方那 **684 行**。
2. **不是「重写」，是「收编」。** M1–M4 要做的不是把 88,600 行重写一遍
   （那是 DSH 的功能面，NeoTrix 不需要），而是**让 684 行自持代码长成完整产品**。

⇒ 净效果：**商用重构 ≈ 把一个半残的上游壳换成我们自己的壳**，
工作量与「重写 86K 行」相差两个数量级。

### 1.1 规模与依赖面

| 组成 | 文件 | 行数 | 归属 |
|---|---|---|---|
| 我方自有前端 | 4（3 ts + 1 css） | **684** | NeoTrix 原创 |
| `src/`（vendored） | 128 | 14,596 | dsh-tauri |
| `packages/`（vendored，13 包） | 766 | 71,807 | dsh-tauri |
| `src/vendor/openghost/` | 3 js | ~2,200 | ANDRETRIPOL/OpenGhost（**独立授权链**） |
| **vendored 合计** | **~897** | **~88,600** | — |
| 构建硬依赖 | `tauri.conf.json`: `frontendDist=frontend/dist` · `beforeBuildCommand=pnpm run build` | | |

### 1.2 我方自有代码的依赖面（这是 B 可行的关键证据）

| 文件 | 行 | import | 调用的 Tauri 命令 |
|---|---|---|---|
| `src/neobot-root.tsx` | 477 | `react` · `@tauri-apps/api/core` · `./vendor/openghost/shim.ts` | `log_frontend` · `neobot_api_call` |
| `src/api-panel.ts` | 184 | **无** | — |
| `src/dom.ts` | 23 | **无** | — |
| `src/api-panel.css` | — | — | — |

⇒ **对上游 `store`(4,230 行) / `hooks`(1,151 行) 零依赖。**
自研 UI 不需要重建状态管理层。

### 1.3 ⚠️ 修正：文件级零依赖 ≠ **渲染路径**零依赖（实测，2026-09-30）

我上一轮说「684 行对上游零依赖」——**就文件 import 而言成立，但渲染路径不是。**
实测真实链路：

```
vite 入口 = 上游 src/main.tsx          ← 不是我们的 neobot-root
   └→ 上游组件树 …
        └→ 上游 src/layout/components/webview.tsx:13
              import { NeoBotRoot } from '@/neobot-root'     ← 引用我方
              第 77 行： selfHosted ? <NeoBotRoot/> : <iframe/>
                        ▲
                        └─ selfHosted = !hasService
                             ▲
                             └─ 后端 get_runtime_info() 返回 has_service: false
                                （desktop.rs:185 注释明写「返回空的 service_url
                                  + has_service: false」；:205 硬编码 false）
```

**两个反直觉但正确的事实：**

1. **我方 684 行就是当前在跑的 UI** —— 正因为我们**没有** DSH 服务，
   `has_service:false` ⇒ `selfHosted=true` ⇒ 走我方分支，iframe/DSH 路径是死的。
2. **但它是被上游"请"来渲染的** —— 入口是上游 `main.tsx`，
   开关由上游 `harness/store.ts` 计算，容器是上游 `webview.tsx`。

⇒ **M1（换入口）不只是"换个入口"**：删 vendored 树时必须一并自持
`main.tsx` 入口 + `webview.tsx` 容器 + `harness/store.ts` 的 selfHosted 判定。
否则我方 UI 会**静默变成死代码**，界面上没有任何异常。

⇒ 已把这道风险做成门：`nt_api_contract.py` 的 **D 自持 UI 接线检查**
（校验 ① 我方 root 至少有一处 import ② 后端 `has_service` 不是 true）。
双向敏感度已验证：断开 import ⇒ rc=1；md5 逐字节还原 ⇒ rc=0。

## 1.4 ✅ 自持 UI 已建成并**构建通过**（2026-09-30，选项 2 产物）

`apps/neobot-desktop/neobot-ui/` —— **零 vendored 依赖**的独立前端树。
因 `frontend/src/` 有他窗在途改动，采用**逐字复制**而非移动（切换 = 改一个配置项）。

**实测（非推断）：**

```
tsc --noEmit   strict + noUnusedLocals/Parameters    rc=0   零错误
vite build     35 modules → 249.17 kB (gzip 81.93 kB) + 2.48 kB CSS
产物纯净度      上游特征符(overlastic/tanstack/toast-provider/store/modules) = 0 命中
               我方特征(neobot_api_call/log_frontend)                        = 在
               openghost 全局(Markdown/Tex/Highlight/code.copy)               = 在
```

**R5（openghost 迁出）✅ 完成**：已在受限树外独立存在，许可门**独立**审计这棵树
（实测两处 openghost 均判「无附加条款」= MIT 正确）。

**R1（换入口）部分完成**：自持入口 `src/main.tsx` 已建（刻意不挂上游
`QueryClientProvider` / `ToastProvider` / `OverlaysProvider` —— 我方 UI 不依赖其中任一，
挂了等于把 vendored 依赖重新引进自持树）。
**尚未切 `tauri.conf.json`**：两套 UI 并存、切换可回滚；待 §1.3 列的三处上游
（入口 / `webview.tsx` 容器 / `harness/store.ts` 开关）一并自持后才切（M6）。

### 1.5 ✅ 运行时实证：自持 UI **真的渲染**（构建绿 ≠ 产物可用）

`scripts/ops/neobot-ui-smoke.mjs` —— headless Chrome（系统 `channel:'chrome'`，
免 150MB 下载）在 **HTTP 子路径 `/app/`** 下加载 `dist/index.html`，注入
**契约忠实**的 Tauri IPC 桩，断言三条：`#root` 真实填充 / 无未捕获错误 /
无资源加载失败（按 **URL** 判定 —— console 文本是泛化的「Failed to load
resource」，**不含 URL**，按文本过滤必然失效，我第一版就这么错的）。

**实测**：`#root` 子元素 = **1**，文本长度 = **103**，零错误零资源失败
⇒ 在**他窗最新代码**上（漂移已同步）tsc 0 错、构建通过、运行时仍渲染。

ⓘ **本测试第一版报「未渲染」，核验后确认是【桩】的错，不是产品的错**：
桩对所有命令返回 `null`，而 `neobot-root.tsx:215` 声明 `invoke<ConvoView[]>`、
Rust 侧返回 `Vec` 序列化成 `[]`，**永不为 null** ⇒ `v[0]` 抛异常。
⇒ **冒烟测试的桩必须按声明类型建模，否则测的是桩的 bug。**
顺带记一处**真实脆弱性**：`:219` 写 `v[0]?.id` 而非 `v?.[0]?.id` ——
可选链只护 `.id` 没护 `v`。**属原件问题，未在本副本改动**（避免漂移分叉）。

⚠️ **同时把上一轮一条夸大的断言降级**（R-SCAN-2：手推 ≠ 实证）：

| 断言 | 状态 |
|---|---|
| `base:'./'`（相对）在子路径下正确加载并渲染 | ✅ **已实证** |
| `base:'/'`（根相对）在 Tauri v2 自定义协议下**必白屏** | ⚠️ **未实证，已降级** |

原写「Tauri 走 `file://` ⇒ `/assets` 解析到文件系统根 ⇒ 白屏」——
Tauri v2 实为自定义协议且把 `frontendDist` 挂在**协议根**，`/assets/…`
**未必**失效。且对照实验初版把路径顺手改成 `/app/assets/…`
（那不是 `base:'/'` 的真实输出 ⇒ **证明不了任何事**）；改为**字面**根相对后，
子路径下确实渲染失败（`#root` 子元素 = 0），但那只证明「子路径下不可用」，
不能推及 Tauri 协议根。
⇒ 仍取 `base:'./'`：它在**根挂载与子路径下都正确**，
是不依赖未验证前提的唯一选择。门的第 3 项措辞已按此边界改写
（「白屏守卫」→「挂载点无关性守卫」），`PROVENANCE.md` 同步修正。

**新门 `scripts/ops/nt_neobot_ui_wiring.py`**（5 项，rc=0 才算完整）：
1. 复制件漂移（md5 对照 + **mtime 归属判定**：原件更新 ⇒ 他窗在途，勿竞速同步）
2. 入口纯净：无 `@/` 上游别名、无 `../` 逃逸
3. `vite base` 为相对路径（白屏守卫）
4. 产物无上游特征符
5. openghost 已迁出受限树

**门自身被实测打脸两次（均修门而非改代码）**：
- 初版把一切 `./x` 判违规 ⇒ 误报 `./neobot-root`（**那正是自持文件**）。
  物理分离的两棵树里相对 import 必然落在自持树内；真正危险的是 `@/` 与 `../..`。
- `base` 检查不感知注释 ⇒ 匹配到注释里举例的 `base:'/'`，把已修正的配置报成缺陷。
  ⇒ 已加 `strip_comments`。
- ⇒ 与 M3 同源教训第三次复现：**门的判据要按结构写，不按字面写。**

## 1.6 ✅ 功能下架决策**由契约表推导**（新门 `nt_feature_viability.py`）

切入口前必须回答「会丢什么」。人工判断不可靠 —— 本仓已 3 次栽在
「导出 ≠ 调用 / 看起来在用其实没接线」。故把裁决做成门。

**判据**：按 vendored 前端**每个功能域**聚合其 `invoke` 的命令，
与契约表状态求交 ⇒ 三态：

| 域 | 调用 | 已实现 | 死 | 状态 | 结论 |
|---|---|---|---|---|---|
| `store` | 29 | 5 | 24 | MIXED | 24/29 指向 Stub/Planned ⇒ **今天就不工作**，下它不丢功能 |
| `ui` | 33 | 11 | 22 | MIXED | 同上（profile/插件/DSH 管理） |
| `layout` | 12 | 9 | 3 | MIXED | 外壳命令，属 **R2 待建** |
| `hooks` | 6 | 3 | 3 | MIXED | 多为未实现命令 ⇒ 下架 |
| `pet` | 5 | **5** | 0 | **ALIVE** | ⚠️ **真能用**，切入口会**真丢** ⇒ 显式决策下架 |
| `utils` | 2 | 2 | 0 | ALIVE | ✅ 已全覆盖（见下）|
| `i18n` | 1 | 1 | 0 | ALIVE | ⏳ **R4 待建** |

**唯一不变量**：不允许静默丢弃「今天真能用」的功能。
三态之外还要求归类：`DROPPED`（显式下架+理由）/ `PENDING`（挂里程碑未建）/
两者都无 ⇒ **门红**。

### ⭐ 由本门逼出来的两个真实发现

1. **`utils` 暴露一个静默失效缺陷**：自持 `copyCode` 用
   `navigator.clipboard`，catch 后**直接 return**。而 Tauri 走自定义协议，
   该 API 可能整个不可用 ⇒ 复制按钮**无声失效**。
   ⛔ 而后端 `write_clipboard_text` 是 `Status::Implemented` **且已注册**
   （`main.rs:58`）⇒ 已补降级路径。**这不是为让门变绿，是补真实缺失的降级。**
2. **归属判据按文件名是错的**：`(根)` 域里混着上游 `frontend/src/main.tsx`
   —— 它与 `neobot-ui/src/main.tsx` **同名但内容不同**（前者 import
   `OverlaysProvider`/`QueryClientProvider`/`ToastProvider`）。
   初版按「文件名存在」判归属 ⇒ 误认成我方 ⇒ 随后 `continue` 跳过，
   **掩盖了上游入口依赖 `get_dsh_theme`** —— 我们的上游入口去问「DSH 的主题」，
   是**DSH 概念泄漏**，正是要清掉的东西。
   ⇒ 已改按 **md5 内容同一性**判归属。这正是 AGENTS.md「同名 ≠ 同一符号」。

### 决策记录（可审计、可推翻）

| 域 | 决定 | 依据 |
|---|---|---|
| `store` `ui` `hooks` | 下架 | 绝大多数调用指向未实现命令，**今天就不工作** |
| `pet` | **下架** | 4/4 可用但实现属上游（许可阻断）；自研重做不在商用关键路径；纯装饰性。**恢复只需删门里那一行** |
| `layout` | 待建 | R2 自研 chrome |
| `i18n` | 待建 | R4 i18n |

## 2. 逐能力裁决

判据三问：① NeoBot 商用是否需要？② 需要的话，自研成本 vs 授权成本？
③ 是否已被我方代码绕过？

### 2.1 必做（阻断商用合规）

| # | 项 | 现状规模 | 自研工作量 | 说明 |
|---|---|---|---|---|
| **R1** | **应用外壳**（`index.html` 入口 · `main.tsx` → `neobot-root`） | 上游 `main.tsx` | **0.5 人日** | 我方 root 已在，只需换入口。这是 B 的**唯一硬阻断项** |
| **R2** | **最小 chrome**（窗口/滚动/快捷键） | `src/layout` 2,948 行 | **2–3 人日** | 自研。**不抄上游布局代码**（它在 no-commercial 范围内） |
| **R3** | `neobot_api_call` 后端契约固化 | ✅ **已完成** | **0（已交付）** | `scripts/ops/nt_api_contract.py` 门已建立：契约表 115 / 注册 60 / 前端 89 三方一致，A=B=C=0，敏感度双向验证通过 |
| **R4** | i18n（zh-CN / en-US） | `src/i18n` 70 行 + 两份 locale | **1 人日** | 词典小，自写成本低于剥离上游 |
| **R5** | `vendor/openghost` **独立裁决** | 3 js / ~2,200 行 | **0 或 2 人日** | MIT 无附加条款 ⇒ 可保留，但须**物理迁出** vendored#1。见 §4。⛔ **当前被并发阻断**：`src/neobot-root.tsx` mtime 实时变动（他窗在写），改 import 会冲突 |

**R1–R4 合计约 5.5–6.5 人日。** 这是让 NeoBot **不含任何 no-commercial 代码**的最小代价。

### 2.2 需你裁决（要或不要，决定后才有工作量）

| # | 能力 | vendored 规模 | 我的建议 | 理由 |
|---|---|---|---|---|
| **D1** | 桌面宠物 `dsh-tauri-pet` | 3,703 行 + `src/pet` 1,283 | **暂不做** | 5,000 行自研不划算；商用前先下架该功能 |
| **D2** | 插件/Skill/MCP 市场 `dsh-tauri-extension` | 5,272 | **不自研** | 这是 DSH 生态功能，NeoTrix 有自己的 capability 体系（`nt_policy`） |
| **D3** | 远程机 SSH `dsh-tauri-ssh` | **19,052（最大）** | **不自研** | 体量最大且与 NeoTrix 定位无关 |
| **D4** | worktree 管理 `dsh-tauri-worktree` | 8,541 | **不自研** | DSH 专属；NeoTrix 用 `nt_worktree_gate.sh` |
| **D5** | 调度器 `dsh-tauri-scheduler` | 5,056 | **不自研** | NeoTrix 有 `l6_meta` 运行时 |
| **D6** | 模型切换 `dsh-tauri-model` | 7,601 | **不自研** | NeoTrix 有 `nt_llm` |
| **D7** | 变更追踪 `dsh-tauri-experimental` | 7,387 | **不自研** | 与 NeoTrix 无关 |
| **D8** | 归档 `dsh-tauri-archive` | 2,187 | **不自研** | 同上 |
| **D9** | 右键菜单 `dsh-tauri-rightclick` | 2,404 | **暂不做** | 体量小但需原生集成；先下架 |
| **D10** | 组件库 `dsh-tauri-ui` + `src/ui` | 5,503 + 3,545 | **部分自研** | 只保留 R2 真正用到的组件；逐个评估，不整包搬 |

> **D1–D10 全部「不自研」的话，总工作量 = 0**。它们是**功能取舍**，不是重构任务。
> 这是本文件最重要的判断：**商用合规的成本主要不是写代码，是砍功能。**

### 2.3 明确不做

| 项 | 理由 |
|---|---|
| 抄上游任何代码「再改」 | 仍在 no-commercial 条款内，改不改都违规 |
| 保留 `dsh-` 包名 | 包名是标识符不是品牌（VENDOR.md 已注明），但自研后应整体去掉以免混淆 |

## 3. 执行顺序与里程碑

```
M1  R3 后端契约固化（先锁接口，否则 UI 重写会漂移）        1 人日
M2  R1 换入口 + R4 i18n → 跑通「最小 NeoBot」               1.5 人日
M3  R2 自研 chrome（不抄上游）                             2–3 人日
M4  R5 vendor/openghost 迁出 + 独立记录                     0.5 人日
M5  砍掉 D1–D10（功能下架，不是重构）                       逐项
M6  删除 vendored#1 整树 + 移除 tauri.conf 的 pnpm 链路     0.5 人日
    ─────────────────────────────────────────────
    合计                                                    约 6 人日（不含砍功能的连带测试）
```

**M6 之前不能商用分发。** M1–M5 期间 NeoBot 可继续内部使用
（内部使用不构成分发，但**条款的"直接使用"边界仍需上游确认**）。

## 4. ⛔ 最容易漏掉的一步：R5 的「物理迁出」

`src/vendor/openghost/` 是 **MIT**（渲染引擎在 MIT 内，非商用保留只覆盖
名字/徽标/视觉设计），**没有附加条款** ⇒ 技术上可保留。

**但它现在住在 `apps/neobot-desktop/frontend/` 内部**，
而该目录整体是 vendored#1。⇒ 只要打包 `frontend/dist`，
MIT 文件与 no-commercial 文件就混在同一个产物里。

必须做的：
1. 把 `src/vendor/openghost/` **迁到 `apps/neobot-desktop/` 之外**
   （如 `apps/neobot-desktop/neobot-ui/vendor/openghost/`）
2. 保留其 `VENDOR-OPENGHOST.md`（逐文件 md5 锁，是本仓最好的 vendor 记录范例）
3. 重跑 `check-license.sh` 确认它作为**独立树**被正确管辖

⛔ 不做这一步 = 商用产物里仍混有受限来源，R1–R4 全白做。

## 5. 风险

| 风险 | 判据 | 缓解 |
|---|---|---|
| 「重写」时不自觉抄了上游 | 检索新代码与 vendored 树的**标识符/字符串/注释**重合 | 门：新增 `check-derivation` 扫新 UI 与 vendored#1 的文本重合 |
| 砍功能连带打断测试 | `vitest.*.config.ts` 4 份 + `test/` | 每个 M 步跑一次前端测试；连带失败逐项判定 |
| `pnpm-lock.yaml` 与上游耦合 | 迁出后 workspace 解析可能失效 | M4 单独验证 `pnpm install && pnpm build` |
| 条款边界仍模糊 | 「内部使用」是否算 direct use 未定 | 需上游书面答复（§6） |

## 6. 路径 A（取得授权）——建议同时推进

自研 6 人日 vs 授权沟通成本未知。**建议两条并行**：
- **A**：向 `dsh-tauri/deepseek-harness-desktop` 提 issue 问
  「MIT + no-commercial-secondary-development 的前提下，
  非商用内部使用 / 商用自研重构，是否允许保留渲染层以外的接口设计？」
  （**问设计而非代码**，上游通常对接口/理念比实现更宽容）
- **B**：按 §3 执行

若 A 成功且上游愿意放宽，B 的 R1/R2 可降级为「按授权范围裁剪」，
省下 3–4 人日。

---

*依据：`.neotrix/LICENSE-EXCEPTIONS.md` 认定时间线 · `VENDOR.md` 条款原文对照 ·
`check-license.sh` 实时判定。所有数字为 2026-10-01 实测。*
