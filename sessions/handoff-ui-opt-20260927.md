# 通用 UI 优化建议全集（2026-09-27，调研＋对标＋落地状态）

> 对标：微信（绿泡/已读/引用/九宫格）／WhatsApp（ticks/壁纸/16:9）／Telegram（液态玻璃/单实例/云草稿）／
> Claude（扁平/轮播对比）／Codex（转录+active_cell）／飞书卡片／抖音信息流／shadcn chat／Ant X。
> 状态：P0 已落地（本文件§1），P1 排队（§2），park（§3）。

## §1 已落地 P0（本轮＋前轮，可验）

- 顶栏：Telegram 液态玻璃悬浮居中（标题绝对居中＋省略，左右留头像/操作）。
- 动作行泡下常驻（复制/引用/重跑/编辑/删除，用户右对方左）。
- 气泡主题三选（雪域/微信绿/WhatsApp）＋深色（浅/深/跟随）＋发送键（Enter/Cmd+Enter）。
- 空态极简（一行问候，chips 下线可回）。
- 消息标签 chips（模型/模式/工具/tokens）＋引用上标＋长文折叠＋代码复制。
- 媒体：图懒加载＋视频 16:9 卡（metadata 预载）＋音频条形器。
- 稳定性：关窗 Hide＋ExitRequested 防退；stale running 复位。

## §2 P1 排队（spec 就绪，未开工）

- 流式 caret＋stopped/error 四态＋首字 600ms。
- 虚拟滚动＋跟随模式＋`回到底部` pill（万条级）。
- 引用悬停源卡（标题/域名/新鲜度）＋追问 chips（底部=下一步）。
- 工具调用折叠卡（已搜索/已读 N 文件，详情按需展）。
- 分支对比（edit-fork/rewind/双答案）。
- 反馈三层（赞踩→分类→文本）＋aria-live＋减弱动效跟随系统。

## §3 park（需产品/专窗）

- 一仓双 Tauri 二选一；L0 正典收敛；God 续拆（proxy/file_ability/unify/pdf）；
  worktree 5 窗内容；serve 认领；entry 死码；桌面大部队（src-tauri 248）。
