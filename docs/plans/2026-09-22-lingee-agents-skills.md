# Lingee Agents & Skills（2026-09-22 收割蒸馏）

> 象限: Reference | 全量: `raw/assistants_available_89.json`、`raw/skills_266_all.json`

## 1. Agents：89 个（Discovery 市场）

字段模型（建议照抄进 skill 注册表）：

```
assistantId/agentId/displayName/desc/icon/iconClass/tags/
templateId/templateVersion/currentVersion/agentStatus(PUBLISHED)/
availablePackageTypeCodes(Trial/Professional/Gift/Ultra)/
trialEnabled/installed/installable
```

- 全可安装；已装 12（记账/经营分析/集团内部对账/库存补货/呆滞料/HR筛选/银企对账/采购入库/CEO工作台/往来对账/人人领用/财务报表编制），明细见 `assistants_installed_12.json`（含版本漂移如银企对账 1.0.5→1.0.6）。
- 版图：财务 ~20（费用审核 v1.0.17 最高）、供应链 ~15、销售 ~12、HR ~8、CEO/CFO 4；约 30 个 workshop/共创/测试 Agent（`0908/自进化/bwtest/第九组` 前缀，多为 v1.0.0）——市场冷启动靠培训批量生产。
- 发布：`POST /api/assistants/upload` **只收 zip**（`upload_zip_only`），带 entitlement 分级 + passcode 解锁；i18n 含上传/删除/锁定全套状态。
- 检索：`/available/search?keyword=`；`/installed/{id}/runtime-data` 给用量（§架构文档 5）。

## 2. Skills：266 个（12 分类）

分类（`skill_categories_12.json`）：财务/数据分析/销售管理/人力资源/效率工具/采购/信用管理/LTC/质量IT/基础通用/公共技能/工具。

- 构成：官方 94（`developer=null`：知识库/pptx/docx/pdf/xlsx/mermaid/CEO简报/灵基日记等基建，最高 v4.4.0）+ UGC 172；版本 `0.1.0` 占 143（共创一次性产物），`1.0.2` 占 52（模板批量生成）。
- `installStatus=1` 仅 4（用户研究/财务结账/项目管理/code-renderer）；个人 Skill 为空；`dataScope`: 8=172（需数云/ERP 数据）vs 0=94（纯对话）。
- 聚类（反面教材：跨租户重复造轮子，市场缺官方收敛版）：简历筛选 ×20+、销售订单查询 ×20+、合同审查 ×8；个人系列（吴涛无人机包、何洋订单链）。

## 3. Skill description 范式（prompt 工程金矿，照抄）

```
触发条件（当用户…时使用）+ 明确排除（不用于…/不要用于…）
+ 输出契约（结构化JSON / HTML报告 / 三档状态[通过|待复核|未通过] / 五维度加权评分）
```

- HR/风控类几乎统一"三档 + 五维度"；查询类必写"条件不完整时主动提问澄清"；MCP 类注明协议（IMAP/ERP写入/字段映射）。
- 已装 Tool 6（`tools_installed_6.json`）：云之家会议/日程、知识库（文档+多维表读写）、网页部署、联网搜索（ali-websearch）、金蝶SaaS builtin——MCP 目录即 Tool 目录，`openFlag/switchable/selectable` 三开关。
