# NeoBot — 独立本地 agent App

> 基于 `neotrix-neobot` 构建的独立 App：桌面端 + CLI 同源（`~/.neobot/neobot.db`），
> 默认零网络/零 Docker/零云端。体验取 ntos 四件：任务中心 / 队友 roster+@全体 /
> 提示词库 / 备份三段式。

- 产品名 `NeoBot`，bundle id `ai.neobot.desktop`（见 `tauri.conf.json`）
- 后端 `src/` 瘦壳：只依赖 `neotrix-neobot`，**97 个注册 IPC** 全是薄封装（`nt_commands.rs`），
  不链接 `neotrix` 大内核
  （注册数 = `main.rs` 的 `generate_handler!` 项数；已与 `#[tauri::command]` 声明集逐名比对，
  97 = 97 —— 漏注册过一次，见下方「自测」）
- 前端 `frontend/`：转录体会话 + hero 卡（三选一）+ 作曲区（发送/停止同键+草稿保持）
  + roster 心跳 + 成本账 + 模型池 + 设置页备份
  + **工作台右列**（文件树 / 编辑器 / 变动双视角 / 任务 / 侧聊，零依赖手写
  Myers diff + 字符级高亮）
  + **IM 渠道页**（Telegram 等；token 只存环境变量名，永不落库）
- capabilities 只放行 `$HOME/.neobot/**` 与 `$TEMP/**`（比主 App `$HOME/**` 更紧，shield 验收点）：
  工作台的文件访问**不走** `tauri-plugin-fs`，而走自有 `neobot_fs_*` 命令 + Rust 侧
  两道 jail（词法段判定 + `canonicalize` 真实路径核验），比 ACL 更严且不动验收点

## 工作台（右列，左轨 📁 按钮开合）

| 页签 | 干什么 | 落地 |
|---|---|---|
| 文件 | **懒加载递归树**（键盘可达，展开目录自动重列）/ 全局搜索 / `@路径` 引用 / 可保存编辑器 | `nt_workspace` + `neobot_fs_*` + `tree.ts` |
| 变动 | **本轮文件**（这一轮模型碰了什么）+ **Git 视角**（仓库欠着什么） | `file_changes` 表 + `nt_git` |
| 任务 | 任务终态 + 每轮碰过的文件 | 既有 `tasks` + `file_changes` |
| 侧聊 | 继承母会话摘要、独立运行、可升格为顶层会话 | `nt_side_chat` |

模型的 `sidebar_open` 工具会「提议」打开某文件/页签 —— **只出指令、由界面执行**，
且提议路径过 jail（开不出工作区之外的东西）。见
`docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md`。

## IM 渠道

设置页 → 「IM 机器人」。已编译进来的渠道：**Telegram**（长轮询，零新依赖）。
桌面里的「收一轮」是**手动**触发的：App 关掉就不该有后台长轮询在偷偷连公网。
要一直在线就另开终端跑 `neobot channel serve`（每轮重读配置，停用/改访问模式不必重启进程）。

- token **只从环境变量读**（如 `NEOBOT_TELEGRAM_TOKEN`），库里只存变量名
- 访问模式：`open` / `allow`（白名单） / `dm_only`；**认不出的值一律归 `allow`（全拒）**
- 闸门在指令之前：陌生人发 `/new` 不会有任何效果
- 指令：`/new` `/new 标题` `/help` `/status` 都可用；**`/stop` 当前不可用** ——
  跑轮是同步的，这一轮跑完之前不会收到新消息（回执会直说，不会谎称「已停止」）。
  这一轮无法中途打断（App 内的停止键只收起输出、不终止运行；要真的终止得退出 App）。
- 入站附件会落到 `~/.neobot/attachments` 并登记到会话（50 MB 上限）；
  文本类文件模型能用 `read_file` 真读到。**出站**（模型 → 用户）还没做，适配器会明确报错

```sh
neobot channel list      # 渠道与机器人（token 只出变量名）
neobot channel probe telegram
neobot channel once      # 收一轮
neobot channel serve     # 常驻长轮询（Ctrl-C 退出）
```

加新渠道 = 实现 `ChannelAdapter` + 在 `nt_cmd_channels::registry()` 加一行，不改调用方。

## 自测

```sh
cd apps/neobot-desktop/frontend && npm run selftest   # diff / highlight / tree 不变式
cargo test -p neotrix-neobot --lib                     # 254 个（2026-09-28 实测）
```

**这些数字只覆盖一层。** `apps/neobot-desktop` 自己的 IPC 层**零 `#[cfg(test)]`**；
侧边栏的渲染函数在本批开工时困在 import 了 Tauri API 的文件里、因而**无法被 `selftest`
引用**（正在抽成零依赖的 `frontend/src/render.ts`）；Telegram 的 23 个测试全是手写 JSON
夹具、**从未打过 `api.telegram.org`**。
本工作台 + IM 链**从未对着真浏览器或真平台端到端跑过**。缺口明细与正在补的批次见
`docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md`「第四轮：验证缺口批」。

加过一条命令时记得：`#[tauri::command]` 声明了不等于前端能调 —— **必须**同时进
`main.rs` 的 `generate_handler!`。漏过一次（42 个工作台/IM 命令全写好但没注册，
前端一个都调不到，而编译与测试全绿）。

## 跑起来

```sh
# 1) CLI（与桌面同源）
bash apps/neobot-desktop/scripts/install-cli.sh

# 2) 桌面 dev（需 Node 18+；Tauri 工具链）
cd apps/neobot-desktop/frontend && npm ci && npm run dev   # vite :1422
cargo run -p neobot-desktop                               # 另起终端

# 3) 打包（dmg；图标已齐，B1 门过）
cd apps/neobot-desktop && cargo tauri build
```

## 环境变量（与 CLI 同名）

| 变量 | 含义 |
|---|---|
| `NEOBOT_DATA_DIR` | 数据目录（默认 `~/.neobot`） |
| `NEOBOT_POLICY` | `enforce` / `dry_run` |
| `NEOBOT_ENGINE` | `echo` / 本机命令名（如 `claude`） / `http` |
| `NEOBOT_*` (`BASE_URL`/`API_KEY`/`MODEL`) | `http` 引擎端点（key 永不落盘，见 `nt_http_engine.rs`） |

## 与主 App 关系

主 App（`src-tauri`，`ai.neotrix.desktop`）内 `commands/neobot.rs` 是同一核心的
 embed 版；本目录是**可独立分发**的瘦壳。两者共享 `~/.neobot`，可并用；
 capabilities/窗口/更新通道互相隔离。
