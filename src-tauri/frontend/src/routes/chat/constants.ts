/**
 * routes/chat/constants.ts — Chat 页面常量与工具函数
 *
 * 从 Chat.tsx 提取：常量定义、dayLabel、formatTime、SUGGESTED_PROMPTS
 */
import type { PermissionMode } from '../../components/PermissionModeSelector'

/* 长消息内容折叠阈值（任务4：超长 assistant 消息折叠；末条/流式消息始终全量渲染，保证流式安全） */
export const LONG_MSG_FOLD_CHARS = 6000
export const LONG_MSG_SNIPPET_CHARS = 4000

/* 权限模式徽章短标签（对标 Claude 顶栏 mode 徽章） */
export const MODE_SHORT_LABEL: Record<PermissionMode, string> = {
  auto: '自动',
  manual: '手动',
  accept_edits: '接受编辑',
  plan: '规划',
}

/* 消息操作按钮 class */
export const actionBtnClass =
  'action-btn p-1.5 rounded-lg text-text-muted/70 hover:text-text-primary hover:bg-white/70 hover:shadow-sm transition-all duration-150'

// 消息流日期分隔：相邻消息跨日时插入「今天 / 昨天 / M月D日」分隔条
export function dayLabel(d: Date): string {
  const now = new Date()
  const start = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime()
  const diff = Math.round((start(now) - start(d)) / 86400000)
  if (diff === 0) return '今天'
  if (diff === 1) return '昨天'
  if (diff < 7) return `${diff} 天前`
  return `${d.getMonth() + 1}月${d.getDate()}日`
}

export function formatTime(date: Date): string {
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

// 空状态引导：首屏建议提示（点击填入输入框，用户可增删后发送）
export const SUGGESTED_PROMPTS: string[] = [
  '帮我规划一个新功能的实现方案',
  '审查当前会话的代码改动',
  '解释这段报错日志的根因',
  '搜索并总结相关开源仓库',
  '压缩会话上下文继续对话',
  '运行 Harness 任务：抓取并吸收一个仓库',
]
