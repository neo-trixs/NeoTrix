# OpenIM 吸收（2026-09-25）：会话能力 + UI 交互

源：`https://github.com/openimsdk/open-im-server`（Go IM 服务端，快照 `/…/T/opencode/open-im-server`）。
只吸**会话语义与交互机制**，不吸服务端架构（Mongo/Redis/Kafka 与本地单文件 SQLite 目标相悖）。

## 映射（OpenIM → neobot）

| OpenIM 机制 | 位置 | neobot 落点 | 取舍 |
|---|---|---|---|
| 已读水位 `hasReadSeq` + 未读数 `maxSeq-hasRead` | `rpc/msg/as_read.go` | `read_marks(convo_id,last_read_at)`；`unread=COUNT(tasks.created_at>水位)` | seq→时间戳（本地无 seq 层，够用）；默认水位=会话创建时间，存量不炸未读 |
| 免打扰 `recvMsgOpt` | `conversation.go:SetConversation` | `conversations.muted`；免打扰会话不亮未读角标 | OpenIM 免打扰仍计未读但不推送；本地无推送通道，取更严的“不计数”，诚实 |
| 置顶 `isPinned` | 同上 | 已有 localStorage 置顶，不动（单机等价） | 服务端置顶无意义，不迁库 |
| 按会话草稿 `draftText` | SDK `ConversationInfo.draftText` | `K.drafts: Record<convoId,text>`；切会话先存后取 | 原全局单草稿是 IM 反模式，必须改 |
| 清空记录 `ClearUserConversationMsg` | `rpc/msg/delete.go` | 仅清**本地转录**（localStorage 该会话线程），任务/账本不动 | 服务端删消息是多端同步语义；本地删任务=改写历史，禁止 |
| 撤回 `RevokeMsg`（限时） | `rpc/msg/revoke.go` | 不做：本地转录删除即撤回，已有；限时窗口无对手端，无意义 | — |
| 正在输入 `Typing` | `rpc/msg/verify.go:Typing` | 不做：无对手客户端协议，靠心跳猜输入=造假 | — |
| 阅后即焚 `destructTime` | `conversation.go` | 不做：本地单机无泄密通道，属伪需求 | — |
| 消息搜索服务端版 | `rpc/msg` | 已有全局搜索覆盖，不动 | — |

## 语义诚实声明
- `unread` = 该会话**水位之后新建的任务数**（routine  firing / 他会话后台完成是主要来源）；打开会话即清零（`mark_read`），本轮完成后也清（自己刚看过的不算未读）。
- `muted` 会话的 unread 不计入角标、不触发完成 toast（当前会话除外：正看着的照常提示）。
