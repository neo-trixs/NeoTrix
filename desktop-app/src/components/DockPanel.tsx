import { useState, type ReactNode } from 'react';

export interface DockItem {
  id: string;
  label: string;
  icon?: ReactNode;
  panel: ReactNode;
  defaultOpen?: boolean;
}

interface DockPanelProps {
  items: DockItem[];
  side?: 'left' | 'right';
}

export default function DockPanel({ items, side = 'right' }: DockPanelProps) {
  const [openTabs, setOpenTabs] = useState<string[]>(
    () => items.filter(i => i.defaultOpen).map(i => i.id)
  );
  const [activeTab, setActiveTab] = useState<string | null>(
    () => items.find(i => i.defaultOpen)?.id || null
  );

  function toggleTab(id: string) {
    setOpenTabs(prev => {
      if (prev.includes(id)) {
        const next = prev.filter(t => t !== id);
        if (activeTab === id) setActiveTab(next[0] || null);
        return next;
      }
      setActiveTab(id);
      return [...prev, id];
    });
  }

  function closeTab(id: string) {
    setOpenTabs(prev => {
      const next = prev.filter(t => t !== id);
      if (activeTab === id) setActiveTab(next[0] || null);
      return next;
    });
  }

  return (
    <div className={`dock-panel dock-panel-${side}`}>
      {/* Tab bar */}
      <div className="dock-tab-bar">
        {items.map(item => {
          const isOpen = openTabs.includes(item.id);
          const isActive = activeTab === item.id;
          return (
            <div
              key={item.id}
              className={`dock-tab ${isOpen ? 'open' : ''} ${isActive ? 'active' : ''}`}
              onClick={() => isOpen ? setActiveTab(item.id) : toggleTab(item.id)}
            >
              {item.icon && <span className="dock-tab-icon">{item.icon}</span>}
              <span className="dock-tab-label">{item.label}</span>
              {isOpen && (
                <button className="dock-tab-close" onClick={e => { e.stopPropagation(); closeTab(item.id); }}>
                  <svg width="7" height="7" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </button>
              )}
            </div>
          );
        })}
      </div>

      {/* Active panel */}
      {activeTab && openTabs.includes(activeTab) && (
        <div className="dock-panel-content">
          {items.find(i => i.id === activeTab)?.panel}
        </div>
      )}
    </div>
  );
}
