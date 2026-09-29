# Lingee UI 取证（2026-09-22 round2 登录态）

> 象限: Reference | 手段: Playwright＋系统 Chrome，`openwork.server.token` 注入 localStorage＋sessionStorage
> 产物: `repo-analyses/lingee-20260922/ui_ref/`（截图×5＋DOM＋1234 CSS vars＋API 增量 3 文件）
> token 只走环境变量，落盘已脱敏（电话严格 11 位独立数字，13 位时间戳保留）

## 1. 侧边栏解剖（`sidebar_ref.png`，宽 ~285px）

自上而下：

| 区 | 内容 | 样式 |
|---|---|---|
| 品牌行 | `Lingee` wordmark（左）＋ collapse 图标（右） | 16px 左右 |
| 分段 | `工作`（active 白 pill）＋ `开发`（`</>` 图标） | 灰底 segmented |
| 导航列表（纵向，非宫格） | 新任务（active 灰 pill）/ 定时任务 / 智能体 / 技能 / 发现 | 15px 行＋线性图标 |
| 历史 tabs | `历史对话`（active 下划线）/ `定时任务`＋右侧搜索图标 | 小字 tabs |
| 分组 | `默认` 灰小字 → 会话项（标题＋`云端`副标）→ `暂无分组`（空态 folder 图标） | — |
| 用户行 | 头像＋名＋`高级`紫 pill＋`反馈`＋菜单图标 | 底栏 |

**纠正上一轮合成**：导航是纵向列表不是 4 宫格；历史/归档是列表顶部 tabs 不是底部按钮；用户行带 plan 徽。

## 2. 作曲区（`shell_full.png` 中央）

- Slogan `灵基一动，工作轻松`＋业务 pills（日常办公/业务分析/业务执行＝网关 `X-Lingee-Business-Type`）
- 大输入框：`＋` `@` `选择智能体⌄` …… `自动⌄`（＝档位！）`mic` `发送`
- `选择分组⌄`＋快捷 chips（图像生成/网页报告/金银财报/问道阳明/PPT 生成）
- 右上紫 pill 横幅（运营位：参与灵基大赛…）

**结论**：档位选择器住在作曲区（`自动⌄`），不在侧栏；智能体选择器也在作曲区。NeoTrix 侧栏档位 pill 是合理简化（设置页跳转），作曲区档位下拉列入下一步。

## 3. Token 三层（`css_tokens.json`，1234 个）

```
基元 --lg-{palette}-{scale} → 语义 --lg-g-{bg,border,text}-{color}-{level} → 组件
```

- 基元：primary `#495DFF/#6D80FF/#3B47F2`，purple `#7F2AF3/#8F40FF/#6C1AD6`，violet `#572FF7`
- radius：small 4 / medium 6 / large 12 / pill＋circle 999px
- 字阶：small 10/12/14，medium 16/18，large 20/24/28/32/36/48/72；全 PingFang SC
- 页面：nav bg `#F1F1F6`，page `#F7F7FA`
- 语义层级词：faint / ghost / soft / subtle / muted / translucent-{faint,soft,subtle,muted} / emphasis / low / default

**对齐动作**：NeoTrix `base.css` 已有两层（`--lg-*` 命名→`--nt-gold-*` 值），补第三层 `--lg-g-*` 语义角色指向 gold 基元（本次落地）。

## 4. 市场卡片（`page_agents/skills.png`，蓝图参考）

- 通用头：标题＋右上主按钮（`上传技能` 蓝实心）＋搜索框
- 分段：我的智能体/今日协作；我的技能(4)/更多技能
- 过滤 tabs：全部/待激活/已激活＋年份下拉；我添加的/我上传的/我开发的
- Agent 卡（3 列）：渐变圆图标＋名（＋`Beta` 紫徽）＋`已在岗N天`＋底行`产物 0／点数 0`
- `待激活`分区：横向卡（图标＋名＋两行 desc，desc 即触发＋排除范式实证）
- Skill 卡（2 列）：方圆图标（单字）＋名＋两行 desc
- 真路由：`/ai-employee`、`/skills`、`/explore/discovery`

