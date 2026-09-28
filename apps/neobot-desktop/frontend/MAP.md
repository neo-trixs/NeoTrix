# NeoBot 桌面前端 MAP（链路总图，最后核验 2026-09-26，`scripts/map-check.sh` 三向一致）

> 入口 `index.html` + `settings.html`（vite 多入口）；零框架 vanilla TS。
> 本文档是改动索引：动任何一条链，先看本表。

## 1. 文件链

```
index.html ─┬─ styles.css（全 App token：字号5/圆角5/边线1/玻璃3/投影3/字体1 + 果球交互态）
            └─ src/main.ts ─┬─ icons.ts（16网格单线图标，data-icon 注入）
                            ├─ theme.ts（TS 侧单一源：品牌七件套/内置七模板/字体栈/头像渐变/果球色）
                            ├─ core.ts（壳状态：K/load/save/esc/toast/DOM/cache/shell/共享类型）
                            ├─ thread.ts（转录存取+渲染+任务/agent 节点卡，经 setThreadActions 回调主窗）
                            ├─ board_render.ts（画板纯渲染；事件装配留 main）
                            ├─ moodball.ts（接线层：live/rail/缩略活球注册表 + setBallColor 主题跟随）
                            ├─ emotionball.ts（自研表情泡：SVG 参数化脸 + 天气拟物饰件 + 6 语义态）
                            └─ @tauri-apps/api（invoke + event）
settings.html ─┬─ styles.css（复用 token）
               ├─ settings.css（setwin/setnav/行列 + 主题模板行/取色器样式）
               └─ src/settings.ts（个人信息 / 通用（外观+主题七模板+七色微调+字体+数据）/ 模型管理 / 关于）
vite.config.ts（双入口 main/settings；dev :1422；dist/ 即 Tauri frontendDist）
```

## 2. 渲染链（main.ts）

```
启动: paintIcons → applyPrefs → applyTheme（存量主题+果球体色） → applyWidths
      → initGrip → mountLiveBall → renderThread → restoreDraft → renderRail
      → refreshAll → heartbeat
      → 定时: heartbeat 30s / refreshAll 15s

refreshAll → 8 路 IPC 并行拉 cache → renderPanes
renderPanes → clearThumbs（杀旧缩略活球，防 rAF 泄漏）→ renderConvosPane
  （对话列表+队友，DM 行带 data-state: unread/busy/muted/sel）→ renderInfo（画板）
  → renderRail（角标/头栏/视图）→ hydrateBalls（占位挂活球）
renderRail → 头栏真实对话名（选中任务即标题）

用户动作链:
 作曲 onSend → setRunning → neobot_run_stream（Channel 增量+步骤）→ pushMsg → refreshAll
 ⚡ 键 → neobot_agent_run（goal=输入）→ assistant 消息 + <details> trace → refreshAll
 模型键 → paintCrystalPanel（配对在线只列晶体+回显；否则池子+免费发现+配对）
 右键行 → openConvoMenu（重命名/置顶/免打扰/成员/清空记录/删除）→ refreshAll
 选中行 → selectConvo（存旧草稿→标已读→取新草稿→渲染）
```

## 3. IPC 链（nt_commands.rs → neotrix-neobot）

| 前端 invoke | Rust 命令 | 核心落点 |
|---|---|---|
| neobot_run, neobot_run_stream(title,text,actorName,convoId,modelProvider,modelName) | spawn_blocking → run_local_turn(_stream)_as(Person) | nt_agent（缺省走晶体核心，见 core_engine） |
| neobot_agent_run(goal,context) | POST /v1/agents/run（token 现读） | nt_core（服务端循环回 trace） |
| neobot_core_status | 配对行 + 活探 + capabilities 版本/工具数 | nt_core |
| neobot_core_free, neobot_core_pair_free | 免费发现 / 一键配对（缺 key 拒绝） | nt_core |
| neobot_core_reload | POST /v1/admin/reload | nt_core |
| neobot_core_capabilities | GET /v1/capabilities 原始透传 | nt_core |
| neobot_convos, neobot_convo_group, neobot_convo_dm, neobot_convo_ensure_default, neobot_convo_rename, neobot_convo_delete, neobot_convo_add_member, neobot_convo_rm_member | 会话容器（DM/群组） | nt_store |
| neobot_convo_mute, neobot_convo_mark_read | 免打扰开关 / 已读水位 | nt_store |
| neobot_attach_add, neobot_attachments, neobot_attach_remove | 附件落盘 + 行（50MB 上限） | nt_store |
| neobot_tasks, neobot_task_claim, neobot_task_release, neobot_task_visibility, neobot_task_cancel, neobot_task_retry, neobot_task_rename, neobot_task_delete | 对应 store 原子方法 | nt_store |
| neobot_audit | list_audit(50)，明细不出前端 | nt_audit |
| neobot_cost_ledger, neobot_cost_ledger_by_actor | ledger 聚合 | nt_store |
| neobot_models | HttpEngine::for_listing + providers 聚合 | nt_http_engine + nt_provider |
| neobot_providers, neobot_provider_add, neobot_provider_remove, neobot_provider_toggle | providers 注册表（key 只存变量名；preset 经 CLI） | nt_provider + nt_store |
| neobot_routine_list, neobot_routine_fire, neobot_routine_sweep | store + fire/sweep_routines | nt_routine |
| neobot_skills | scan_skills（~/.neobot/skills） | nt_skills |
| neobot_memory_get, neobot_memory_set + MEMORY.md 注入 | 8K 上限 | nt_memory + nt_http_engine |
| neobot_members, neobot_member_add, neobot_member_remove | list/upsert/remove（首成员=owner） | nt_store |
| neobot_control_status, neobot_control_take, neobot_control_release | control 表 + 交接审计 | nt_store |
| neobot_roster_heartbeat, neobot_roster_list | PresenceMap 内存（45s/120s） | nt_commands |
| （CLI `export` 子命令；桌面备份走事件见 §4） | 单 zip 备份（db+附件+MEMORY+config+manifest） | nt_export |
| neobot_settings_window | 第二窗口 settings.html | tauri Manager |
| neobot_doctor | probe + 回收计数 | 各引擎 |

