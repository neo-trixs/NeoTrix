/* ════════════════════════════════════════════
   DeepLinkBridge — neotrix://page/<name> → 前端路由 (Phase 4)
   后端 on_open_url 转发 'neotrix-navigate' 事件, 此桥导航。
   必须渲染在 Router 内; 非 Tauri 环境 (冒烟测试) 安全降级。
   ════════════════════════════════════════════ */
import { onMount, onCleanup } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { listen } from '@tauri-apps/api/event'

export function DeepLinkBridge() {
  const navigate = useNavigate()
  onMount(() => {
    let un: (() => void) | null = null
    try {
      listen<string>('neotrix-navigate', (e) => navigate(e.payload))
        .then((f) => { un = f })
        .catch(() => undefined)
    } catch {
      /* 非 Tauri 环境 */
    }
    onCleanup(() => un?.())
  })
  return null
}
