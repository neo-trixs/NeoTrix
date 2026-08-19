/* ════════════════════════════════════════════
   routes/chat/slashCommands.ts — 斜杠命令逻辑（对标 Claude Code）
   从 Chat.tsx 抽出：/compact /model /status /cost /export /clear /new /help。
   纯逻辑模块（无 JSX），依赖注入 chatStore/neocodex/UI 反馈函数，
   可在无组件环境单测。UI 渲染与键盘导航仍在 Chat.tsx / SlashMenu。
   ════════════════════════════════════════════ */
import type { SlashCommandDef } from '../../components/SlashMenu'
import { neocodex } from '../../api'
import type { ChatStore } from '../../stores/chat'

/** 执行斜杠命令所需的外部依赖（由 Chat.tsx 提供） */
export interface SlashContext {
  store: ChatStore
  currentSessionId: () => string
  /** 命令执行后刷新输入区（清空输入 / 自动调整高度） */
  clearInput: () => void
  /** 用户通知（info 通道） */
  showInfo: (msg: string, ms?: number) => void
  /** 错误通知（error 通道，带 3s 自动消失） */
  showError: (msg: string) => void
}

/** 只读命令：当前激活模型（与状态栏同源 providerConfig） */
export async function runSlashModel(ctx: SlashContext): Promise<void> {
  try {
    const cfg = await neocodex.providerConfig()
    if (!cfg) {
      ctx.showInfo('暂无提供商配置', 3000)
      return
    }
    const model = cfg.active_model || '(未配置)'
    const resolvable = cfg.resolvable
    ctx.showInfo(`当前模型：${model}${resolvable ? '' : '（不可解析，请检查 API 配置）'} · 可用提供商 ${cfg.provider_count} 个`, 5000)
  } catch (error) {
    console.error('[Chat] /model failed:', error)
    ctx.showInfo('读取模型失败，请检查提供商配置', 3000)
  }
}

/** 只读命令：运行状态诊断（模型 / 上下文 / 用量 / 成本） */
export async function runSlashStatus(ctx: SlashContext): Promise<void> {
  try {
    const s = await neocodex.agentStatus()
    if (!s) {
      ctx.showInfo('无运行状态', 3000)
      return
    }
    const model = s.provider_model || '未知'
    const pct = Math.round((s.context_usage ?? 0) * 100)
    const tokens = (s.tokens_used ?? 0).toLocaleString()
    const cost = ((s.cost_spent ?? 0) / 1000).toFixed(3)
    const budget = ((s.cost_budget ?? 0) / 1000).toFixed(3)
    ctx.showInfo(`模型 ${model} · 上下文 ${pct}% · tokens ${tokens} · 成本 $${cost} / $${budget}`, 6000)
  } catch (error) {
    console.error('[Chat] /status failed:', error)
    ctx.showInfo('读取状态失败', 3000)
  }
}

/** 只读命令：token 用量与成本估算 */
export async function runSlashCost(ctx: SlashContext): Promise<void> {
  try {
    const s = await neocodex.agentStatus()
    if (!s) {
      ctx.showInfo('无用量数据', 3000)
      return
    }
    const tokens = (s.tokens_used ?? 0).toLocaleString()
    const cost = ((s.cost_spent ?? 0) / 1000).toFixed(3)
    const budget = ((s.cost_budget ?? 0) / 1000).toFixed(3)
    const pct = s.cost_budget ? `${Math.round(((s.cost_spent ?? 0) / s.cost_budget) * 100)}%` : '—'
    ctx.showInfo(`已用 ${tokens} tokens · 花费 $${cost} / $${budget}（${pct}）`, 5000)
  } catch (error) {
    console.error('[Chat] /cost failed:', error)
    ctx.showInfo('读取用量失败', 3000)
  }
}

/** 导出当前会话为 Markdown（浏览器下载） */
export async function runSlashExport(ctx: SlashContext): Promise<void> {
  const sessionId = ctx.currentSessionId()
  if (!sessionId) {
    ctx.showInfo('当前没有激活会话，无法导出', 3000)
    return
  }
  try {
    const content = await neocodex.exportSession(sessionId, 'markdown')
    if (content) {
      const name = ctx.store.currentSession?.title || 'session'
      const blob = new Blob([content], { type: 'text/markdown' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${name.replace(/[^\w\u4e00-\u9fff-]+/g, '_')}.md`
      a.click()
      URL.revokeObjectURL(url)
      ctx.showInfo('会话已导出为 Markdown', 3000)
    } else {
      ctx.showInfo('会话为空，无内容可导出', 3000)
    }
  } catch (error) {
    console.error('[Chat] /export failed:', error)
    ctx.showInfo('导出失败，请重试', 3000)
  }
}

/** 派发斜杠命令（runSlash 的纯逻辑版；UI 清除/菜单关闭由 Chat.tsx 处理） */
export function runSlashDispatch(ctx: SlashContext, cmd: SlashCommandDef): void {
  if (cmd.id === 'clear') {
    ctx.store.clearMessages()
    const sid = ctx.currentSessionId()
    if (sid) {
      // 后端落盘同步清除：仅清本地 store 会在会话重载后复活
      void neocodex.clearSession(sid).catch((error) => {
        console.error('[Chat] /clear failed:', error)
        ctx.showError('清除会话失败，请重试')
      })
    }
  } else if (cmd.id === 'new') {
    void ctx.store.addSession()
  } else if (cmd.id === 'model') {
    void runSlashModel(ctx)
  } else if (cmd.id === 'status') {
    void runSlashStatus(ctx)
  } else if (cmd.id === 'cost') {
    void runSlashCost(ctx)
  } else if (cmd.id === 'export') {
    void runSlashExport(ctx)
  } else if (cmd.id === 'help') {
    ctx.showInfo('快捷键：Enter 发送 · Shift+Enter 换行 · ⌘K 命令面板 · ⌘1-6 功能面板 · ⌘7 电脑视图 · ⌘N 新建对话 · Esc 关闭', 5000)
  }
}