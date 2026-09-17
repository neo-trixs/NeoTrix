// ══════════════════════════════════════════════════════════════════════════
//  Settings — 独立路由页面（从 SettingsModal 复用，支持 ⌘, 快捷键直达）
// ══════════════════════════════════════════════════════════════════════════
import { SettingsModal } from '../components/SettingsModal'

export function Settings() {
  return (
    <div class="h-screen w-screen overflow-hidden bg-bg-primary">
      <SettingsModal open onClose={() => history.back()} />
    </div>
  )
}
