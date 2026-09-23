# NeoTrix 桌面端迭代清单（Lingee 对标，2026-09-22）

> 象限: How-to | 输入: 同目录 `2026-09-22-lingee-architecture.md`、`2026-09-22-lingee-agents-skills.md`、`repo-analyses/lingee-20260922/`
> 上游正典: `docs/architecture/FIVE-ENTITY-BLUEPRINT-V3.md`（E1/E3 字段与门禁定义的唯一来源；本清单只记桌面落点）。

## P0（本周可做）

1. **模型档位层**（1 天）：`model_router` 加 `modelLevel/autoRouting/consumptionCoefficient/orderNumber`
   四字段（Lingee 11 档实测：fast 0.3 / expert 0.9 / ultra 1.2 / Qwen-Flash 0.1 / Kimi-K3 3.3…），前端只暴露档位名。
2. **Skill 描述三段式门禁**（半天）：触发 + 排除 + 输出契约；266 条入库做 few-shot 示例。
   ✅ 已落地（2026-09-22）：`neotrix-core/src/skill_loader.rs` 加 `SkillEntry.exclusions/output_contract`
  （`#[serde(default)]`，旧 index.json 兼容）＋ `ResolvedSkill.admission: SkillAdmission`
  ＋纯函数 `gate_skill`（三段齐＝Admitted，否则 NeedsWork 明细）＋4 处构造点全接线
  （legacy 无元数据→NeedsWork 全缺）＋ `SkillFilter.require_admitted`＋排序 ＋8.0；
   新增 4 单测；rustfmt 全 clean；门禁逻辑 2/2 独立验证通过；cargo 全量待基线绿后复验。
3. **401 回灌登录**（半天）：照抄 LingeeBridge `token-expired → notify → 回退登录`，不再直接掉登录页。
   ✅ 已落地（2026-09-22）：后端 `src-tauri/src/browser_host.rs::AuthBridge`
  （`get_config`/`notify_token_expired`/`notify_logout`/`require_login`＋`auth_bridge_get_config`
   命令已注册进 `main.rs`；`BrowserError::Emit`＋`app_error.rs BROWSER_EMIT` 分支同步）＋
   前端 `src/api/events.ts::subscribeAuthBridge`（`token-expired`/`require-login`/`auth-logout`）。
   验证：rustfmt 我区零 diff；`tsc --noEmit` exit 0；cargo 全量待基线绿后复验。

## P1（两周）

4. **Bridge 事件表**（2 天）：Tauri 版 `getConfig/emit/notify`（logout/token-expired），附件可信路径按 OS 区分。
5. **用量看板 schema**（2 天）：照抄 `runtime-data`（scope/period/token/credits/dailyStats）。
6. **设计 token 两层化**（2 天）：命名层照抄 `--lg-*`（radius 4→32+pill/circle；spacing 2→72 非线性档；
   字族 PingFang SC 三槽合一；仅 light），值层换 NeoTrix gold。
7. **空/错/加载态查漏**（1 天）：对照 `skillCenter.empty.*`、`loadFailed*`、`installSuccess/Failed`、
   水印配置（开关/透明度/密度/范围）、套餐升级引导逐项补。

## P2（本月）

8. **市场字段模型**（3 天）：`templateId/version/entitlement/upload-zip` 进 skill 注册表（R-P100 评审依据），
   设"官方认证"去重（Lingee 简历筛选 ×20 的教训）。
9. **MFE 拆分预研**（5 天）：Host 壳 + 联邦远程（灵基 10KB 壳 + 三级懒加载实证），Desktop 先拆设置/市场两页试点。
10. **翻译 DOM 防护**（0.5 天）：React #11538 removeChild/insertBefore 父子校验进 webview。

## 不抄的部分

- 华为云 WAF 绑定架构、中台 entitlement 付费墙（阶段不符）、sessionStorage 放 token（关窗即焚的坑，见本次登录复盘）。
