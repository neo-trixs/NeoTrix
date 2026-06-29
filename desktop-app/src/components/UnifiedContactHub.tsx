import { useState, useEffect } from 'react';
import { layoutEngine } from '../core/layout-engine';
import { socialIdentity, vsaFromPubkey } from '../core/social-identity';
import { chatStore } from '../core/chat-store';
import { socialTasks } from '../core/social-tasks';

interface UnifiedContact {
  id: string;
  name: string;
  type: 'agent' | 'person';
  status: string;
  unread: number;
  online?: boolean;
  parentId?: string;
}

const STATUS_COLORS: Record<string, string> = {
  idle: '#666', running: '#48b87a', paused: '#e8a030', done: '#5e8aff', error: '#e04e4e',
};
const TYPE_ICONS: Record<string, string> = { agent: '⚡', person: '👤' };

export default function UnifiedContactHub() {
  const [filter, setFilter] = useState<'all' | 'agents' | 'people'>('all');
  const [search, setSearch] = useState('');
  const [contacts, setContacts] = useState<UnifiedContact[]>([]);
  const [activeChat, setActiveChat] = useState(chatStore.activeId);
  const [taskCount, setTaskCount] = useState(0);

  useEffect(() => {
    const rebuild = () => {
      const list: UnifiedContact[] = [];
      const layout = layoutEngine.getState();
      for (const agent of layout.contacts) {
        list.push({
          id: `agent:${agent.id}`,
          name: agent.name,
          type: 'agent',
          status: agent.status ?? 'idle',
          unread: 0,
          parentId: agent.parentId,
        });
      }
      for (const c of socialIdentity.contacts) {
        const contact = socialIdentity.getContact(c.pubkey);
        list.push({
          id: `person:${c.pubkey}`,
          name: c.alias,
          type: 'person',
          status: contact?.online ? 'online' : 'offline',
          unread: c.unread ?? 0,
          online: contact?.online,
        });
      }
      setContacts(list);
      setTaskCount(socialTasks.getOpenTasks().length);
    };

    rebuild();
    const unsub = chatStore.subscribe(() => setActiveChat(chatStore.activeId));
    const iv = setInterval(rebuild, 3000);
    return () => { clearInterval(iv); unsub(); };
  }, []);

  const filtered = contacts.filter(c => {
    if (filter === 'agents' && c.type !== 'agent') return false;
    if (filter === 'people' && c.type !== 'person') return false;
    if (search && !c.name.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  // 按 parentId 分组显示父子关系
  const parents = filtered.filter(c => !c.parentId);
  const children = filtered.filter(c => c.parentId);

  const renderContact = (c: UnifiedContact) => (
    <div key={c.id}
      className={`ucontact-item ${activeChat === `dm:${c.id}` ? 'active' : ''}`}
      data-ai-role="contact-item" data-ai-contact-id={c.id}
      onClick={() => {
        if (c.type === 'person') {
          chatStore.openDM(c.id.replace(/^(person|agent):/, ''), c.name, c.type);
        } else {
          chatStore.openDM(c.id.replace(/^(person|agent):/, ''), c.name, 'agent');
        }
      }}>
      <div className="ucontact-icon">{TYPE_ICONS[c.type]}</div>
      <div className="ucontact-body">
        <div className="ucontact-name-row">
          <span className="ucontact-name">{c.name}</span>
          <span className={`ucontact-status-dot ${c.type === 'person' ? 'online' : c.status}`}
            style={{ background: c.type === 'person' ? (c.online ? '#48b87a' : '#5a5a66') : (STATUS_COLORS[c.status] ?? '#666') }} />
        </div>
        <div className="ucontact-meta">
          <span className="ucontact-type-tag">{c.type}</span>
          <span className="ucontact-status-text">{c.type === 'person' ? (c.online ? 'Online' : 'Offline') : c.status}</span>
          {c.type === 'person' && (
            <span className="ucontact-vsa-tag" title={`VSA: ${vsaFromPubkey(c.id.replace(/^person:/, ''))}`}>
              VSA:{vsaFromPubkey(c.id.replace(/^person:/, '')).slice(0, 6)}
            </span>
          )}
        </div>
      </div>
      {c.unread > 0 && <span className="ucontact-badge">{c.unread}</span>}
    </div>
  );

  return (
    <div className="ucontact">
      {/* Filter tabs */}
      <div className="ucontact-filters" data-ai-role="filter-tabs">
        {(['all', 'agents', 'people'] as const).map(f => (
          <button key={f} className={`ucontact-filter ${filter === f ? 'active' : ''}`}
            onClick={() => setFilter(f)}>
            {f === 'all' ? 'All' : f === 'agents' ? 'Agents' : 'People'}
          </button>
        ))}
      </div>

      {/* Search */}
      <input className="ucontact-search" placeholder="Search contacts..."
        value={search} onChange={e => setSearch(e.target.value)} data-ai-role="search" />

      {/* Identity card */}
      <div className="ucontact-identity" data-ai-role="identity-card" title={socialIdentity.vsaId}>
        <span className="ucontact-id-dot" />
        <span className="ucontact-id-name">{socialIdentity.profile?.name ?? 'NeoTrix'}</span>
        <span className="ucontact-id-key">{socialIdentity.shortId.slice(0, 10)}</span>
        <span className="ucontact-id-vsa" title="VSA 4096-bit identity (derived from Ed25519)">
          VSA:{socialIdentity.vsaId.slice(0, 8)}
        </span>
      </div>

      {/* Contact list */}
      <div className="ucontact-list" data-ai-role="contact-list">
        {parents.map(renderContact)}
        {children.length > 0 && (
          <>
            <div className="ucontact-label">Sub-agents</div>
            {children.map(renderContact)}
          </>
        )}
        {filtered.length === 0 && (
          <div className="ucontact-empty">
            <div className="ucontact-empty-text">No contacts found</div>
            <div className="ucontact-empty-hint">
              {filter === 'agents' ? 'Agents appear when running' : 'Add contacts via pubkey'}
            </div>
          </div>
        )}
      </div>

      {/* Footer actions */}
      <div className="ucontact-footer">
        <button className="ucontact-action" onClick={() => {
          const pk = prompt('Enter pubkey:');
          if (pk) {
            const name = prompt('Alias:') || `User_${pk.slice(0, 8)}`;
            socialIdentity.addContact({ pubkey: pk, alias: name });
          }
        }}>
          + Add Person
        </button>
        <div className="ucontact-task-section">
          <div className="ucontact-task-header">{taskCount} open tasks</div>
          {taskCount > 0 && (
            <button className="ucontact-action" onClick={() => {
              if (socialTasks.getOpenTasks().length > 0) {
                const task = socialTasks.getOpenTasks()[0];
                socialTasks.claimTask(task.id);
                setTaskCount(socialTasks.getOpenTasks().length);
              }
            }}>
              ⚡ Claim nearest task
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