## 5. 用量 schema 实证（`raw/runtime_data_12.json`）

```json
{ "requestedDataScope": "my", "effectiveDataScope": "my",
  "period": { "key": "month", "startTime": "…", "endTime": "…" },
  "counts": { "taskCount": 0, "artifactCount": 0 },
  "usage": { "totalTokenUsage": "0", "totalCredits": 0.0 },
  "dailyStats": [{ "date": "…", "tokenUsage": "0", "credits": 0.0, "taskCount": 0 }] }
```

14 天 dailyStats。NeoTrix 用量卡 P1-5 按此 schema 接后端（当前为 provider_status 聚合过渡）。

## 6. 去重证据（`raw/search_multi_sample.json`）

销售订单 21 hits、采购 18 hits——跨租户重复造轮子实证，P2-8 官方收敛版＋"官方认证"去重的直接依据。

## 7. 侧栏实测构造（`ui_ref/sidebar_tree.json`，Playwright 计算样式）

容器非 `aside`，而是 `div.lg-sidebar.session-sidebar-v2`，280×900，bg `#F1F1F6`，`padding-top 52px`，flex 纵向：

| 节点 | 类 | 实测尺寸 | 关键样式 |
|---|---|---|---|
| 分段 | `.lg-sidebar-tab-bar[role=tablist]` | 280×32，pad 0 8px | 滑块 `.lg-sidebar-tab-bar__slider` 130×32，白 64%＋radius 999（动画滑块，active 背景透明靠滑块衬） |
| 分段项 | `button[role=tab]` ×2 | 130×32，icon 18＋label | active 仅 `aria-selected`＋字色 0.94 vs 0.82 |
| 滚动区 | `.lg-scroll-area.lg-sidebar__scroll` | 280×765 | — |
| 导航行 | `button.lg-sidebar-nav-item[--active]` | **264×32**，x=8 | 5 行（新任务 active＋定时/智能体/技能/发现） |
| 历史头 | `.lg-sidebar-sticky-header` | 280×43 | **sticky**：历史 tabs＋右侧 actions（搜索） |
| 分组头 | `.lg-sidebar-group-header` | 264×28 | — |
| 会话行 | `.lg-sidebar-list-item` | 264×34 | — |
| 文件夹区 | `section.lg-sidebar-folder-section` | 264×92 | — |
| 底栏 | `.lg-sidebar-footer` | 280×40，pad 8px | trigger＋actions 左右分栏 |
| 用户触发 | `.lg-sidebar-footer__trigger[role=button]` | 152×36 | **radius 20px**；头像 20＋名＋后缀徽 |
| plan 徽 | `span.lg-tag.lg-tag--sm.lg-tag--pill.lg-tag--info` | 36×20 | 官方 tag 组件三段修饰（size＋shape＋tone）——tag-pill 基座方向被实证 |
| 底栏动作 | `.lg-sidebar-footer__actions` 108×36 | 反馈文本钮 68×36＋知识库 icon 钮 36×36 | — |

## 8. 构造→NeoTrix 对齐（取证后修正）

| 取证 | NeoTrix 动作 | 状态 |
|---|---|---|
| 导航 264×32 行 | 4 宫格→纵向列表行（对话 active 灰 pill＋chevron），行高 h-8 精确 32px | 落地（含一次并发覆盖后重打补丁） |
| 历史 sticky 头 | 历史/归档 tabs 上移列表顶（list 外本就 fixed，等效 sticky） | 落地 |
| plan 徽 lg-tag 三段式 | `tag-pill tone-gold`（形态对齐，色走 gold） | 落地 |
| nav bg #F1F1F6 | `--lg-g-nav-bg` 由 #f4f4f6 修正为精确值 | 落地 |
| 滑块式分段 | 历史 tabs 保持下划线式（滑块需量测＋动画，暂缓） | 待定 |
| 底栏反馈＋知识库钮 | 无反馈后端，暂缓 | 待定 |
| 作曲区档位/智能体/业务 pills | Chat 输入区升级 | 下一步 |
| 市场卡片布局 | PluginMarketplace 改版 | 下一步 |
| runtime-data schema | 用量后端字段 | 下一步（P1-5） |
