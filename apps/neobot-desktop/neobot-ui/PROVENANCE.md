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

⚠️ **踩过的坑（已固化进门）**：`vite` 默认 `base:'/'` 产出
`src="/assets/index-xxx.js"`，而 Tauri 走 `file://` 加载 ⇒ `/assets`
解析到**文件系统根** ⇒ **白屏**，且 `vite build` **成功、tsc 通过、零报错**。
⇒ 「构建通过」不等于「产物可用」。本目录 `base:'./'`，并由门守住。

## 5. 切换方式（M6，尚未执行）

```diff
# apps/neobot-desktop/tauri.conf.json
- "frontendDist": "frontend/dist"
+ "frontendDist": "neobot-ui/dist"
- "beforeBuildCommand": "cd frontend && pnpm run build"
+ "beforeBuildCommand": "cd neobot-ui && pnpm run build"
```

**在此之前不动 vendored 树。** 两套 UI 并存，切换可回滚。
切换后才可删 `frontend/src/vendor/` 与 dsh-tauri 链路（M6）。
