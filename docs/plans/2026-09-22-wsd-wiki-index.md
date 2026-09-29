# WSD 企业 Wiki 索引（NeoTrix 挂载页）

> 日期：2026-09-22 ｜ 类型：外部企业知识库索引（实体在 `/Users/neo/Downloads/wsd/wiki/`，本文件仅挂载，不复制内容）
> 遵循 `DOCUMENTATION-MAP.md`：plans/ 日期前缀 + 稳定架构不进 architecture/ + API 文档不手写。

## Wiki 位置

- 实体：`/Users/neo/Downloads/wsd/wiki/`（19 篇 docs/ + mkdocs.yml + .obsidian/ + scripts/；正文已移入 `docs/` 子目录以满足 mkdocs 规范）
- 本地预览：`cd /Users/neo/Downloads/wsd/wiki && bash serve.sh` → http://127.0.0.1:8001（mkdocs 1.6.1，`site/` 已构建验证通过）
- Obsidian：直接打开 `/Users/neo/Downloads/wsd/wiki` 作为 vault（附件 `attachments/`）

## 分册（绝对路径，可直达）

- 首页：`/Users/neo/Downloads/wsd/wiki/docs/README.md`
- 01 企业画像：`/Users/neo/Downloads/wsd/wiki/docs/01-企业画像.md`（含 1b 邮件签名身份拼图：WESDOM/WEIZIDOM 三拼写）
- 02 组织与团队：`/Users/neo/Downloads/wsd/wiki/docs/02-组织与团队.md`
- 03 客户与市场：`/Users/neo/Downloads/wsd/wiki/docs/03-客户与市场.md`
- 04 产品与阀门知识：`/Users/neo/Downloads/wsd/wiki/docs/04-产品与阀门知识.md`（第 4 节已纠偏：外贸列基本全空，美元 4.1%/青岛 FOB 0.4%/天津 0%）
- 05 供应商与采购：`/Users/neo/Downloads/wsd/wiki/docs/05-供应商与采购.md`
- 06 订单与合同流程：`/Users/neo/Downloads/wsd/wiki/docs/06-订单与合同流程.md`
- 07 营销获客与渠道：`/Users/neo/Downloads/wsd/wiki/docs/07-营销获客与渠道.md`
- 08 邮件与沟通SOP：`/Users/neo/Downloads/wsd/wiki/docs/08-邮件与沟通SOP.md`（含 2b：415 封模板库分布）
- 09 数据资产与系统：`/Users/neo/Downloads/wsd/wiki/docs/09-数据资产与系统.md`
- 10 术语词典：`/Users/neo/Downloads/wsd/wiki/docs/10-术语词典.md`
- 11 S类36名单：`/Users/neo/Downloads/wsd/wiki/docs/11-S类36名单.md`
- 12 菲律宾转化复盘：`/Users/neo/Downloads/wsd/wiki/docs/12-菲律宾转化复盘.md`
- 13 哈萨克斯坦激活专题：`/Users/neo/Downloads/wsd/wiki/docs/13-哈萨克斯坦激活专题.md`
- 14 补抓操作手册：`/Users/neo/Downloads/wsd/wiki/docs/14-补抓操作手册.md`（状态见 `/Users/neo/Downloads/wsd/wiki/14-补抓状态.json`）
- 15 WA话术挖掘：`/Users/neo/Downloads/wsd/wiki/docs/15-WA话术挖掘.md`（4232 条，英 83.9%/俄 11.2%）
- 16 询盘语料与GEO追踪：`/Users/neo/Downloads/wsd/wiki/docs/16-询盘语料与GEO追踪.md`（1809 条，法兰 55.2%）
- 17 关联公司与外部补全：`/Users/neo/Downloads/wsd/wiki/docs/17-关联公司与外部补全.md`（威之盾 137；清洗脚本 `wiki/scripts/clean_contract_amounts.py`）
- 数据血缘：`/Users/neo/Downloads/wsd/wiki/docs/DATA-SOURCES.md`（复算命令已换绝对路径可跑通）

## 数据快照

- CRM 2652 客户 / 21 业务员 / 中央产品库 6440 行 / 订单文件夹 292 / 采购合同 778
- 富通天下 corporationId 67847 / companyId 65491；补抓阻塞在“需用户 Chrome 重登”（见 14）
- 口径纠偏：菲律宾订单 37（非 26）、哈萨克订单 10（非 6），见 12/13

## 与 NeoTrix 的关系

- WSD 外贸 Agent（Rust）与 `neotrix-core/src/l1_action/nt_act/nt_act_trade/`（18 模块 + unified_types.rs）为业务原型关系，详见 wiki 09
- 本索引不引入 wsd 数据进 neotrix git；如需 KB 入库，走 nt_file_ability / experience-tree 流程另立任务
