# ntos × 模型内核（2026-09-22 → 0.2.0 修订）

> ntos＝独立 AI 应用（Lingee 式布局，`ntos/`），**唯一外部依赖＝第三方大模型
> （OpenAI 兼容 `/v1/chat/completions`，配置存本机 localStorage）**。
> 修订：去 neotrix 内核依赖（ntcode/会话列表探测仅作 Tauri 宿主兼容保留）；
> 未配置模型时诚实引导，不做死按钮。

## 1. 壳结构（`ntos/src/`）

```
ntos/
├── package.json            # solid + @tauri-apps/api + clsx（72 包，dist 25KB）
├── src/api/domain.ts       # 内核 domain_call 客户端（主应用同源拷贝）
├── src/api/ntcode.ts       # ntcode 9 命令＋4 事件（主应用同源拷贝）
├── src/api/tauri-bridge.ts # invoke 封装
├── src/lib/env.ts          # isTauriRuntime 探测
├── src/shell/Sidebar.tsx   # 侧栏（sidebar_tree.json 实测复刻）
├── src/shell/Composer.tsx  # 作曲区（shell_full.png 中央复刻）
├── src/shell/modelTiers.ts # 11 档元数据（主应用同源拷贝）
├── src/App.tsx             # 视图路由＋内核接线＋降级
└── preview :4174
```

## 2. 内核接口表（壳→核，全部已接线）

| 壳能力 | 内核调用 | 降级 |
|---|---|---|
| 会话列表 | `domain.call('session','list')` | 本地欢迎会话 |
| 发送消息 | `ntcode.sendNtcodeMessage`＋`listenNtcodeEvents` 流式拼装 | 本地回显（注明未连内核） |
| 模型池 | `ntcode.getNtcodeModels` | 空态＋"连接内核后展示" |
| 档位 | localStorage `nt_model_tier`（与主应用同键，Tauri 同源共享） | 默认 auto |
| 定时任务 | notice（面板随内核版本上线） | toast 注明 |
| 附件／智能体选择 | notice | toast 注明 |

## 3. 与主应用的关系

- 主应用（`src-tauri/frontend`）＝全功能控制台；ntos＝轻量对话壳。共享：IPC 契约、11 档元数据、tag-pill/token 语言。
- 冲突面：`nt_model_tier` 同键（有意：同桌面同偏好）；`session/list` 与主应用会话体系同源（ntcode conversations 另体系，切换时注意）。

## 4. 下一步（需 Rust 窗口）

1. Tauri 第二窗口 `ntos`（或独立 app crate 依赖 neotrix core lib）＋窗口透明/圆角复用现有配置。
2. `business_type` 透传（pills 已就位，等 `chat.send` 加字段）。
3. `usage_report` action（用量三处回填）。
4. 定时任务面板移植（`ScheduledTasks` 组件已有，搬壳即用）。
