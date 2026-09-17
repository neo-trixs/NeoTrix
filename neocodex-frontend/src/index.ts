/**
 * src/index.ts — FSD 架构中心枢纽
 * 导入所有 FSD 切片，确保每个切片至少被引用一次
 */
// Entities
import './entities/agent'
import './entities/domain'
import './entities/message'
import './entities/session'
import './entities/tool'

// Features
import './features/approval'
import './features/approve-tool'
import './features/edit-message'
import './features/search-knowledge'
import './features/send-message'
import './features/stream-response'
import './features/streaming'

// Widgets
import './widgets/canvas-viewer'
import './widgets/chat-panel'
import './widgets/right-sidebar'
import './widgets/tool-panel'

// Pages
import './pages/canvas'
import './pages/chat'
import './pages/knowledge'
import './pages/settings'
