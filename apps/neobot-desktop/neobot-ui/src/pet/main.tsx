/** 桌宠页入口（`pet.html` 的挂载点）。⛔ 不套 <Shell>：桌宠窗要的是透明无边框。 */
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import { Pet } from './pet'

const host = document.getElementById('root')
if (host) {
  createRoot(host).render(
    <StrictMode>
      <Pet />
    </StrictMode>,
  )
}
