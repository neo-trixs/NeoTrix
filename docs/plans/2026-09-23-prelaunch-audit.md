# 桌面 App 上线前审计（2026-09-23 融合四仓＋ntos 吸收）

> 方法：静态横扫＋全量测试（tsc/vitest/build）＋Playwright 终极冒烟 24 项。
> 吸收视角：Lingee（企业态完整度）／OpenMuse（交互正确性）／unreal（异步契约）／
> buzz（审计追踪）／cumora（身份表达）／ntos（独立壳简洁度）。

## 一、执行结果

| 项 | 结果 |
|---|---|
| tsc --noEmit | 0 error |
| vitest 全套 | 71/72 文件，489 passed（1 flaky：RightBar GlobeView，单跑全过，与本轮无关） |
| vite build | ✓ |
| 终极冒烟 `/tmp/ultimate_smoke.py` | **24/24**（9 设置分区＋开发视图＋市场过滤＋零真实 pageerror＋零本地请求失败） |

## 二、审计发现与处置

### P0（已修，本轮）
1. **DSH 市场开关死按钮**（`ImSection` 空 onClick＋`.ss-toggle` 样式不存在）→ 接 `get/toggleDshMarket`＋乐观切换＋失败回滚＋真实 switch 样式。
2. **市场深链接二次失效**（settings 已在 market 分区时侧栏直达不切 tab）→ `initialTab` 改 createEffect 同步。冒烟从 21/24 → 24/24。
3. **对标注释 165 处**→ 0（脚本＋手工；中途脚本误删未闭合注释头致 33 错，已逐个修复并加规则）。

### P0（未修，他窗领地，勿动）
4. **RightBar kanban/floor 硬编码 mock agents**（GOD Agent＋console.log 死回调，用户可达）→ 上线前必须接真实数据或隐藏入口。
5. **ntcode 后端 4 编译错**（他窗代码，E0283/E0308/E0277）→ 挡整仓 `cargo build`，二进制出不来。

### P1（建议，不挡 debug 上线）
6. **密钥存放**：ntos LLM Key＋主应用 API Key 走 localStorage 明文。后端有 keyring 依赖，建议迁移（Lingee sessionStorage 关窗即焚是反面教材，别学；要学系统钥匙串）。
7. **401/过期回灌**：Lingee Bridge `token-expired → notify → 回退登录` 模式，主应用无统一过期处理（P0-3 原清单项，仍有效）。
8. **用量 token 明细**：三处用量 UI 均为调用计数聚合，token/credits 诚实置零（已标注，不算欺骗，但 release note 要写）。
9. **空态覆盖**：SkillStates 水印/升级引导明确 `不实现`（有记录，可接受）。

### P2（路线图）
10. IM 作者归因＋AgentBadge、@参与者目录、receipts、市场 3 列＋产物/点数、session 存储版本化、AG-UI（不跟）。

## 三、上线测试清单（终极）

### 自动（已跑，ुवं全绿）
- [x] `npx tsc --noEmit` → 0 error
- [x] `npx vitest run` → 489 passed（1 flaky 单跑过）
- [x] `npm run build` → ✓
- [x] `python3 /tmp/ultimate_smoke.py` → 24/24

### 需二进制（他窗编完后按序）
- [ ] `cargo build -p neotrix-tauri` → 0 error（先修 P0-5）
- [ ] 启动 `desktop --ntos` → 双窗口＋`ntos 壳窗口已创建` 日志
- [ ] 跟进队列目验：连发两条，第二条进队→自动 drain
- [ ] 队员 roster 目验：IM 配 bot 后侧栏＋IM 页身份行
- [ ] ntcode 第一测：`getNtcodeModels`＋流式 chunk
- [ ] 模型 Key 真聊：ntos 填 Key→测试连接→流式回复
- [ ] RightBar kanban/floor：真数据或已隐藏（P0-4 关闭项）

### 发版前人工
- [ ] 关于页版本号与 tag 一致（现 0.22.0 已统一四处）
- [ ] release note 写明：用量 token 置零、水印/升级未实现、DSH 市场 NL 降级语义