## 7. CLI 链（`neobot …`，bin/neobot.rs → 同库函数；map:check 同校验）

| 命令 | 落点 |
|---|---|
| init / doctor | 建目录+建表 / 自检 + 残留回收 |
| run（--engine/--provider/--model/--convo/--stream） | run_local_turn(_stream)_as（缺省走晶体核心） |
| task list, task claim, task release, task cancel, task retry, task rename, task rm | nt_store 任务原子方法 |
| audit list, audit prune | nt_store 审计读 + 留存清扫 |
| ledger（--by-actor） | nt_store 账本聚合 |
| routine list, routine add, routine fire, routine sweep, routine remove | nt_store + nt_routine（fire/sweep 经 DaemonGate 去抖，见 nt_daemon） |
| skill list, skill install, skill show | nt_skills（~/.neobot/skills） |
| memory set, memory get, memory clear | nt_memory（8K 上限） |
| member add, member list, member remove | nt_store 成员（删 owner 拒绝） |
| convo list, convo group, convo dm, convo rename, convo rm, convo members, convo add-member, convo rm-member | nt_store 会话容器 |
| attach add, attach list, attach rm | nt_store 附件（50MB 上限） |
| control take, control release, control status | nt_store 接管 + 交接审计 |
| policy drill | nt_policy 八项演练 |
| models（--provider） | 池子聚合（跳过不可达） |
| provider add, provider list, provider remove, provider on, provider off, provider preset | nt_provider + nt_store（key 只存变量名） |
| core pair, core pair-free, core free, core unpair, core status, core reload | nt_core 配对状态机（探活才写） |
| agent run | nt_core（POST /v1/agents/run，trace 回显） |
| export（--out） | nt_export 单 zip |

## 4. 事件链（主窗 ↔ 设置窗，localStorage 按 webview 隔离，必须走事件）

```
设置窗 → 主窗: neobot:theme{name,tokens} / neobot:prefs{五键} / prefs-query
          / backup-export/import/clear / set-me / theme-query
主窗 → 设置窗: neobot:theme-state / neobot:prefs-state / backup-data / backup-done
主题链: 设置选模板/调色/字体 → emit → 主窗 applyTheme（CSS 变量 + 果球体色）
        → 持久化 localStorage → 重启 applyTheme 先于挂球
```

## 5. 状态键（localStorage，主窗口命名空间）

```
ntos_threads_v1（按会话转录）/ ntos_drafts_v1（按会话草稿）/ mates / starters / me_v1
ntos_selconvo（选中会话）/ ntos_model（选中模型，缺省 neotrix-crystal）
ntos_seltask/selroutine/selskill_v1（选中）/ pinned_v1（置顶）/ boardfilter_v1（画板过滤）
ntos_pref_density/font/motion/msgwidth/face（外观）/ ntos_theme（主题名+tokens）
w_convs/w_info/info_open（列宽）
```

## 6. 设计语言（自研沉淀，不挂外部名）

| 界面元素 | 说明 |
|---|---|
| 三栏 + token 体系 | Rail/会话栏/thread/右 peek；CSS :root 与 theme.ts 双表同值 |
| 气泡/作曲/空态 | 扁平灰泡 + 白胶囊作曲 + 空态建议按钮 |
| 消息行 | tick/编辑/删除/复制/日期线 |
| 设置语言 | 无边框白卡 + hairline 行 + 10px 大写 label |
| 表情球 | 自研 emotionball（参数化脸 + 天气拟物 + 6 语义态 + 交互态 data-state） |
| 画布 | 无限视口 + 平移/缩放/节点拖拽 |
| 账本/记忆/Skills | 实测计价 + MEMORY.md 注入 + 技能指令参考 |
| 主题 | 七模板（默认/墨青/暖阳/樱粉/雾紫/夜蓝/抹茶）+ 七色微调 + 四字体 |
