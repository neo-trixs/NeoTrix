# preview-ui-v2.html 变更日志

## 2026-07-02 12:16 — 链可视化+拓扑地图+桌面宠物+响应式修复

- 变更:
  - 链可视化增强: SVG叠加脉冲连接线 (px-chain-svg), 节点状态名+延迟显示, 执行中节点高亮 (exec态+动画), fn_animateChain()函数
  - 拓扑提供者地图: 新增px-topo-wrap在World Map选项卡下, 显示Gemini/Anthropic/Groq/OpenRouter/Pollinations/Cerebras节点, 延迟指示器(绿/黄/红), 活动路由动画线, fn_renderTopoMap()+fn_topoAnimLoop()
  - 桌面宠物: 新增pet.mode人格模式(研究员/工程师/导师/创作者), 右键/点击身体外循环模式, 模态标识徽章, 思考气泡 (pet-speech) 显示当前思维状态, fn_petSpeechBubble()+fn_cyclePetMode()
  - 响应式修复: @media (max-width:1310px) 添加右面板溢出滚动, px/px-grid-2/cw-sidebar/cd-tree 窄屏适配
  - 新增CSS: .px-cldot.exec, @keyframes pxExec, .px-cline.active, @keyframes linePulse, .px-chain-svg + dashMove, .px-topo-*, .pet-speech, .pet-mode-badge
- 文件: preview-ui-v2.html (5810行, +284行)
- 原因: Cycle 10 前端迭代 — 链可视化增强、提供者拓扑地图、宠物人格系统、响应式修复
- 备份: preview-ui-v2.html.bak.20260702_121609

## 2026-07-02 11:35 — 创建设计框架

- 变更: 初始化 design/ 目录结构 + 备份规范
- 文件: preview-ui-v2.html (5526 行, 278KB)
- 备份: preview-ui-v2.html.bak.20260702_113507
- 原因: 建立前端设计规范化流程

## 2026-07-02 Cycle 7 — 前端组件补齐

- 变更:
  - clearChat() 函数
  - 键盘快捷键 (Cmd+, / Cmd+K / Esc / ?)
  - DOMContentLoaded 自动初始化
  - evo-card / onboard-card / oc-code / artifacts / art-link CSS
  - 项目面板 / 成果面板 / 计划任务 / 流程管线 / Agent历史 / 链运行
  - overlayProjects 通用面板
- 文件: preview-ui-v2.html
- 原因: Cycle 7 前端集成工作

## 2026-07-02 Cycle 9 — 桌面宠物渲染引擎升级

- 变更: +816 行 (4464→5280)
  - 面部引擎 (梯度皮肤/anime眼睛/睫毛/眼环辉光)
  - 发型系统 (drawHair + drawBangs, 流动发梢)
  - 身体细节 (梯度躯干/领口/胸标/腰带/手臂摆动/尾巴/浮空平台)
  - 配饰系统 (眼镜/耳机/皇冠光环)
  - 粒子引擎 (环境星点 + 点击爱心爆发)
  - 挤压/拉伸待机弹跳动画
  - IPC桥接 (REST API + EventSource + E8能量联动)
  - 人格系统 (研究员/工程师/导师/创作者)
- 文件: preview-ui-v2.html
- 原因: Cycle 9 桌面宠物 + 人格系统

## 2026-07-02 Cycle 6 — 约束解码集成 + 前端验证

- 变更: clearChat() / 键盘快捷键 / 各面板渲染 / simulateChainRun
- 文件: preview-ui-v2.html
- 原因: Cycle 6 前端集成工作
