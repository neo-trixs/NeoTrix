import { clsx } from 'clsx'

export type RbTab = 'files' | 'map' | 'project' | 'canvas' | 'agents'

export const RB_TABS: RbTab[] = ['files', 'map', 'project', 'canvas', 'agents']

interface RightBarTabsProps {
  rbTab: RbTab
  onSetTab: (tab: RbTab) => void
}

function TabIcon({ tab }: { tab: RbTab }) {
  switch (tab) {
    case 'files':
      return (
        <svg viewBox="0 0 14 14" class="rb-tab-ic">
          <path d="M2 1.5h10v11H2z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round" />
          <line x1="4.5" y1="4.5" x2="9.5" y2="4.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
        </svg>
      )
    case 'map':
      return (
        <svg viewBox="0 0 14 14" class="rb-tab-ic">
          <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.1" fill="none" />
          <ellipse cx="7" cy="7" rx="2.6" ry="5.5" stroke="currentColor" stroke-width="1.1" fill="none" />
          <line x1="1.5" y1="7" x2="12.5" y2="7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
        </svg>
      )
    case 'project':
      return (
        <svg viewBox="0 0 14 14" class="rb-tab-ic">
          <path d="M1.5 2.5h4.5a1.5 1.5 0 011.5 1.5v7.5a1.5 1.5 0 00-1.5-1.5H1.5z" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linejoin="round" />
          <path d="M12.5 2.5H8a1.5 1.5 0 00-1.5 1.5v7.5a1.5 1.5 0 011.5-1.5h4.5z" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linejoin="round" />
        </svg>
      )
    case 'canvas':
      return (
        <svg viewBox="0 0 14 14" class="rb-tab-ic">
          <rect x="1.5" y="1.5" width="4.5" height="4.5" rx="1" stroke="currentColor" stroke-width="1.1" fill="none" />
          <rect x="8" y="1.5" width="4.5" height="4.5" rx="1" stroke="currentColor" stroke-width="1.1" fill="none" />
          <rect x="1.5" y="8" width="4.5" height="4.5" rx="1" stroke="currentColor" stroke-width="1.1" fill="none" />
          <rect x="8" y="8" width="4.5" height="4.5" rx="1" stroke="currentColor" stroke-width="1.1" fill="none" />
        </svg>
      )
    case 'agents':
      return (
        <svg viewBox="0 0 14 14" class="rb-tab-ic">
          <circle cx="7" cy="4" r="2.5" stroke="currentColor" stroke-width="1.1" fill="none" />
          <path d="M2.5 12.5c0-2.5 2-4.5 4.5-4.5s4.5 2 4.5 4.5" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round" />
        </svg>
      )
  }
}

const TAB_LABELS: Record<RbTab, string> = {
  files: '文件',
  map: '地图',
  project: '项目',
  canvas: '画板',
  agents: '协调',
}

export function RightBarTabs(props: RightBarTabsProps) {
  const moveRbTab = (dir: 1 | -1) => {
    props.onSetTab(RB_TABS[(RB_TABS.indexOf(props.rbTab) + dir + RB_TABS.length) % RB_TABS.length])
  }

  const tabKeyDown = (e: KeyboardEvent) => {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
    e.preventDefault()
    moveRbTab(e.key === 'ArrowRight' ? 1 : -1)
    requestAnimationFrame(() => {
      const tabs = document.querySelectorAll<HTMLElement>('.rb-tabs [role="tab"]')
      tabs[RB_TABS.indexOf(props.rbTab)]?.focus()
    })
  }

  return (
    <div class="rb-tabs" role="tablist" aria-label="右栏视图">
      <For each={RB_TABS}>
        {(tab) => (
          <button
            class={clsx('rb-tab', props.rbTab === tab && 'on')}
            onClick={() => props.onSetTab(tab)}
            role="tab"
            aria-selected={props.rbTab === tab}
            tabIndex={props.rbTab === tab ? 0 : -1}
            onKeyDown={tabKeyDown}
          >
            <TabIcon tab={tab} />
            {TAB_LABELS[tab]}
          </button>
        )}
      </For>
    </div>
  )
}

import { For } from 'solid-js'
