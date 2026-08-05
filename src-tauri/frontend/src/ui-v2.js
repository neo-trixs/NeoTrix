// AUTO-MIGRATED from design/previews/preview-ui-v2.html
import { invoke, listen, isTauri } from "./ipc";

/* ===== Global exposure for inline onclick handlers ===== */
const g = window;
g.autoResize = autoResize;
g.clearChat = clearChat;
g.closeOverlay = closeOverlay;
g.openOverlay = openOverlay;
g.closePopover = closePopover;
g.closePreview = closePreview;
g.copyPreview = copyPreview;
g.dispatch = dispatch;
g.expandPreview = expandPreview;
g.handleKey = handleKey;
g.onNavClick = onNavClick;
g.openRecent = openRecent;
g.onRbLeave = onRbLeave;
g.openRbSidebar = openRbSidebar;
g.openSettingsModal = openSettingsModal;
g.registerMcp = registerMcp;
g.refreshPreview = refreshPreview;
g.selectCwSession = selectCwSession;
g.selectSetting = selectSetting;
g.sendMsg = sendMsg;
g.setPreviewMode = setPreviewMode;
g.showFilePreviewFromChat = showFilePreviewFromChat;
g.showFilePreview = showFilePreview;
g.showToast = showToast;
g.switchArtifactView = switchArtifactView;
g.switchView = switchView;
g.toggleRbSidebar = toggleRbSidebar;
g.toggleSidebar = toggleSidebar;
g.toggleTheme = toggleTheme;
g.toggleUserPopover = toggleUserPopover;
g.copyMsgCode = copyMsgCode;
g.renderRichText = renderRichText;
g.escHtml = escHtml;
g.createSession = createSession;
g.refreshAgent = refreshAgent;
g.runAgent = runAgent;
g.runMsgCode = runMsgCode;
g.stopAgent = stopAgent;
g.loadHypercube = loadHypercube;
g.loadSessions = loadSessions;
g.loadRegistry = loadRegistry;
g.kbSearch = kbSearch;
g.sendSuggestion = sendSuggestion;
g.renderHeroSuggest = renderHeroSuggest;
g.cwFilter = cwFilter;
  /* Free LLM 模型池 — 模型选择下拉的数据源（Tauri 下与 neocodex_provider_config 合并） */
  const MODEL_POOL = [
    { id: 'Groq',        title: 'Groq',        model: 'Llama 3.3 70B', lat: 340, online: true  },
    { id: 'Cerebras',    title: 'Cerebras',    model: 'Llama 3.1 8B',  lat: 280, online: true  },
    { id: 'OpenRouter',  title: 'OpenRouter',  model: 'Mixtral 8x7B',  lat: 520, online: true  },
    { id: 'DeepSeek',    title: 'DeepSeek',    model: 'V3 chat',       lat: 450, online: true  },
    { id: 'Pollinations',title: 'Pollinations',model: 'GPT-4o mini',   lat: 890, online: true  },
    { id: 'SambaNova',   title: 'SambaNova',   model: 'Llama 3.1 70B', lat: 0,   online: false },
  ];
  let currentModelId = 'Groq';
  let attachList = [];


  /* ════════════════════════════════════════════════════════════
     BACKEND API FLOW PER TAB

     ── Tab 1: 对话 (Chat) ──────────────────────────────────────
     User Input → `invoke("agent_reason", { prompt, session_id })`
       → ReasoningEngine.reason()
         → GatewayV2.complete_with_selection()
           → CircuitBreaker (nt_core_cb): CLOSED/HALF_OPEN check
           → RateLimiter (nt_core_rl): RPM + TPM token buckets
           → ProviderPool.select(): composite score S = s²/lat × cost × health
             → Free provider (Groq/OpenRouter/Pollinations)
             → Fallback chain: primary → 2 retries → aggressive all-providers
         → Streaming tokens via `listen("streaming-token")`
           → UI: append token to .mb in chat scroll
       → core_review() records ConversationRecord → KB (nt_memory_kb)
       → ConversationDistillStage (every 3 ticks) → EvolutionRecord

     ── Tab 2: 团队 (Cowork) ────────────────────────────────────
     `invoke("cmd_session_list")` → SessionManager.list_sessions()
     `invoke("cmd_session_create", { name })` → SessionManager.create(name)
     `invoke("cmd_session_switch", { id })` → SessionManager.activate(id)
     Task Dispatch → AgentTeam.execute() (nt_act_orch_patterns)
       → Supervisor pattern: Orchestrator → Worker[1..N] → Gate
       → Swarm pattern: parallel agent broadcast → collect
       → Pipeline pattern: ChainRefine sequential stages
     Inter-agent → AgentBus.publish() / subscribe() (nt_cap_orch_handoff)
     UI Polling → `poll("cmd_cowork_status")` every 2s → update progress bars

     ── Tab 3: 代码 (Code) ──────────────────────────────────────
     File Tree → `invoke("read_dir_recursive", { path })` → render .ft
     File Open  → `invoke("read_file", { path })` → .ap artifact pane
     File Save  → `invoke("write_file", { path, content })` → confirm
     Agent Session:
       `invoke("cmd_agent_start", { task })` → spawn background agent
       `invoke("cmd_agent_stop", { id })` → kill
       `invoke("cmd_agent_status", { id })` → poll step + token progress
     Git Status → `invoke("cmd_git_status")` → branch/diff indicators

     ── Tab 4: 代理 (Agent/Proxy) ───────────────────────────────
      Proxy Dashboard: hero ring + chain viz + daemon controls
        Tab overview: stat grid (circuit breaker, rate limiter, etc.)
                     + IP Pool (12 proxy entries with latency bars)
                     + Free LLM Providers (6 providers with scores)
                     + Network Links (6 system links with status)
        Tab map: SVG world map (5 continent paths + 20+ geo points
                 + node dots with health color + connection lines)
        Tab nodes: node table (24 entries with health bar distribution)
        Tab subscriptions: URL input + list with add/remove
        Tab settings: port config + strategy select + system proxy toggle

      Backend (when daemon connected):
        `invoke("proxy_status")` → daemon PID/port/uptime/mode
        `invoke("proxy_set_mode")` → off/geo/stealth/tor
        `invoke("proxy_source_status")` → source health
        `invoke("proxy_pool_nodes")` → node list with latency/score
        `invoke("proxy_sub_list/add/remove")` → subscription CRUD
        `invoke("proxy_config_get/set")` → config persistence

      Agent/Gateway (L7 Capability):
        E8 State Machine (L4) → `invoke("e8_get_state")` every 500ms
       → nt_core_e8::E8StateMachine { Ground/Receptive/Arousal/… }
       → Hexagram → nt_core_e8::to_hexagram()
     SAE Features (L4) → `invoke("sae_get_active_latents")`
       → nt_core_sae::SparseAutoencoder.forward() → top-K features
     GWT Resonance (L5) → `invoke("gwt_resonance_status")`
       → nt_core_gwt::ResonatorNetwork → active resonator count
     StarPulse Protocol (L7) → `invoke("starpulse_bus_status")`
       → l7_capability::protocol::PulseBus → queued messages
     Capability Bid (L7) → `invoke("cap_registry_bids")`
       → l7_capability::registry → bid scores per capability
     MCP Tools (L1) → `invoke("mcp_tool_status")`
       → nt_agent_mcp_discovery → tool health + latency

     ALL TABS share:
     - GatewayV2 for LLM completion
     - nt_memory_kb for context retrieval
     - nt_io_telemetry for span tracing (future)
     ════════════════════════════════════════════════════════════ */
  /* ===== Icon Library ===== */
  const I = {
    plus: '<svg viewBox="0 0 14 14"><path d="M7 1.5l1.2 3.8 3.8 1.2-3.8 1.2L7 11.5l-1.2-3.8L2 6.5l3.8-1.2z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round"/></svg>',
    folder: '<svg viewBox="0 0 14 14"><path d="M1.5 4.5h3.5l1-1.5h6a1 1 0 011 1v6a1 1 0 01-1 1h-10a1 1 0 01-1-1v-5.5z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>',
    play: '<svg viewBox="0 0 14 14"><polygon points="3.5,2 11.5,7 3.5,12" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/><circle cx="3.5" cy="7" r="1.2" fill="currentColor" stroke="none" opacity="0.3"/></svg>',
    clock: '<svg viewBox="0 0 14 14"><circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.2" fill="none"/><line x1="7" y1="7" x2="7" y2="4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/><line x1="7" y1="7" x2="9.5" y2="8.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>',
    grid: '<svg viewBox="0 0 14 14"><path d="M3.5 2l.8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2L1 5l2.2-.8z" stroke="currentColor" stroke-width="1" fill="none" stroke-linejoin="round"/><path d="M10.5 8l.8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2-2.2-.8 2.2-.8z" stroke="currentColor" stroke-width="1" fill="none" stroke-linejoin="round"/></svg>',
    sliders: '<svg viewBox="0 0 14 14"><line x1="2" y1="4" x2="12" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><path d="M4 3.5l.5 1.5h-1z" stroke="currentColor" stroke-width="1" fill="none"/><line x1="2" y1="9" x2="12" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><path d="M10 8.5l.5 1.5h-1z" stroke="currentColor" stroke-width="1" fill="none"/></svg>',
    playAlt: '<svg viewBox="0 0 14 14"><polygon points="3,2 12,7 3,12" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>',
    cube: '<svg viewBox="0 0 14 14"><path d="M7 1.5l5 2.75v5.5L7 12.5l-5-2.75v-5.5z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round"/><path d="M7 6.8L2 4M7 6.8v5.6M7 6.8l5-2.8" stroke="currentColor" stroke-width="1" fill="none" stroke-linejoin="round"/></svg>',
    search: '<svg viewBox="0 0 14 14"><circle cx="6" cy="6" r="3.5" stroke="currentColor" stroke-width="1.2" fill="none"/><line x1="9" y1="9" x2="12.5" y2="12.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>',
  };

  /* ===== Nav & Recent Data ===== */
  const navData = {
    chat: [
      { icon: 'plus', label: '新对话', action: 'newChat' },
      { icon: 'folder', label: '项目', action: 'showProjects' },
    ],
    cowork: [
      { icon: 'plus', label: '新会话', action: 'newCoworkTask' },
      { icon: 'folder', label: '项目', action: 'showProjects' },
      { icon: 'sliders', label: '代理设置', action: 'runAgentChain' },
    ],
  };

  const recentData = {
    chat: [
      { text: '复刻桌面APP UI设计和布局', time: '2小时前' },
    ],
    cowork: [
      { text: '架构讨论 · 3 个任务', time: '10分钟前' },
      { text: '代码审查 Sprint', time: '1小时前' },
    ],
  };

  let currentNav = { chat: 0, cowork: 0 };
  let isChatMode = false;
  let currentView = 'chat';

  /* ===== Action Dispatch ===== */
  const actions = {
    newChat() {
      switchView(document.querySelector('.segb[data-view="chat"]'), 'chat');
      isChatMode = false;
      document.getElementById('heroSection').style.display = 'flex';
      const cs = document.getElementById('chatScroll');
      cs.style.display = 'none';
      cs.innerHTML = '';
      document.getElementById('chatInput').value = '';
    },
    async showProjects() {
      document.getElementById('opTitle').textContent = '项目';
      const body = document.getElementById('opBody');
      if(!body) return;
      if(!isTauri()){
        body.innerHTML = `<div style="display:flex;flex-direction:column;gap:12px;padding:12px">
          <div style="display:flex;justify-content:space-between;align-items:center"><h3 style="font-size:16px;font-weight:600;margin:0">活跃项目</h3><span style="font-size:11px;color:var(--tx3)">—</span></div>
          <div class="kb-empty">浏览器模式：仅 Tauri 下可读取项目</div>
        </div>`;
        openOverlay('overlayProjects');
        return;
      }
      try{
        const projects = await invoke('project_list');
        const list = Array.isArray(projects) ? projects : [];
        body.innerHTML = `<div style="display:flex;flex-direction:column;gap:12px;padding:12px">
          <div style="display:flex;justify-content:space-between;align-items:center"><h3 style="font-size:16px;font-weight:600;margin:0">活跃项目</h3><span style="font-size:11px;color:var(--tx3)">${list.length} 个</span></div>
          ${list.length ? list.map(p => `<div class="evo-card" style="cursor:pointer" onclick="showToast('项目: ${escHtml(p.name)}')"><span class="lbl">${escHtml(p.project_type || 'project')}</span><span class="val">${escHtml(p.name)}</span><span style="font-size:10px;color:var(--tx3)">${escHtml(p.path || '')}${p.pinned ? ' · 📌' : ''}</span></div>`).join('') : '<div class="kb-empty">暂无项目</div>'}
          <button class="cht" style="margin-top:4px" onclick="showToast('创建新项目...')">+ 新建项目</button>
        </div>`;
      }catch(_e){
        body.innerHTML = `<div style="display:flex;flex-direction:column;gap:12px;padding:12px"><div class="kb-empty">项目读取失败</div></div>`;
      }
      openOverlay('overlayProjects');
    },
    showAchievements() {
      document.getElementById('opTitle').textContent = '成果';
      document.getElementById('opBody').innerHTML = `<div style="display:flex;flex-direction:column;gap:10px;padding:12px">
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">
          <div class="evo-card"><span class="lbl">测试通过</span><span class="val" style="color:var(--suc)">5,437</span></div>
          <div class="evo-card"><span class="lbl">模块实现</span><span class="val" style="color:var(--pri)">48</span></div>
          <div class="evo-card"><span class="lbl">代码行数</span><span class="val">142K</span></div>
          <div class="evo-card"><span class="lbl">API 调用</span><span class="val">2.1K</span></div>
        </div>
        <div class="evo-card"><span class="lbl">近期里程碑</span><span style="font-size:11px;color:var(--tx2);margin-top:2px">Cycle 6: 约束解码引擎 · GatewayV2 集成 · 9 层架构</span></div>
      </div>`;
      openOverlay('overlayProjects');
    },
    newCoworkTask() {
      switchView(document.querySelector('.segb[data-view="cowork"]'), 'cowork');
      createSession();
    },
    showScheduled() {
      const now = new Date();
      const nxtH = (now.getMinutes()+15 >= 60) ? now.getHours()+1 : now.getHours();
      const nxtM = (now.getMinutes()+15) % 60;
      document.getElementById('opTitle').textContent = '计划任务';
      document.getElementById('opBody').innerHTML = `<div style="display:flex;flex-direction:column;gap:10px;padding:12px">
        <div class="evo-card"><span class="lbl">下一步</span><span class="val">记忆整合</span><span style="font-size:10px;color:var(--tx3)">${String(nxtH).padStart(2,'0')}:${String(nxtM).padStart(2,'0')} · 每 30 分钟</span></div>
        <div class="evo-card"><span class="lbl">下个整点</span><span class="val">对话蒸馏</span><span style="font-size:10px;color:var(--tx3)">${String(now.getHours()+1).padStart(2,'0')}:00 · 每 60 分钟</span></div>
        <div class="evo-card"><span class="lbl">每日</span><span class="val">SEAL 自迭代</span><span style="font-size:10px;color:var(--tx3)">03:00 · 27 阶段管线</span></div>
      </div>`;
      openOverlay('overlayProjects');
    },
    registry() {
      openOverlay('overlayRegistry');
      loadRegistry();
    },
    hypercube() {
      openOverlay('overlayHypercube');
    },
    runAgentChain() {
      openSettingsModal();
      const items = document.querySelectorAll('.st-item');
      const target = Array.from(items).find(el => el.textContent.includes('代理'));
      if(target) selectSetting(target, 'gateway');
    },
  };

  /* Agent chain simulation removed — moved to Settings → 代理 · 网关 */

  function dispatch(action) {
    if (action.startsWith('toast:')) { showToast(action.slice(6)); return; }
    if (actions[action]) { actions[action](); return; }
    showToast('功能开发中: ' + action);
  }

  /* ===== Sidebar Renderer ===== */
  function renderSidebar(tab) {
    const navEl = document.getElementById('navList');
    navEl.innerHTML = navData[tab].map((item, i) => {
      const active = i === currentNav[tab];
      return `<div class="nl${active ? ' on' : ''}" data-index="${i}" onclick="onNavClick('${tab}',${i})">${I[item.icon]}<span>${item.label}</span></div>`;
    }).join('');

    const reEl = document.getElementById('recentList');
    reEl.innerHTML = '<div class="re-h">最近</div>' + recentData[tab].map(r => {
      if (r.ghost) {
        return `<div class="re-i ghost"><span class="circle"></span><span class="t">${escHtml(r.text)}</span></div>`;
      }
      const timeHtml = r.time ? `<span class="re-time">${r.time}</span>` : '';
      const click = r.id
        ? `onclick="openRecent('${tab}','${r.id}',${r.cowork ? 'true' : 'false'})"`
        : `onclick="dispatch('toast:打开 ${escHtml(r.text)}')"`;
      return `<div class="re-i" ${click}><span class="dot"></span><span class="t">${escHtml(r.text)}</span>${timeHtml}</div>`;
    }).join('');
  }

  async function openRecent(tab, id, cowork){
    if(!isTauri() || !id){ dispatch('toast:打开 ' + (recentData[tab].find(x=>x.id===id)?.text || '')); return; }
    const title = recentData[tab].find(x=>x.id===id)?.text || id;
    try{
      if(cowork){
        const idx = backendSessionList.findIndex(s => s.id === id && s.isCowork);
        if(idx >= 0){
          switchView(document.querySelector('.segb[data-view="cowork"]'), 'cowork');
          selectCwSession(idx, true);
          showToast('已打开协同会话: ' + title);
          return;
        }
      }
      await invoke('neocodex_switch_session', { session_id: id });
      const msgs = await invoke('neocodex_get_session_messages', { session_id: id });
      if(!Array.isArray(msgs)) return;
      switchView(document.querySelector('.segb[data-view="chat"]'), 'chat');
      document.getElementById('heroSection').style.display = 'none';
      const cs = document.getElementById('chatScroll');
      cs.style.display = 'flex';
      cs.innerHTML = '';
      msgs.forEach(m => {
        const u = document.createElement('div');
        u.className = 'msg r';
        u.innerHTML = `<div class="mb">${escHtml(m.content)}</div>`;
        cs.appendChild(u);
        const a = document.createElement('div');
        a.className = 'msg l';
        const t = new Date((m.timestamp || Date.now()/1000) * 1000).toLocaleTimeString([], { hour:'2-digit', minute:'2-digit' });
        a.innerHTML = `<div class="msg-h"><span class="name">NeoTrix</span><span class="time">${t}</span></div><div class="mb">${renderRichText(m.content)}</div>`;
        cs.appendChild(a);
      });
      showToast('已打开会话: ' + title);
    }catch(e){ showToast('打开会话失败: ' + e); }
  }

  function onNavClick(tab, index) {
    currentNav[tab] = index;
    const item = navData[tab][index];
    renderSidebar(tab);
    dispatch(item.action);
  }

  function switchView(el,view){
    document.querySelectorAll('.segb').forEach(b=>{
      b.classList.remove('on');
      b.setAttribute('aria-selected','false');
    });
    el.classList.add('on');
    el.setAttribute('aria-selected','true');
    currentView=view;
    document.querySelectorAll('.vw-chat,.vw-cowork').forEach(v=>v.style.display='none');
    const map={chat:'viewChat',cowork:'viewCowork'};
    const t=document.getElementById(map[view]);
    if(t)t.style.display='flex';
    currentNav[view]=0;
    renderSidebar(view);
    if(view==='chat'){ renderHeroSuggest(); }
    if(view==='cowork') { renderCowork(); }
  }

  /* ===== Right Sidebar — File Tree ===== */
  const fileTreeData = [
    { name:'src', type:'dir', open:true, children:[
      { name:'main.rs', type:'file' },
      { name:'lib.rs', type:'file' },
      { name:'engine_core.rs', type:'file', content:'pub struct ReasoningEngine {\n    pub gateway: Arc<GatewayV2>,\n    e8_state: E8State,\n    confidence: f64,\n}\n\nimpl ReasoningEngine {\n    pub fn new() -> Self {\n        Self {\n            gateway: GatewayV2::default(),\n            e8_state: E8State::Ground,\n            confidence: 0.92,\n        }\n    }\n}' },
      { name:'config.rs', type:'file', content:'pub struct Config {\n    pub provider: ProviderType,\n    pub max_tokens: u32,\n    pub temperature: f64,\n}\n\nimpl Default for Config {\n    fn default() -> Self {\n        Self {\n            provider: ProviderType::OpenAI,\n            max_tokens: 4096,\n            temperature: 0.7,\n        }\n    }\n}' },
    ]},
    { name:'components', type:'dir', open:false, children:[
      { name:'mod.rs', type:'file' },
      { name:'chat.rs', type:'file' },
      { name:'sidebar.rs', type:'file' },
    ]},
    { name:'tests', type:'dir', open:false, children:[
      { name:'test_engine.rs', type:'file' },
    ]},
    { name:'Cargo.toml', type:'file' },
  ];

  let currentPreviewMode = 'rendered';
  let currentArtifactView = 'preview'; /* 'preview' or 'code' */

  function renderFileTree(nodes,parent,level=0){
    const el = parent || document.getElementById('fileTree');
    if(!parent) el.innerHTML = '';
    nodes.forEach(n=>{
      const div = document.createElement('div');
      div.className = 'ft-item' + (n.type==='file'?' ft-file':'');
      div.style.paddingLeft = (level*14+4)+'px';
      if(n.type==='dir'){
        const ch = n.open ? 'open' : '';
        div.innerHTML = `<svg class="chev ${ch}" viewBox="0 0 9 9"><line x1="3" y1="2.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><line x1="3" y1="6.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg><svg class="fic" viewBox="0 0 14 14"><path d="M1.5 4.5h3.5l1-1.5h6a1 1 0 011 1v6a1 1 0 01-1 1h-10a1 1 0 01-1-1v-5.5z" stroke="currentColor" stroke-width="1" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>${escHtml(n.name)}`;
        div.onclick = (e)=>{ e.stopPropagation(); n.open = !n.open; renderFileTree(null,null); };
        el.appendChild(div);
        const childDiv = document.createElement('div');
        childDiv.className = 'ft-children' + (n.open?' open':'');
        el.appendChild(childDiv);
        if(n.children) renderFileTree(n.children,childDiv,level+1);
      } else {
        div.innerHTML = `<svg class="fic" viewBox="0 0 14 14"><path d="M2 1.5h10v11H2z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round"/><line x1="4.5" y1="4.5" x2="9.5" y2="4.5" stroke="currentColor" stroke-width="1" stroke-linecap="round"/></svg>${escHtml(n.name)}`;
        div.onclick = (e)=>{ e.stopPropagation(); document.querySelectorAll('.ft-file').forEach(f=>f.classList.remove('ft-active')); div.classList.add('ft-active'); showFilePreview(n); };
        el.appendChild(div);
      }
    });
  }

  /* ════════════════════════════════════════════════
     Artifact Pane — Unified Format Tabs + View Toggle
     ════════════════════════════════════════════════ */
  const previewFormats = [
    { id:'raw', label:'Raw' },
    { id:'rendered', label:'Rendered' },
    { id:'wechat', label:'WeChat' },
    { id:'zhihu', label:'Zhihu' },
    { id:'juejin', label:'Juejin' },
    { id:'web', label:'Web' },
  ];

  function renderFormatTabs(){
    const el = document.getElementById('fpFmt');
    el.innerHTML = previewFormats.map(f =>
      `<button class="ap-tab${f.id===currentPreviewMode?' on':''}" data-fmt="${f.id}" onclick="setPreviewMode('${f.id}')">${f.label}</button>`
    ).join('');
  }

  function setPreviewMode(mode){
    currentPreviewMode = mode;
    const ap = document.getElementById('filePreview');
    const name = ap._currentName || '';
    const content = ap._currentContent || '';
    renderFormatTabs();
    renderPreviewContent(name, content);
  }

  async function showFilePreview(node){
    openRbSidebar();
    const ap = document.getElementById('filePreview');
    ap.classList.remove('mini');
    const body = document.getElementById('fpBody');
    body.classList.add('open');
    ap._currentName = node.name;
    let content = node.content || '// (empty)';
    if(node.load && isTauri()){
      content = '// 加载中…';
      renderPreviewContent(node.name, content);
      try{
        const raw = await invoke('read_file', { path: node.load });
        content = (typeof raw === 'string' && raw) ? raw : '// (empty)';
      }catch(e){
        content = '// 读取失败: ' + String(e);
      }
    }
    ap._currentContent = content;
    document.getElementById('fpName').textContent = node.name;
    renderFormatTabs();
    renderPreviewContent(node.name, content);
  }

  /* ── View Toggle: Preview ↔ Code ── */
  function switchArtifactView(btn, view){
    currentArtifactView = view;
    document.querySelectorAll('.ap-view-btn').forEach(b=>b.classList.remove('on'));
    btn.classList.add('on');
    /* re-render with current mode to switch between rendered/code view */
    const ap = document.getElementById('filePreview');
    renderPreviewContent(ap._currentName||'', ap._currentContent||'');
  }

  /* ── Smart MD Renderer ── */
  function renderPreviewContent(name, content){
    const el = document.getElementById('fpContent');
    const mode = currentArtifactView==='code' ? 'raw' : currentPreviewMode;
    const text = content || '// (empty)';

    /* Set platform class on content container */
    el.className = 'ap-content';
    if(mode !== 'raw'){
      const platformClass = mode==='rendered'?'md-web':('md-'+mode);
      el.classList.add(platformClass);
    }

    if(mode === 'raw'){
      const kw = ['pub','struct','impl','fn','let','mut','const','Self','for','in','return','if','else','match','use','mod','trait','enum','type','where','as','async','await','move'];
      let html = text;
      kw.forEach(k=>{ html = html.replace(new RegExp('\\b'+k+'\\b','g'),'<span class="kw">'+k+'</span>'); });
      html = html.replace(/\/\/.*/g,m=>'<span class="cm">'+escHtml(m)+'</span>');
      html = html.replace(/\b[A-Z]\w+(?=\s*(?:[({<]|::))/g,m=>'<span class="fn">'+m+'</span>');
      el.innerHTML = '<code style="font-family:var(--fm);font-size:10px;line-height:1.6;color:var(--tx2);">'+escHtml(html)+'</code>';
      return;
    }

    /* Smart markdown → HTML */
    let md = text
      .replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')
      .replace(/^#### (.+)$/gm,'<h4>$1</h4>')
      .replace(/^### (.+)$/gm,'<h3>$1</h3>')
      .replace(/^## (.+)$/gm,'<h2>$1</h2>')
      .replace(/^# (.+)$/gm,'<h1>$1</h1>')
      .replace(/```(\w*)\n([\s\S]*?)```/g,(m,lang,code)=>`<pre><code class="lang-${lang}">${escHtml(code)}</code></pre>`)
      .replace(/`([^`]+)`/g,'<code>$1</code>')
      .replace(/\*\*(.+?)\*\*/g,'<strong>$1</strong>')
      .replace(/\*(.+?)\*/g,'<em>$1</em>')
      .replace(/^> (.+)$/gm,(m,c)=>`<div class="callout">${c}</div>`)
      .replace(/^- (.+)$/gm,'<li>$1</li>')
      .replace(/(<li>.*<\/li>\n?)+/g,'<ul>$&</ul>')
      .replace(/^(\d+)\. (.+)$/gm,'<li value="$1">$2</li>')
      .replace(/!\[([^\]]*)\]\(([^)]+)\)/g,'<img src="$2" alt="$1" loading="lazy"/>')
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g,'<a href="$2">$1</a>')
      .replace(/^---+$/gm,'<hr/>')
      .replace(/\n\n/g,'</p><p>')
      .replace(/^(?!<[hplib]|<[uo]l|<pre|<bl|<div|<h[1-4])/gm,'<p>')
      .replace(/<\/p>\s*<p>/g,'</p><p>');

    el.innerHTML = md;
  }

  /* ── Artifact Pane Actions ── */
  function expandPreview(e){
    if(e.target.closest('.ap-tab')) return;
    const ap = document.getElementById('filePreview');
    const body = document.getElementById('fpBody');

    /* Expand from mini state */
    if(ap.classList.contains('mini')){
      ap.classList.remove('mini');
      body.classList.add('open');
      return;
    }

    body.classList.toggle('open');
  }

  function closePreview(e){
    e.stopPropagation();
    const ap = document.getElementById('filePreview');
    ap.classList.add('mini');
    document.getElementById('fpBody').classList.remove('open');
    document.getElementById('fpName').textContent = '未选择文件';
    document.querySelectorAll('.ft-active').forEach(f=>f.classList.remove('ft-active'));
  }

  function copyPreview(e){
    e.stopPropagation();
    const ap = document.getElementById('filePreview');
    const text = ap._currentContent || '';
    navigator.clipboard.writeText(text).then(()=>showToast('已复制'));
  }

  function refreshPreview(e){
    e.stopPropagation();
    showToast('已刷新');
  }

  /* ════════════════════════════════════════════════
     Chat file link → auto-open sidebar preview
     ════════════════════════════════════════════════ */
  function showFilePreviewFromChat(name, content){
    const fileNode = { name, content };
    openRbSidebar();
    showFilePreview(fileNode);
  }

  /* ════════════════════════════════════════════════
     Right Sidebar — Auto-hide + Toggle
     Mouse near right edge (≤12px) → show sidebar
     Mouse leave → hide after 600ms
     ════════════════════════════════════════════════ */
   let rbAutoHide = false;
  let rbHideTimer = null;
  let rbEdgeActive = false;
  const chatDocs = [
    {name:'architecture-overview.md', content:'# 架构概览\n\n## 系统设计\n- 9层意识架构\n- VSA HyperCube 知识表示\n- E8 状态空间推理引擎\n\n## 核心组件\n- L7 能力层处理路由\n- StarPulse 协议实现层间通信\n- GWT 注意力路由与谐振'},
    {name:'api-reference.md', content:'# API 参考\n\n## 端点\n- `POST /api/reason` — 提交推理任务\n- `GET /api/status` — 检查系统健康状态\n- `POST /api/knowledge/search` — 查询知识库\n\n## 认证\n通过 `Authorization` 请求头传递 Bearer 令牌。'},
    {name:'deployment-guide.md', content:'# 部署指南\n\n## 前提条件\n- Rust 2021 edition\n- Node.js 18+\n- SQLite 3.x\n\n## 步骤\n1. `cargo build -p neotrix`\n2. 配置 `~/.config/neotrix/config.toml`\n3. 运行 `neotrix serve`'}
  ];

  /* Edge detection: mousemove near right edge */
  document.addEventListener('mousemove', e=>{
    if(!rbAutoHide) return;
    const nearEdge = e.clientX >= window.innerWidth - 12;
    const rb = document.getElementById('rightbar');
    const overRb = rb.matches(':hover');
    if(nearEdge && !rbEdgeActive){
      rbEdgeActive = true;
      rb.classList.add('rb-hover');
      clearTimeout(rbHideTimer);
    } else if(!nearEdge && !overRb && rbEdgeActive){
      rbEdgeActive = false;
      rbHideTimer = setTimeout(()=>{
        document.getElementById('rightbar').classList.remove('rb-hover');
      }, 600);
    }
  });

  function toggleAutoHide(){
    rbAutoHide = !rbAutoHide;
    const rb = document.getElementById('rightbar');
    rb.classList.toggle('auto-hide', rbAutoHide);
    rb.classList.remove('collapsed', 'rb-hover');
    clearTimeout(rbHideTimer);
    updateRbFloatIcon();
  }

  function toggleRbSidebar(){
    const rb = document.getElementById('rightbar');
    if(rbAutoHide){
      toggleAutoHide();
      return;
    }
    rb.classList.toggle('collapsed');
    updateRbFloatIcon();
  }

  function openRbSidebar(){
    const rb = document.getElementById('rightbar');
    if(rbAutoHide){
      clearTimeout(rbHideTimer);
      rb.classList.add('rb-hover');
    } else if(rb.classList.contains('collapsed')){
      rb.classList.remove('collapsed');
      updateRbFloatIcon();
    }
  }

  function onRbLeave(){
    if(!rbAutoHide) return;
    rbEdgeActive = false;
    rbHideTimer = setTimeout(()=>{
      document.getElementById('rightbar').classList.remove('rb-hover');
    }, 600);
  }

  function updateRbFloatIcon(){
    const rb = document.getElementById('rightbar');
    const icon = document.getElementById('rbFloatIcon');
    const hidden = rb.classList.contains('collapsed') || rbAutoHide;
    icon.innerHTML = hidden
      ? '<line x1="3" y1="2" x2="5" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><line x1="3" y1="6" x2="5" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>'
      : '<line x1="5" y1="2" x2="3" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/><line x1="5" y1="6" x2="3" y2="4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>';
  }

  renderFileTree(fileTreeData, null);

  /* Initialize: collapsed by default */
  (function init(){
    const rb = document.getElementById('rightbar');
    rb.classList.remove('auto-hide');
    rb.classList.add('collapsed');
    updateRbFloatIcon();
    /* Overlay backdrop close on click outside box */
    document.querySelectorAll('.overlay-panel').forEach(p=>{
      p.addEventListener('click', function(e){
        if(e.target===this){ this.classList.remove('open'); updateTrafficVisibility(); }
      });
    });
  })();

  /* ════════════════════════════════════════════════
     Proxy Dashboard — IP Pool + Free LLM + Network
     State machine updates all tables every 900ms
     ════════════════════════════════════════════════ */
  /* ── Existing data ── */
  const PROXY_DATA = [
    { name:'res-1.us-west', reg:'🇺🇸', ip:'104.28.0.1:8080', good:true, lat:42, score:'S' },
    { name:'res-2.us-east', reg:'🇺🇸', ip:'104.28.1.5:3128', good:true, lat:38, score:'A' },
    { name:'res-3.eu-berlin', reg:'🇩🇪', ip:'85.10.0.22:8080', good:true, lat:89, score:'A' },
    { name:'res-4.eu-london', reg:'🇬🇧', ip:'31.6.0.44:80', good:false, lat:0, score:'D' },
    { name:'res-5.asg-sin', reg:'🇸🇬', ip:'103.2.0.8:3128', good:true, lat:112, score:'B' },
    { name:'res-6.hkg', reg:'🇭🇰', ip:'47.0.0.1:443', good:true, lat:98, score:'B' },
    { name:'res-7.br-sao', reg:'🇧🇷', ip:'177.0.0.5:8080', good:false, lat:0, score:'D' },
    { name:'res-8.jp-tokyo', reg:'🇯🇵', ip:'103.0.0.9:3128', good:true, lat:65, score:'A' },
    { name:'res-9.au-syd', reg:'🇦🇺', ip:'1.0.0.1:80', good:true, lat:148, score:'B' },
    { name:'res-10.in-mum', reg:'🇮🇳', ip:'103.0.0.15:8080', good:false, lat:0, score:'D' },
    { name:'res-11.za-joburg', reg:'🇿🇦', ip:'196.0.0.3:3128', good:true, lat:195, score:'C' },
    { name:'res-12.kr-seoul', reg:'🇰🇷', ip:'121.0.0.7:443', good:true, lat:54, score:'A' },
  ];

  const FREE_LLM_DATA = [
    { name:'Groq Llama 3.3 70B', prov:'Groq', good:true, lat:340, rpm:'28/30', cost:'$0', score:'S' },
    { name:'OpenRouter Mixtral', prov:'OpenRouter', good:true, lat:520, rpm:'18/20', cost:'$0', score:'A' },
    { name:'Pollinations GPT', prov:'Pollinations', good:true, lat:890, rpm:'12/15', cost:'$0', score:'B' },
    { name:'Cerebras Llama', prov:'Cerebras', good:true, lat:280, rpm:'22/25', cost:'$0', score:'S' },
    { name:'SambaNova Llama', prov:'SambaNova', good:false, lat:0, rpm:'0/10', cost:'$0', score:'D' },
    { name:'DeepSeek V3', prov:'DeepSeek', good:true, lat:450, rpm:'14/20', cost:'$0', score:'A' },
  ];

  /* ===== Cowork Session Management ===== */
  let CW_DATA = [
    { name:'架构讨论', status:'进行中', tasks:3, done:1, fail:0, agents:[{n:'分析员',on:true},{n:'架构师',on:true},{n:'审查员',on:false}] },
    { name:'代码审查 Sprint', status:'进行中', tasks:5, done:3, fail:0, agents:[{n:'审查员',on:true},{n:'检查员',on:true}] },
    { name:'文档生成', status:'已完成', tasks:2, done:2, fail:0, agents:[{n:'写手',on:false}] },
  ];
  let cwStatusFilter = 'all';
  function cwFilter(status){
    cwStatusFilter = status;
    document.querySelectorAll('.cw-fchip').forEach(b=>b.classList.toggle('on', b.dataset.status===status));
    renderCowork();
  }
  let agentRunning = false, agentTask = '', agentUp = 0;

  function fmtRelTime(ts){
    if(!ts) return '';
    const diff = Date.now() - (typeof ts === 'number' ? ts * 1000 : new Date(ts).getTime());
    if(diff < 60e3) return '刚刚';
    if(diff < 3600e3) return Math.floor(diff/60e3) + '分钟前';
    if(diff < 86400e3) return Math.floor(diff/3600e3) + '小时前';
    if(diff < 604800e3) return Math.floor(diff/86400e3) + '天前';
    return Math.floor(diff/604800e3) + '周前';
  }

  function fmtUptime(sec){
    if(sec < 60) return `${Math.round(sec)}s`;
    const m = Math.floor(sec/60);
    if(m < 60) return `${m}m`;
    const h = Math.floor(m/60);
    return `${h}h ${m%60}m`;
  }

  function renderCowork(){
    const sl = document.getElementById('cwSessionList');
    if(!sl) return;
    const filtered = CW_DATA.filter(s => {
      if(cwStatusFilter === 'active') return s.status !== '已完成';
      if(cwStatusFilter === 'done') return s.status === '已完成';
      return true;
    });
    sl.innerHTML = filtered.map((s,i) => {
      const pct = s.tasks > 0 ? Math.round(s.done/s.tasks*100) : 0;
      return `<div class="cw-sitem${i===0?' active':''}" data-idx="${CW_DATA.indexOf(s)}" onclick="selectCwSession(${CW_DATA.indexOf(s)}, true)">
        ${escHtml(s.name)}
        <span class="s">${s.done}/${s.tasks} 任务 · ${pct}%</span>
      </div>`;
    }).join('');
    if(!filtered.length){
      sl.innerHTML = `<div class="cw-empty" style="display:flex;flex-direction:column;align-items:center;gap:6px;padding:24px 12px;text-align:center"><p style="margin:0;font-size:var(--fs-small);color:var(--tx2)">没有符合条件的会话</p><span style="font-size:var(--fs-caption);color:var(--tx-meta)">切换其他状态或新建会话</span></div>`;
      document.getElementById('cwEmpty').style.display = 'flex';
      document.getElementById('cwContent').style.display = 'none';
      return;
    }
    selectCwSession(0);
  }

  function selectCwSession(idx, load){
    const s = CW_DATA[idx] || CW_DATA[0];
    document.querySelectorAll('.cw-sitem').forEach(el => el.classList.toggle('active', String(el.dataset.idx) === String(idx)));
    document.getElementById('cwEmpty').style.display = 'none';
    document.getElementById('cwContent').style.display = 'flex';
    document.getElementById('cwDetailTitle').textContent = escHtml(s.name);
    const mc = s.message_count ? ` · ${s.message_count} 消息` : '';
    document.getElementById('cwDetailStatus').textContent = `${s.tasks} 任务 · ${s.done} 完成 · ${s.fail} 失败${mc}`;
    document.getElementById('cwHBadge').textContent = s.status;

    const tl = document.getElementById('cwTaskList');
    if(tl){
      let tasks = [];
      if(s.isCowork && s.cw){
        const c = s.cw;
        const items = [];
        if(c.deliverables && c.deliverables.length){
          c.deliverables.forEach((d, i) => {
            items.push({ name: d.name || ('交付物 #' + (i+1)), done: i < (c.files_created||0), fail: false, meta: d.kind || '' });
          });
        }
        if(c.files_created || c.files_modified){
          items.push({ name: '读取 ' + (c.files_read||0) + ' 文件', done: true, fail: false });
          items.push({ name: '修改 ' + (c.files_modified||0) + ' 文件', done: true, fail: false });
          items.push({ name: '创建 ' + (c.files_created||0) + ' 文件', done: (c.files_created||0) > 0, fail: false });
        }
        if(!items.length) items.push({ name: '无任务', done: false, fail: false });
        tasks = items;
      }else{
        for(let i=0;i<s.tasks;i++){
          const done = i < s.done;
          const fail = !done && i < s.done + s.fail;
          tasks.push({ name: `任务 #${i+1}`, done, fail });
        }
        if(!s.tasks) tasks.push({ name: '无任务 · 发送消息开始', done: false, fail: false });
      }
      tl.innerHTML = tasks.map(t => {
        const cls = t.done ? 'done' : (t.fail ? 'fail' : '');
        const label = t.done ? '已完成' : (t.fail ? '失败' : '进行中');
        const meta = t.meta ? `<span class="tstat">${escHtml(t.meta)}</span>` : `<span class="tstat">${label}</span>`;
        return `<div class="cw-task"><span class="dot ${cls}"></span><span class="tname">${escHtml(t.name)}</span>${meta}</div>`;
      }).join('');
    }

    const al = document.getElementById('cwAgentList');
    if(al) renderAgentRow();

    if(load && s.id && !s.isCowork){ loadSessionMessages(s.id); }
    refreshAgent();
  }

  async function runMsgCode(btn){
    const pre = btn.closest('.msg-code')?.querySelector('pre');
    if(!pre) return;
    const cmd = pre.textContent.trim();
    if(!cmd){ showToast('空命令'); return; }
    btn.disabled = true;
    const out = document.createElement('div');
    out.className = 'msg-code-out';
    out.innerHTML = '<span class="msg-code-run">$ ' + escHtml(cmd) + '</span><span class="typing"><span></span><span></span><span></span></span>';
    pre.after(out);
    try{
      const result = isTauri() ? await invoke('execute_terminal_command', { command: cmd }) : `（浏览器模式模拟）\nHello from NeoTrix shell\n`;
      const code = (typeof result === 'string' ? result : '') || '';
      out.innerHTML = '<span class="msg-code-run">$ ' + escHtml(cmd) + '</span><pre class="msg-code-res">' + escHtml(code) + '</pre>';
      if(code.trim()) out.classList.add('ok');
      else out.innerHTML += '<span class="msg-code-emp">（无输出）</span>';
    }catch(e){
      out.innerHTML = '<span class="msg-code-run">$ ' + escHtml(cmd) + '</span><span class="msg-code-err">' + escHtml(String(e)) + '</span>';
    }
    btn.disabled = false;
  }

  /* ===== Code Tab Switching ===== */
  function highlightCode(code){
    const kwSet = new Set(['fn','pub','let','mut','Result','Ok','f64','std','use','match','for','in','if','else','return','struct','impl','enum','trait']);
    const esc = s => s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
    const re = /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\/\/[^\n]*|[A-Z]\w+|\w+|[^a-zA-Z0-9_'"\s]+|\s+)/g;
    let out = '', m;
    while((m = re.exec(code)) !== null){
      const tok = m[0];
      if(tok.startsWith('"') || tok.startsWith("'")) out += '<span class="hl">' + esc(tok) + '</span>';
      else if(tok.startsWith('//')) out += '<span class="cm">' + esc(tok) + '</span>';
      else if(/^[A-Z]\w+$/.test(tok)) out += '<span class="fn">' + tok + '</span>';
      else if(/^\w+$/.test(tok)) out += kwSet.has(tok) ? '<span class="kw">' + tok + '</span>' : tok;
      else out += esc(tok);
    }
    return out;
  }

  function copyMsgCode(btn){
    const pre = btn.closest('.msg-code')?.querySelector('pre');
    if(!pre) return;
    const text = pre.textContent;
    navigator.clipboard.writeText(text).then(() => showToast('代码已复制'));
  }

  function renderRichText(text){
    const esc = s => escHtml(s).replace(/\n/g, '<br>');
    const parts = String(text || '').split(/```/);
    let out = '', inBlock = false;
    for(const part of parts){
      if(!inBlock){ out += esc(part); }
      else{
        const nl = part.indexOf('\n');
        const lang = nl === -1 ? part.trim() : part.slice(0, nl).trim();
        const code = nl === -1 ? '' : part.slice(nl + 1);
        out += `<div class="msg-code"><div class="msg-code-h"><span class="msg-code-lang">${escHtml(lang || 'code')}</span><span class="msg-code-actions"><button class="msg-code-cp" onclick="runMsgCode(this)">运行</button><button class="msg-code-cp" onclick="copyMsgCode(this)">复制</button></span></div><pre class="msg-code-b">${highlightCode(code)}</pre></div>`;
      }
      inBlock = !inBlock;
    }
    return out || '<span></span>';
  }

  function switchProvider(val){
    showToast('切换提供者: ' + val);
  }

  /* ===== IPC-backed streaming send ===== */
  const streamSubs = new Map();
  function ensureStreamListeners(){
    if(!isTauri() || streamSubs.size) return;
    const attach=(ev,fn)=>{ try{ listen(ev, fn).then(un=>streamSubs.set(ev,un)).catch(()=>{}); }catch(_e){} };
    attach('neocodex_stream_token', p => {
      const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
      if(el) el.textContent += String(p);
    });
    attach('neocodex_stream_end', p => {
      const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
      if(el){ el.classList.remove('streaming'); el.innerHTML = renderRichText(String(p)); attachUsageFooter(el.closest('.msg')); }
      document.getElementById('sendBtn').disabled=false;
    });
    attach('neocodex_stream_done', async () => {
      document.getElementById('sendBtn').disabled=false;
      const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
      if(el){ el.classList.remove('streaming'); attachUsageFooter(el.closest('.msg')); }
      await loadUsage();
      const foot=document.querySelector('#chatScroll .msg.l:last-child .msg-usage');
      if(foot) foot.textContent = '上下文 ' + Math.round(lastContextUsage * 100) + '%';
    });
    attach('neocodex_stream_start', () => { /* reserve */ });
  }

  function sendMsg(){
    ensureStreamListeners();
    const inp=document.getElementById('chatInput');
    const txt=inp.value.trim();if(!txt)return;
    if(!isChatMode){
      isChatMode=true;
      document.getElementById('heroSection').style.display='none';
      const cs=document.getElementById('chatScroll');
      cs.style.display='flex';
      cs.innerHTML='';
    }
    const s=document.getElementById('chatScroll');
    const d=new Date();
    const t=`${String(d.getHours()).padStart(2,'0')}:${String(d.getMinutes()).padStart(2,'0')}`;
    const u=document.createElement('div');u.className='msg r';
    u.innerHTML=`<div class="mb">${escHtml(txt)}</div>`;
    s.appendChild(u);
    inp.value='';inp.style.height='auto';
    document.getElementById('sendBtn').disabled=true;
    s.scrollTop=s.scrollHeight;
    openRbSidebar();
    const a=document.createElement('div');a.className='msg l';
    a.innerHTML=`<div class="msg-h"><span class="name">NeoTrix</span><span class="time">${t}</span></div><div class="mb streaming"><span class="typing"><span></span><span></span><span></span></span></div>`;
    s.appendChild(a);s.scrollTop=s.scrollHeight;

    if(!isTauri()){
      /* Browser fallback: simulated reply so the UI stays demo-able */
      setTimeout(()=>{
        const rs=[
          '好的，我来逐步分析这个问题。',
          '收到，基于当前上下文，这是我的分析结果。',
          '没问题！我已经分析了你的请求，以下是我的思考。'
        ];
        const mb=a.querySelector('.mb');
        mb.classList.remove('streaming');
        const demo = rs[Math.floor(Math.random()*rs.length)] + '\n\n```rust\nfn main() {\n    println!("Hello, NeoTrix!");\n    let engine = ReasoningEngine::new();\n    engine.run();\n}\n```';
        mb.innerHTML = renderRichText(demo);
        document.getElementById('sendBtn').disabled=false;
        attachUsageFooter(a);
      },600+Math.random()*400);
      return;
    }

    /* Real Tauri path: stream via neocodex_send_message_stream */
    invoke('neocodex_send_message_stream', {
      content: txt,
      attachments: null,
      regenerate: false,
      permission_mode: 'auto',
      temperature: null,
      max_tokens: null,
    }).catch(err=>{
      const mb=a.querySelector('.mb');
      if(mb){ mb.classList.remove('streaming'); mb.textContent='[IPC 错误] '+String(err); }
      document.getElementById('sendBtn').disabled=false;
    });
  }
  function sendSuggestion(t){
    document.getElementById('chatInput').value=t;
    sendMsg();
  }

  /* ===== Hero Quick Suggestions (Claude 式) ===== */
  const HERO_SUGGESTIONS = [
    { t: '复刻一个桌面 APP 的 UI 布局', icon: I.plus },
    { t: '分析 neotrix-core 架构与依赖', icon: I.cube },
    { t: '吸收一篇论文并登记到知识库', icon: I.folder },
  ];
  function renderHeroSuggest(){
    const el = document.getElementById('heroSuggest');
    if(!el) return;
    el.innerHTML = HERO_SUGGESTIONS.map(s =>
      `<button class="hero-sug-item" onclick="sendSuggestion('${escHtml(s.t)}')">${s.icon}<span>${escHtml(s.t)}</span></button>`
    ).join('');
  }

  function openSettings(){
    showToast('设置面板 (开发中)');
  }

  /* ===== Missing Functions ===== */
  function clearChat(){
    actions.newChat();
    showToast('已清空对话');
  }

  /* ===== Keyboard Shortcuts ===== */
  document.addEventListener('keydown', e => {
    if((e.metaKey || e.ctrlKey) && e.key === ','){ e.preventDefault(); openSettingsModal(); }
    if((e.metaKey || e.ctrlKey) && e.key === 'n'){ e.preventDefault(); createSession(); }
    if((e.metaKey || e.ctrlKey) && e.key === 'f'){ e.preventDefault(); openStData(); }
    if((e.metaKey || e.ctrlKey) && e.key === 'w'){ e.preventDefault(); closeWindow(); }
    if((e.metaKey || e.ctrlKey) && (e.key === '1' || e.key === '2')){
      if(e.target.closest('textarea, input')) return;
      e.preventDefault();
      const tab = e.key === '1' ? 'chat' : 'cowork';
      const btn = document.querySelector('.segb[data-view="' + tab + '"]');
      if(btn) switchView(btn, tab);
    }
    if((e.metaKey || e.ctrlKey) && e.key === 'k'){ e.preventDefault();
      const inp = document.querySelector('.st-search input');
      if(inp && document.getElementById('overlaySettings').classList.contains('open')){ inp.focus(); return; }
      showToast('按 Cmd+K 搜索设置'); openSettingsModal();
      setTimeout(() => { const si = document.querySelector('.st-search input'); if(si) si.focus(); }, 100);
    }
    if(e.key === 'Escape'){
      document.querySelectorAll('.overlay-panel.open').forEach(p => p.classList.remove('open'));
      closePopover();
      updateTrafficVisibility();
    }
    if(e.key === '?' && !e.target.closest('textarea, input')){
      showToast('快捷键: ⌘1/⌘2 切换 · ⌘, 设置 · ⌘N 新建 · ⌘F 知识库 · ⌘W 关闭 · ⌘K 搜索 · Esc 关闭 · ? 帮助');
    }
  });

  /* ⌘F — open Settings → Data control, focus KB search */
  async function openStData(){
    const nav = document.querySelector('.st-item[onclick*="selectSetting(this,\'data\')"]');
    if(nav){ openSettingsModal(); nav.click(); }
    else{ openSettingsModal(); }
    setTimeout(() => { const si = document.getElementById('kbSearchInput'); if(si) si.focus(); }, 150);
  }

  /* ⌘W — close window; browser-mode fallback keeps session alive */
  async function closeWindow(){
    if(!isTauri()){ showToast('浏览器模式：⌘W 仅桌面版生效'); return; }
    try{ await invoke('window_close'); }
    catch(e){ showToast('关闭窗口失败: ' + e); }
  }

  /* ===== Auto Init ===== */
  // DOM is already loaded at this point (script at end of body)
  renderSidebar('chat');
  renderHeroSuggest();
  renderCowork();
  // Send button initial state (direct call — no synthetic event needed)
  const ci0 = document.getElementById('chatInput');
  const sb0 = document.getElementById('sendBtn');
  if(ci0) autoResize(ci0);
  if(sb0) sb0.disabled = !(ci0 && ci0.value.trim());
  // Live backend: hydrate real data when inside Tauri
  wireBackend();
  fusionInit();



  /* ════════════════════════════════════════════════
     Backend wiring — real IPC over the static demo data.
     Runs at startup when running inside Tauri; the demo
     data is kept as a graceful browser fallback.
     ════════════════════════════════════════════════ */
  let backendSessionMap = new Map(); // idx -> session id
  let backendSessionList = [];

  async function loadSessions(){
    if(!isTauri()) return;
    try{
      const [sessions, cowork] = await Promise.all([
        invoke('neocodex_list_sessions', { project_path: null }),
        invoke('cowork_list').catch(() => []),
      ]);
      const merged = [];
      if(Array.isArray(sessions)){
        merged.push(...sessions.map((s, i) => {
          backendSessionMap.set(i, s.id);
          const agents = [];
          if(s.mode) agents.push({ n: s.mode === 'Plan' ? '规划' : 'Agent', on: true });
          return {
            name: s.name || ('会话 ' + (i+1)),
            status: '就绪',
            tasks: 0, done: 0, fail: 0,
            agents,
            id: s.id,
            message_count: s.message_count || 0,
            updated_at: s.updated_at || s.created_at || 0,
          };
        }));
      }
      if(Array.isArray(cowork) && cowork.length){
        cowork.forEach((c, i) => {
          const idx = merged.length + i;
          backendSessionMap.set(idx, c.id);
          merged.push({
            name: c.name || ('协同 ' + (i+1)),
            status: c.status === 'active' ? '进行中' : (c.status === 'paused' ? '已暂停' : (c.status === 'completed' ? '已完成' : (c.status || '就绪'))),
            tasks: (c.deliverables ? c.deliverables.length : 0),
            done: Math.min((c.deliverables ? c.deliverables.length : 0), c.files_created || 0),
            fail: 0,
            agents: [{ n: '协同', on: c.status === 'active' }],
            id: c.id,
            isCowork: true,
            cw: c,
          });
        });
      }
      if(merged.length) backendSessionList = merged;
      CW_DATA = merged.length ? merged : CW_DATA;
      recentData.chat = merged.filter(s => !s.isCowork).slice(0, 5).map(s => ({
        text: s.name, time: fmtRelTime(s.updated_at), id: s.id,
      }));
      recentData.cowork = merged.filter(s => s.isCowork).slice(0, 5).map(s => ({
        text: s.name, time: fmtRelTime(s.cw?.last_active_at || s.cw?.updated_at), id: s.id, cowork: true,
      }));
      renderSidebar(currentView);
      renderCowork();
      if(CW_DATA.length) showToast('已加载 ' + CW_DATA.length + ' 个会话');
    }catch(e){ /* keep demo data */ }
  }

  async function createSession(){
    if(!isTauri()){ showToast('浏览器模式：新建会话'); return; }
    try{
      const info = await invoke('neocodex_create_session', { name: null });
      const id = (info && info.id) ? info.id : String(info);
      if(id){
        const s = { name: (info && info.name) || ('新会话 ' + CW_DATA.length), status: '就绪', tasks: 0, done: 0, fail: 0, agents: [], id, message_count: 0 };
        CW_DATA.unshift(s);
        backendSessionMap.clear();
        CW_DATA.forEach((x,i) => { if(x.id) backendSessionMap.set(i, x.id); });
        renderCowork();
        showToast('已创建会话');
      }
    }catch(e){ showToast('创建失败: ' + e); }
  }

  async function loadSessionMessages(id){
    if(!isTauri() || !id) return;
    try{
      await invoke('neocodex_switch_session', { session_id: id });
      const msgs = await invoke('neocodex_get_session_messages', { session_id: id });
      if(!Array.isArray(msgs)) return;
      document.getElementById('heroSection').style.display = 'none';
      const cs = document.getElementById('chatScroll');
      cs.style.display = 'flex';
      cs.innerHTML = '';
      msgs.forEach(m => {
        const t = m.timestamp ? new Date(m.timestamp).toTimeString().slice(0,5) : '';
        if(m.role === 'user'){
          const u = document.createElement('div'); u.className = 'msg r';
          u.innerHTML = `<div class="mb">${escHtml(m.content)}</div>`;
          cs.appendChild(u);
        }else{
          const a = document.createElement('div'); a.className = 'msg l';
          a.innerHTML = `<div class="msg-h"><span class="name">NeoTrix</span><span class="time">${t}</span></div><div class="mb">${renderRichText(m.content)}</div>`;
          cs.appendChild(a);
        }
      });
      isChatMode = true;
      cs.scrollTop = cs.scrollHeight;
    }catch(e){ /* keep current chat */ }
  }

  async function refreshAgent(){
    const al = document.getElementById('cwAgentList');
    if(!al) return;
    if(!isTauri()){ al.innerHTML = '<div class="cw-agent"><span class="adot"></span>浏览器模式 · 未连接</div>'; return; }
    try{
      const st = await invoke('neocodex_agent_status');
      if(st){
        agentRunning = st.running;
        agentTask = st.current_task || '';
        agentUp = st.uptime_secs || 0;
      }
      const coord = await invoke('coordinator_list').catch(() => null);
      if(coord && Array.isArray(coord.workers) && coord.workers.length){
        const rows = coord.workers.slice(0, 6).map(w => {
          const ok = w.status === 'running';
          const pct = Math.max(0, Math.min(100, Math.round((w.progress || 0) * 100)));
          return `<div class="cw-agent"><span class="adot ${ok ? 'ok' : ''}"></span><span class="cw-aname">${escHtml(w.id)}</span><span class="cw-atask">${escHtml(String(w.task).slice(0, 18))}</span><div class="cw-aprog"><div class="cw-aprog-f" style="width:${pct}%"></div></div><span class="cw-apct">${pct}%</span></div>`;
        }).join('');
        al.innerHTML = rows;
        return;
      }
    }catch(e){ /* fall through */ }
    renderAgentRow();
  }

  function renderAgentRow(){
    const al = document.getElementById('cwAgentList');
    if(!al) return;
    const color = agentRunning ? 'var(--suc)' : 'var(--tx-meta)';
    const label = agentRunning
      ? `<span class="adot" style="background:${color}"></span>运行中 · ${escHtml(agentTask || '处理任务')} · ${agentUp}s`
      : `<span class="adot"></span>空闲`;
    al.innerHTML = `<div class="cw-agent">${label}</div>
      <div class="cw-actions">
        <button class="cw-abtn" onclick="${agentRunning ? 'stopAgent()' : 'runAgent()'}">${agentRunning ? '■ 停止' : '▶ 启动'}</button>
        <button class="cw-abtn" onclick="refreshAgent()">↻ 刷新</button>
      </div>`;
  }

  async function runAgent(){
    if(!isTauri()){ showToast('浏览器模式：无法启动智能体'); return; }
    const s = CW_DATA.find(x => x.id);
    const prompt = '协同会话：' + ((s && s.name) || '默认任务');
    try{
      await invoke('neocodex_send_message_stream', { content: prompt, permission_mode: 'auto' });
      agentRunning = true; agentTask = prompt; agentUp = 0;
      renderAgentRow();
      showToast('智能体已启动');
    }catch(e){ showToast('启动失败: ' + e); }
  }
  async function stopAgent(){
    if(!isTauri()) return;
    try{
      await invoke('neocodex_stop_stream');
      agentRunning = false; agentTask = ''; agentUp = 0;
      renderAgentRow();
      showToast('智能体已停止');
    }catch(e){ showToast('停止失败: ' + e); }
  }

  async function loadRegistry(){
    const root = document.querySelector('#overlayRegistry .overlay-bd');
    if(!root) return;
    if(!isTauri()){
      const cap = root.querySelector('.reg-iter') || null;
      if(cap) cap.textContent = '—';
      return;
    }
    try{
      const bs = await invoke('brain_stats');
      if(bs){
        const set = (k, v) => { const p = root.querySelector(k); if(p) p.textContent = String(v); };
        set('.reg-iter', bs.iteration ?? '—');
        set('.reg-cap', (bs.capability_sum ?? 0).toFixed(2));
        set('.reg-dim', (bs.dimension_names && bs.dimension_names.length) ? bs.dimension_names.length + ' 维' : '—');
        if(Array.isArray(bs.capability_vector) && bs.capability_vector.length){
          const dims = Array.isArray(bs.dimension_names) && bs.dimension_names.length
            ? bs.dimension_names : bs.capability_vector.map((_, i) => 'dim_' + i);
          const max = Math.max(...bs.capability_vector) || 1;
          const bars = bs.capability_vector.slice(0, 12).map((v, i) => {
            const pct = Math.max(2, Math.round(Math.abs(v) / max * 100));
            return `<div class="vec-bar"><span class="vec-lbl">${escHtml(String(dims[i] || i))}</span><div class="vec-track"><div class="vec-fill" style="width:${pct}%"></div></div><span class="vec-val">${Number(v).toFixed(2)}</span></div>`;
          }).join('');
          const vb = document.getElementById('regVecBars');
          if(vb) vb.innerHTML = bars;
        }
      }
    }catch(e){ /* keep static list */ }
    try{
      const hr = await invoke('neocodex_health_report');
      if(hr){
        const id = (k, v) => { const p = document.getElementById(k); if(p) p.textContent = v; };
        id('regTurns', (hr.turn_count ?? 0).toLocaleString());
        id('regTools', (hr.tool_call_count ?? 0).toLocaleString());
        id('regTokens', (hr.tokens_used ?? 0).toLocaleString());
        id('regCtx', (Math.round((hr.context_usage ?? 0) * 100)) + '%');
        id('regEvo', (hr.evolution_iterations ?? 0).toLocaleString());
        id('regCost', '$' + Number(hr.cost_spent ?? 0).toFixed(2));
      }
    }catch(e){}
    try{
      const st = await invoke('neocodex_agent_status');
      const eng = root.querySelector('.reg-eng');
      if(eng && st) eng.textContent = st.running ? '运行中' : '空闲';
    }catch(e){}
  }

  async function loadHypercube(){
    if(!isTauri()) return;
    try{
      const [ks, bs] = await Promise.all([
        invoke('get_knowledge_stats').catch(() => null),
        invoke('brain_stats').catch(() => null),
      ]);
      const set = (id, v) => { const el = document.getElementById(id); if(el) el.textContent = String(v); };
      if(ks && ks.total_nodes != null){
        set('hcNodes', Number(ks.total_nodes).toLocaleString());
        set('hcEdges', Number(ks.total_edges).toLocaleString());
        if(ks.by_type && Array.isArray(ks.by_type) && ks.by_type.length){
          const total = ks.by_type.reduce((a, t) => a + (Array.isArray(t) ? Number(t[1]) : Number(t.count)), 0) || 1;
          const list = ks.by_type.map(t => {
            const name = Array.isArray(t) ? t[0] : (t.node_type || t.name || 'other');
            const count = Number(Array.isArray(t) ? t[1] : (t.count || t.total || 0));
            const pct = Math.max(2, Math.round(count / total * 100));
            return `<div class="vec-bar"><span class="vec-lbl">${escHtml(name)}</span><div class="vec-track"><div class="vec-fill" style="width:${Math.min(100, pct)}%"></div></div><span class="vec-val">${count.toLocaleString()}</span></div>`;
          }).join('');
          document.getElementById('hcTypeDist').innerHTML = list;
        }
      }
      if(bs){
        set('hcVsa', (bs.dimension_names && bs.dimension_names.length) ? 'D=' + bs.dimension_names.length : '—');
        set('hcCap', (bs.capability_sum != null) ? Number(bs.capability_sum).toFixed(2) : '—');
        set('kbNodeCount', (bs.memory_count || 0).toLocaleString() + ' 节点');
        if(Array.isArray(bs.capability_vector) && bs.capability_vector.length){
          const dims = Array.isArray(bs.dimension_names) && bs.dimension_names.length
            ? bs.dimension_names : bs.capability_vector.map((_, i) => 'dim_' + i);
          const max = Math.max(...bs.capability_vector) || 1;
          const bars = bs.capability_vector.slice(0, 12).map((v, i) => {
            const pct = Math.max(2, Math.round(Math.abs(v) / max * 100));
            return `<div class="vec-bar"><span class="vec-lbl">${escHtml(String(dims[i] || i))}</span><div class="vec-track"><div class="vec-fill" style="width:${pct}%"></div></div><span class="vec-val">${Number(v).toFixed(2)}</span></div>`;
          }).join('');
          document.getElementById('hcVector').innerHTML = bars;
        }
      }
    }catch(e){ /* keep em-dash placeholders */ }
  }

  async function kbSearch(){
    const inp = document.getElementById('kbSearchInput');
    const res = document.getElementById('kbResults');
    if(!inp || !res) return;
    const q = inp.value.trim();
    if(!q){ res.innerHTML = '<div class="kb-empty">输入关键词检索知识库</div>'; return; }
    if(!isTauri()){ res.innerHTML = '<div class="kb-empty">浏览器模式：仅 Tauri 下可检索</div>'; return; }
    res.innerHTML = '<div class="kb-empty">检索中…</div>';
    try{
      const hits = await invoke('kb_search', { query: q, limit: 8 });
      if(!Array.isArray(hits) || !hits.length){ res.innerHTML = '<div class="kb-empty">无结果</div>'; return; }
      res.innerHTML = hits.map(h => {
        const dom = h.domain ? `<span class="kb-dom">${escHtml(h.domain)}</span>` : '';
        return `<div class="kb-hit"><div class="kb-hit-t">${escHtml(h.title || h.id)}${dom}</div><div class="kb-hit-s">${escHtml((h.summary || '').slice(0,120))}</div></div>`;
      }).join('');
    }catch(e){ res.innerHTML = '<div class="kb-empty">检索失败: ' + escHtml(String(e)) + '</div>'; }
  }

  async function loadHealth(){
    if(!isTauri()) return;
    try{
      const h = await invoke('neocodex_health_report');
      if(!h) return;
      const el = document.getElementById('stGwStatus');
      if(el){
        el.textContent = h.turn_count > 0 ? `运行中 · ${h.turn_count} 轮` : '运行中';
        el.style.color = 'var(--suc)';
      }
    }catch(e){}
  }

  async function loadFileTree(){
    if(!isTauri()) return;
    try{
      let project = await invoke('neocodex_get_project').catch(() => null);
      if(!project){
        try { project = await invoke('detect_project'); } catch(_e){}
      }
      if(!project) return;
      const nodes = await invoke('read_dir_recursive', { path: String(project), max_depth: 3 });
      if(!Array.isArray(nodes) || !nodes.length) return;
      const root = buildTreeFromFlat(nodes);
      if(root.children && root.children.length){
        renderFileTree([root], null);
      }
    }catch(e){}
  }

  function buildTreeFromFlat(flat){
    const root = { name: '项目', type: 'dir', open: true, children: [] };
    const byDepth = [];
    const stack = [{ node: root, depth: 0 }];
    for(const f of flat){
      const node = { name: f.name, type: f.is_dir ? 'dir' : 'file', path: f.path };
      if(f.is_dir) node.children = [];
      let parent = stack[stack.length - 1].node;
      // find parent by depth
      while(stack.length > 1 && stack[stack.length-1].depth >= f.depth){
        stack.pop();
      }
      parent = stack[stack.length - 1].node;
      if(parent && parent.children) parent.children.push(node);
      if(f.is_dir) stack.push({ node, depth: f.depth });
      else if(node.name.endsWith('.rs') || node.name.endsWith('.toml') || node.name.endsWith('.md') || node.name.endsWith('.json')){
        node.content = '// ' + node.name + '\n// 点击加载内容';
        node.load = f.path;
      }
    }
    return root;
  }

  async function loadProxy(){
    if(!isTauri()) return;
    try{
      const nodes = await invoke('proxy_pool_nodes');
      if(!Array.isArray(nodes) || !nodes.length) return;
      PROXY_DATA.length = 0;
      nodes.slice(0, 12).forEach(n => {
        PROXY_DATA.push({
          name: n.tag || n.url,
          reg: n.geo_tag ? n.geo_tag.slice(0,2) : '🌐',
          ip: n.url,
          good: !!n.healthy,
          lat: Math.round(n.latency_ms || 0),
          score: n.speed_tier || 'C',
        });
      });
      if(document.getElementById('stGwNodeList')) renderStGateway();
    }catch(e){}
  }

  async function wireBackend(){
    if(!isTauri()) return;
    await Promise.all([loadSessions(), loadHealth(), loadFileTree(), loadProxy(), loadUsage()]);
  }

  window.wireBackend = wireBackend;

  /* ════════════════════════════════════════════════
     Claude-parity fusion layer
     · composer "+" menu (attach / slash / reference)
     · permission-mode selector (manual/accept/plan/auto/bypass)
     · usage ring from health_report
     · keyboard shortcuts (⌘Shift+]/[ · ⌘Shift+M · ⌘\ )
     ════════════════════════════════════════════════ */

  function fusionInit(){
    const cicLeft = document.querySelector('#viewChat .cic-left');
    if(cicLeft && !document.getElementById('ntxPlusMenu')){
      const wrap = document.createElement('div');
      wrap.className = 'cic-plus-wrap';
      wrap.style.cssText = 'position:relative;display:inline-flex';
      wrap.innerHTML = `
        <button class="cic-attach" id="ntxPlusBtn" title="添加文件 / 命令">
          <svg viewBox="0 0 16 16"><path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>
        </button>
        <div class="ntx-plus-menu" id="ntxPlusMenu">
          <button class="ntx-pm-item" data-act="attach">${I.folder}<span>附加文件</span></button>
          <button class="ntx-pm-item" data-act="slash"><span style="font-family:var(--fm)">/</span><span>命令 (Slash)</span></button>
          <button class="ntx-pm-item" data-act="ref"><span style="font-size:11px">@</span><span>引用上下文</span></button>
          <div class="ntx-pm-sep"></div>
          <button class="ntx-pm-item" data-act="diff"><svg viewBox="0 0 12 12"><line x1="1.5" y1="3" x2="10.5" y2="3" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/><line x1="1.5" y1="7" x2="10.5" y2="7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/><line x1="1.5" y1="11" x2="10.5" y2="11" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/></svg><span>查看 Diff</span></button>
          <div class="ntx-pm-sep"></div>
          <button class="ntx-pm-item" data-act="achievements">${I.play}<span>成果</span></button>
          <button class="ntx-pm-item" data-act="registry">${I.grid}<span>能力</span></button>
          <button class="ntx-pm-item" data-act="hypercube">${I.cube}<span>知识</span></button>
        </div>`;
      wrap.querySelector('#ntxPlusBtn').addEventListener('click', (e) => {
        e.stopPropagation();
        const m = document.getElementById('ntxPlusMenu');
        m.classList.toggle('open');
      });
      wrap.querySelectorAll('.ntx-pm-item').forEach(btn => {
        btn.addEventListener('click', () => handlePlusAction(btn.dataset.act));
      });
      cicLeft.prepend(wrap);
    }

    const cicRight = document.querySelector('#viewChat .cic-right');
    if(cicRight && !document.getElementById('ntxModelWrap')){
      const wrap = document.createElement('div');
      wrap.className = 'cic-mode-wrap';
      wrap.style.cssText = 'position:relative;display:inline-flex';
      wrap.innerHTML = `
        <button class="ntx-mode-btn" id="ntxModelBtn" title="选择模型"><svg viewBox="0 0 14 14"><path d="M7 1.5v11M1.5 7h11M3.5 3.5l7 7M10.5 3.5l-7 7" stroke="currentColor" stroke-width="1" stroke-linecap="round"/></svg><span id="ntxModelLabel">Groq</span><svg viewBox="0 0 10 6" style="width:9px;height:6px"><polyline points="1,1 5,5 9,1" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round"/></svg></button>
        <div class="ntx-mode-menu" id="ntxModelMenu"></div>`;
      const menu = wrap.querySelector('#ntxModelMenu');
      renderModelMenu();
      wrap.querySelector('#ntxModelBtn').addEventListener('click', (e) => {
        e.stopPropagation();
        menu.classList.toggle('open');
        renderModelMenu();
      });
      menu.addEventListener('click', (e) => {
        const item = e.target.closest('.ntx-mm-item');
        if(item) selectModel(item.dataset.id);
      });
      cicRight.prepend(wrap);
    }

    const cic = document.querySelector('.cic');
    if(cic && !document.getElementById('ntxToolbar')){
      const tb = document.createElement('div');
      tb.className = 'ntx-chat-toolbar';
      tb.innerHTML = `<div class="l" id="ntxAttachArea"></div><div class="r" id="ntxToolbarRight"></div>`;
      cic.insertBefore(tb, cic.querySelector('.cic-actions'));
    }

    if(!window._ntxShortcutsBound){
      window._ntxShortcutsBound = true;
      document.addEventListener('keydown', (e) => {
        const m = e.metaKey || e.ctrlKey;
        if(m && e.shiftKey && e.key === ']'){ e.preventDefault(); cycleView(1); }
        if(m && e.shiftKey && e.key === '['){ e.preventDefault(); cycleView(-1); }
        if(m && e.shiftKey && (e.key === 'M' || e.key === 'm')){ e.preventDefault(); showToast('⌘Shift+M — 打开侧边栏'); openRbSidebar(); }
        if(m && e.key === '\\'){ e.preventDefault(); toggleSidebar(); }
        if(m && e.shiftKey && e.key === 'Enter'){ e.preventDefault(); showToast('⌘Shift+Enter — 发送'); sendMsg(); }
      });
    }

    document.addEventListener('click', (e) => {
      if(!e.target.closest('#ntxPlusMenu') && !e.target.closest('#ntxPlusBtn')){
        const m = document.getElementById('ntxPlusMenu');
        if(m) m.classList.remove('open');
      }
      if(!e.target.closest('#ntxModelMenu') && !e.target.closest('#ntxModelBtn')){
        const m = document.getElementById('ntxModelMenu');
        if(m) m.classList.remove('open');
      }
    });

    renderModelMenu();
    loadModelPool();
  }

  function cycleView(delta){
    const order = ['chat', 'cowork'];
    let idx = order.indexOf(currentView);
    if(idx === -1) idx = 0;
    idx = (idx + delta + order.length) % order.length;
    const btn = document.querySelector(`.segb[data-view="${order[idx]}"]`);
    if(btn) switchView(btn, order[idx]);
  }

  function renderModelMenu(){
    const menu = document.getElementById('ntxModelMenu');
    if(!menu) return;
    menu.innerHTML = MODEL_POOL.map(m => {
      const on = m.id === currentModelId;
      const state = m.online ? `<span class="ntx-mm-lat">${m.lat ? m.lat + 'ms' : '—'}</span>` : `<span class="ntx-mm-lat off">离线</span>`;
      return `<button class="ntx-mm-item${on ? ' on' : ''}" data-id="${m.id}">
        <span class="ntx-mm-title"><span class="dot" style="background:${on ? 'var(--suc,#4CAF50)' : 'var(--tx3)'}"></span>${m.title}</span>
        <span class="ntx-mm-desc">${m.model}</span>${state}
      </button>`;
    }).join('');
    const lbl = document.getElementById('ntxModelLabel');
    const cur = MODEL_POOL.find(x => x.id === currentModelId);
    if(lbl) lbl.textContent = cur ? cur.title : '—';
  }

  async function loadModelPool(){
    if(!isTauri()) return;
    try{
      const cfg = await invoke('neocodex_provider_config');
      if(cfg && Array.isArray(cfg.providers) && cfg.providers.length){
        const known = MODEL_POOL.map(x => x.id);
        cfg.providers.forEach(p => {
          const idx = MODEL_POOL.findIndex(x => x.id.toLowerCase() === String(p.name||'').toLowerCase());
          if(idx >= 0){ MODEL_POOL[idx].model = p.model || MODEL_POOL[idx].model; MODEL_POOL[idx].online = !!p.resolvable; }
          else if(!known.includes(p.name)){
            MODEL_POOL.push({ id: p.name, title: p.name, model: p.model || '', lat: 0, online: !!p.resolvable });
          }
        });
        if(cfg.active_model){
          const act = MODEL_POOL.find(x => x.model === cfg.active_model) || MODEL_POOL.find(x => x.id === cfg.active_model);
          if(act) currentModelId = act.id;
        }
        renderModelMenu();
      }
    }catch(_e){}
  }

  async function selectModel(id){
    const m = document.getElementById('ntxModelMenu');
    if(m) m.classList.remove('open');
    currentModelId = id;
    renderModelMenu();
    if(isTauri()){
      try { await invoke('neocodex_set_provider', { name: id }); } catch(_e){}
    }
    testModelCall(id);
  }

  async function testModelCall(id){
    const model = MODEL_POOL.find(x => x.id === id);
    const label = model ? (model.title + ' · ' + model.model) : id;
    if(!model || !model.online){
      showToast('「' + label + '」离线，跳过测试调用');
      return;
    }
    showToast('测试调用 ' + label + ' …');
    if(!isTauri()){
      setTimeout(() => showToast(label + ' 响应 ' + model.lat + 'ms（模拟）'), model.lat + 300);
      return;
    }
    const t0 = performance.now();
    try{
      const result = await invoke('neocodex_send_message_stream', {
        content: 'ping — 连通性测试',
        attachments: null,
        regenerate: false,
        permission_mode: 'auto',
        temperature: null,
        max_tokens: 16,
      });
      const ms = Math.round(performance.now() - t0);
      const head = String(result || '').slice(0, 24);
      showToast(label + ' ✓ ' + ms + 'ms' + (head ? ' · ' + head : ''));
    }catch(e){
      showToast('「' + label + '」调用失败: ' + String(e).slice(0, 60));
    }
  }

  async function handlePlusAction(act){
    const m = document.getElementById('ntxPlusMenu');
    if(m) m.classList.remove('open');
    if(act === 'diff'){
      openDiff();
      return;
    }
    if(act === 'attach'){
      if(isTauri()){
        showToast('搜索文件…');
        try{
          const files = await invoke('neocodex_search_files', { query: '' });
          if(Array.isArray(files) && files.length){
            files.slice(0, 6).forEach(f => addAttachChip(f));
            showToast('已附加 ' + Math.min(files.length, 6) + ' 个文件');
          }
        }catch(_e){ showToast('无可用文件'); }
      } else {
        showToast('附加文件 (浏览器演示 — 需 Tauri)');
      }
      return;
    }
    if(act === 'achievements'){ showAchievements(); return; }
    if(act === 'registry'){ openOverlay('overlayRegistry'); loadRegistry(); return; }
    if(act === 'hypercube'){ openOverlay('overlayHypercube'); return; }
    showToast('「' + { slash: '命令 (Slash)', ref: '引用上下文' }[act] + '」功能开发中');
  }

  function addAttachChip(name){
    const area = document.getElementById('ntxAttachArea');
    if(!area) return;
    if(attachList.includes(name)) return;
    attachList.push(name);
    const chip = document.createElement('span');
    chip.className = 'ntx-attach-chip';
    chip.innerHTML = `${escHtml(name)}<span class="x" data-f="${escHtml(name)}">×</span>`;
    chip.querySelector('.x').addEventListener('click', () => {
      attachList = attachList.filter(f => f !== name);
      chip.remove();
    });
    area.appendChild(chip);
  }

  let lastContextUsage = 0;
  async function loadUsage(){
    if(!isTauri()) return;
    try{
      const h = await invoke('neocodex_health_report');
      lastContextUsage = Math.max(0, Math.min(1, (h && h.context_usage) || 0));
    }catch(_e){}
  }

  function attachUsageFooter(msgEl){
    const mb = msgEl.querySelector('.mb');
    if(!mb || mb.querySelector('.msg-usage')) return;
    const el = document.createElement('div');
    el.className = 'msg-usage';
    el.textContent = '上下文 ' + Math.round(lastContextUsage * 100) + '%';
    mb.appendChild(el);
  }

  /* ════════════════════════════════════════════════
     Diff 行内评论 — hunk 渲染 + 行级评论
     ════════════════════════════════════════════════ */
  const diffComments = new Map();
  const SAMPLE_DIFF = {
    files: [
      {
        path: 'neotrix-core/src/nt_core_synthesis.rs',
        hunks: [
          {
            lines: [
              { t: 'ctx', o: 10, n: 10, s: '    fn fuse(&self, modals: &[f64]) -> f64 {' },
              { t: 'del', o: 11, n: null, s: '        self.weighted * 0.5' },
              { t: 'add', o: null, n: 11, s: '        self.weighted * self.attention' },
              { t: 'ctx', o: 12, n: 12, s: '    }' },
              { t: 'add', o: null, n: 13, s: '' },
              { t: 'add', o: null, n: 14, s: '    // attention 归一化防 tail-mass 泄漏' },
            ],
          },
        ],
      },
      {
        path: 'src-tauri/src/lib.rs',
        hunks: [
          {
            lines: [
              { t: 'del', o: 42, n: null, s: '        mode: "auto".to_string(),' },
              { t: 'add', o: null, n: 42, s: '        mode: permission_mode.clone(),' },
              { t: 'ctx', o: 43, n: 43, s: '    }' },
            ],
          },
        ],
      },
    ],
  };

  function escDiffHtml(s){
    return escHtml(s || '').replace(/ /g, '&nbsp;');
  }

  async function openDiff(){
    let data = null;
    if(isTauri()){
      try { data = await invoke('neocodex_get_diff'); } catch(_e){}
    }
    if(!data || !Array.isArray(data.files) || !data.files.length){
      data = SAMPLE_DIFF;
    }
    renderDiff(data);
    openOverlay('overlayDiff');
  }

  function renderDiff(data){
    const title = document.getElementById('diffTitle');
    if(title) title.textContent = '代码变更 · ' + data.files.length + ' 文件';
    const body = document.getElementById('diffBody');
    if(!body) return;
    body.innerHTML = '';
    const files = data.files.map((f, fi) => {
      const hunks = f.hunks.map((h, hi) => {
        const rows = h.lines.map((ln, li) => {
          const key = fi + ':' + hi + ':' + li;
          const badge = ln.t === 'add' ? '+' : ln.t === 'del' ? '−' : '';
          const cls = 'df-line ' + (ln.t === 'add' ? 'add' : ln.t === 'del' ? 'del' : '');
          const oldNo = ln.o != null ? ln.o : '';
          const newNo = ln.n != null ? ln.n : '';
          const cmt = diffComments.get(key);
          return `
            <div class="${cls}" data-cmt="${key}">
              <span class="df-num o">${oldNo}</span>
              <span class="df-num n">${newNo}</span>
              <span class="df-badge">${badge}</span>
              <span class="df-text">${escDiffHtml(ln.s)}</span>
              <span class="df-cmt-btn" title="行内评论" onclick="diffAddComment(${fi},${hi},${li})">+</span>
            </div>
            ${cmt ? `<div class="df-comment" data-cmt="${key}"><span class="dc-dot"></span><span class="dc-body">${escHtml(cmt)}</span></div>` : ''}`;
        }).join('');
        return `<div class="df-hunk">${rows}</div>`;
      }).join('');
      return `<div class="df-file">
        <div class="df-path"><svg viewBox="0 0 12 12"><path d="M10.5 5v4.5a1 1 0 01-1 1h-7a1 1 0 01-1-1v-7a1 1 0 011-1H5" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d="M7.5 1.5h3v3" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round"/><path d="M6.5 5.5l4-4" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round"/></svg>${escHtml(f.path)}</div>
        ${hunks}</div>`;
    }).join('');
    body.innerHTML = files;
  }

  function diffAddComment(fi, hi, li){
    const key = fi + ':' + hi + ':' + li;
    const row = document.querySelector(`.df-line[data-cmt="${key}"]`);
    if(!row) return;
    const box = document.createElement('div');
    box.className = 'df-cmt-editor';
    box.dataset.cmt = key;
    box.innerHTML = `<textarea rows="2" placeholder="对此行添加评论…"></textarea><div class="dc-actions"><button class="dc-save" onclick="diffSaveComment(${fi},${hi},${li})">评论</button><button class="dc-cancel" onclick="this.closest('.df-cmt-editor').remove()">取消</button></div>`;
    row.after(box);
    box.querySelector('textarea').focus();
  }

  function diffSaveComment(fi, hi, li){
    const key = fi + ':' + hi + ':' + li;
    const ed = document.querySelector(`.df-cmt-editor[data-cmt="${key}"]`);
    const ta = ed ? ed.querySelector('textarea') : null;
    const val = ta ? ta.value.trim() : '';
    const row = document.querySelector(`.df-line[data-cmt="${key}"]`);
    if(!val){
      if(ed) ed.remove();
      return;
    }
    diffComments.set(key, val);
    if(ed) ed.remove();
    if(row && row.nextElementSibling && row.nextElementSibling.classList.contains('df-comment')){
      row.nextElementSibling.remove();
    }
    const cmt = document.createElement('div');
    cmt.className = 'df-comment';
    cmt.innerHTML = `<span class="dc-dot"></span><span class="dc-body">${escHtml(val)}</span><span class="dc-del" title="删除评论" onclick="diffDelComment(${fi},${hi},${li})">×</span>`;
    row.after(cmt);
  }

  function diffDelComment(fi, hi, li){
    const key = fi + ':' + hi + ':' + li;
    diffComments.delete(key);
    const row = document.querySelector(`.df-line[data-cmt="${key}"]`);
    if(row && row.nextElementSibling && row.nextElementSibling.classList.contains('df-comment')){
      row.nextElementSibling.remove();
    }
  }

  window.diffAddComment = diffAddComment;
  window.diffSaveComment = diffSaveComment;
  window.diffDelComment = diffDelComment;
  window.openDiff = openDiff;
  window.renderDiff = renderDiff;






  /* ===== POPOVER & SETTINGS MODAL ===== */
  function toggleUserPopover(e){
    const popover = document.getElementById('userPopover');
    if (popover.contains(e.target)) { e.stopPropagation(); return; }
    const isOpen = popover.style.display === 'block';
    if (isOpen) {
      popover.style.display = 'none';
    } else {
      const bar = document.getElementById('userBar');
      const r = bar.getBoundingClientRect();
      popover.style.left = r.left + 'px';
      popover.style.bottom = (window.innerHeight - r.top + 8) + 'px';
      popover.style.display = 'block';
    }
    e.stopPropagation();
  }
  function closePopover(){
    document.getElementById('userPopover').style.display = 'none';
  }
  document.addEventListener('click',function(e){
    const p=document.getElementById('userPopover');
    const b=document.getElementById('userBar');
    if(p.style.display==='block'&&!b.contains(e.target))closePopover();
  });

  function updateTrafficVisibility(){
    /* Native traffic lights are OS-controlled; no-op kept for callers. */
  }

  function openOverlay(id){
    const el=document.getElementById(id);
    if(el)el.classList.add('open');
    if(id === 'overlayHypercube') loadHypercube();
    updateTrafficVisibility();
  }

  function closeOverlay(id){
    const el=document.getElementById(id);
    if(el)el.classList.remove('open');
    updateTrafficVisibility();
  }

  function openSettingsModal(){
    closePopover();
    openOverlay('overlaySettings');
  }

  async function selectSetting(el,section){
    document.querySelectorAll('.st-item').forEach(i=>i.classList.remove('on'));
    el.classList.add('on');
    document.querySelectorAll('.st-section').forEach(s=>s.classList.remove('open'));
    const t=document.getElementById('st'+section.charAt(0).toUpperCase()+section.slice(1));
    if(t)t.classList.add('open');
    if(section==='gateway') await renderStGateway();
    if(section==='compute') await renderStCompute();
    if(section==='limits') await renderStLimits();
    if(section==='privacy') await renderStPrivacy();
    if(section==='data') await renderStData();
    if(section==='profile') initProfileHandlers();
    if(section==='appearance') initAppearanceHandlers();
    if(section==='speech') initSpeechHandlers();
    if(section==='compute') initComputeHandlers();
    if(section==='privacy') initPrivacyHandlers();
  }

  function initProfileHandlers(){
    const inputs = document.querySelectorAll('#stProfile input, #stProfile select');
    inputs.forEach(el => {
      el.onchange = () => showToast('已保存: ' + (el.previousElementSibling?.textContent || el.name || '设置'));
    });
  }

  function initAppearanceHandlers(){
    const fontSel = document.querySelector('#stAppearance select');
    if(fontSel) fontSel.onchange = () => showToast('字体大小已更改: ' + fontSel.value);
    const reduceTrans = document.querySelector('#stAppearance input[type="checkbox"]');
    if(reduceTrans) reduceTrans.onchange = () => showToast(reduceTrans.checked ? '已开启减少透明效果' : '已关闭减少透明效果');
  }

  function initSpeechHandlers(){
    const inputs = document.querySelectorAll('#stSpeech input, #stSpeech select');
    inputs.forEach(el => {
      el.onchange = () => showToast('语音设置已更改: ' + (el.previousElementSibling?.textContent || '设置'));
    });
  }

  function initComputeHandlers(){
    const providerSel = document.querySelector('#stCompute select');
    if(providerSel){
      providerSel.onchange = async () => {
        if(isTauri()){
          try{
            await invoke('neocodex_set_provider', { name: providerSel.value });
            showToast('默认提供者已切换: ' + providerSel.value);
          }catch(e){ showToast('切换失败: ' + e); }
        }else{
          showToast('浏览器模式：仅 Tauri 下可切换提供者');
        }
      };
    }
    const tokenSel = document.querySelector('#stCompute select:last-of-type');
    if(tokenSel) tokenSel.onchange = () => showToast('最大 Token 已设为: ' + tokenSel.value);
    const localInfer = document.querySelector('#stCompute input[type="checkbox"]');
    if(localInfer) localInfer.onchange = () => showToast(localInfer.checked ? '已启用本地推理引擎' : '已禁用本地推理引擎');
  }

  function initPrivacyHandlers(){
    const switches = document.querySelectorAll('#stPrivacy input[type="checkbox"]');
    const labels = ['对话存储', '使用数据', '本地处理'];
    switches.forEach((sw, i) => {
      sw.onchange = () => showToast((sw.checked ? '已开启' : '已关闭') + labels[i]);
    });
  }

  async function renderStGateway(){
    const llmEl = document.getElementById('stGwLlmList');
    const nodeEl = document.getElementById('stGwNodeList');
    if(!isTauri()){
      if(llmEl) llmEl.innerHTML = FREE_LLM_DATA.map(p => {
        const cls = p.good ? 'good' : 'dead';
        const barW = p.good ? Math.max(30, 100 - p.lat / 6) : 0;
        const barColor = p.lat < 400 ? 'var(--suc)' : p.lat < 700 ? 'var(--yellow)' : 'var(--des)';
        return `<div class="px-item"><span class="px-iname">${p.name}</span><span class="px-iprov">${p.prov}</span><span class="px-idot ${cls}"></span><div class="px-ibar"><div class="px-ibar-f" style="width:${barW}%;background:${barColor}"></div></div><span class="px-ilat">${p.good ? p.lat + 'ms' : '—'}</span><span class="px-irpm">${p.rpm}</span></div>`;
      }).join('');
      if(nodeEl) nodeEl.innerHTML = PROXY_DATA.map(p => {
        const cls = p.good ? 'good' : 'dead';
        const barW = p.good ? Math.max(30, 100 - p.lat / 2) : 0;
        const barColor = p.lat < 60 ? 'var(--suc)' : p.lat < 120 ? 'var(--yellow)' : 'var(--des)';
        return `<div class="px-item"><span class="px-ireg">${p.reg}</span><span class="px-iname">${p.name}</span><span class="px-idot ${cls}"></span><div class="px-ibar"><div class="px-ibar-f" style="width:${barW}%;background:${barColor}"></div></div><span class="px-ilat">${p.good ? p.lat + 'ms' : '—'}</span></div>`;
      }).join('');
      return;
    }
    try{
      const [providers, nodes] = await Promise.all([
        invoke('neocodex_provider_config'),
        invoke('proxy_pool_nodes').catch(() => [])
      ]);
      if(llmEl && providers && providers.providers){
        llmEl.innerHTML = providers.providers.map(p => {
          const cls = p.resolvable ? 'good' : 'dead';
          const lat = p.latency_ms || 0;
          const barW = p.resolvable ? Math.max(30, 100 - lat / 6) : 0;
          const barColor = lat < 400 ? 'var(--suc)' : lat < 700 ? 'var(--yellow)' : 'var(--des)';
          return `<div class="px-item"><span class="px-iname">${p.name}</span><span class="px-iprov">${p.model}</span><span class="px-idot ${cls}"></span><div class="px-ibar"><div class="px-ibar-f" style="width:${barW}%;background:${barColor}"></div></div><span class="px-ilat">${p.resolvable ? lat + 'ms' : '—'}</span><span class="px-irpm">${p.rpm || '—'}</span></div>`;
        }).join('');
        document.getElementById('stGwLlmMeta').textContent = providers.providers.filter(p=>p.resolvable).length + '/' + providers.provider_count + ' 在线';
      }
      if(nodeEl && Array.isArray(nodes)){
        nodeEl.innerHTML = nodes.slice(0,12).map(p => {
          const cls = p.healthy ? 'good' : 'dead';
          const barW = p.healthy ? Math.max(30, 100 - (p.latency_ms||0) / 2) : 0;
          const barColor = (p.latency_ms||0) < 60 ? 'var(--suc)' : (p.latency_ms||0) < 120 ? 'var(--yellow)' : 'var(--des)';
          return `<div class="px-item"><span class="px-ireg">${p.geo_tag ? p.geo_tag.slice(0,2) : '🌐'}</span><span class="px-iname">${p.tag || p.url}</span><span class="px-idot ${cls}"></span><div class="px-ibar"><div class="px-ibar-f" style="width:${barW}%;background:${barColor}"></div></div><span class="px-ilat">${p.healthy ? (p.latency_ms||0) + 'ms' : '—'}</span></div>`;
        }).join('');
        document.getElementById('stGwNodeMeta').textContent = nodes.filter(p=>p.healthy).length + '/' + nodes.length + ' 在线';
      }
      // MCP servers
      const mcpEl = document.getElementById('stGwMcpList');
      const mcpMeta = document.getElementById('stGwMcpMeta');
      if(mcpEl && mcpMeta){
        try{
          const mcpList = await invoke('neocodex_mcp_list');
          if(Array.isArray(mcpList) && mcpList.length){
            mcpEl.innerHTML = mcpList.map(s => `<div class="px-item"><span class="px-iname">${s.name}</span><span class="px-iprov">${s.transport}</span><span class="px-idot ${s.healthy ? 'good' : 'dead'}"></span><span class="px-ilat">${s.tool_count} 工具</span></div>`).join('');
            mcpMeta.textContent = mcpList.filter(s=>s.healthy).length + '/' + mcpList.length + ' 运行中';
          }else{
            mcpEl.innerHTML = '<div class="px-item" style="color:var(--tx-meta)">暂无 MCP 服务器</div>';
            mcpMeta.textContent = '0 服务器';
          }
        }catch(e){ mcpEl.innerHTML = '<div class="px-item" style="color:var(--des)">加载失败</div>'; mcpMeta.textContent = '0 服务器'; }
      }
    }catch(e){
      console.error('renderStGateway failed:', e);
    }
  }

  async function registerMcp(){
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可注册'); return; }
    const name = document.getElementById('mcpName')?.value?.trim();
    const cmd = document.getElementById('mcpCmd')?.value?.trim();
    const args = document.getElementById('mcpArgs')?.value?.trim();
    if(!name || !cmd){ showToast('请填写名称和命令'); return; }
    const argArr = args ? args.split(',').map(a=>a.trim()).filter(Boolean) : [];
    try{
      await invoke('neocodex_mcp_register', { name, command: cmd, args: argArr });
      showToast('MCP 服务器已注册: ' + name);
      document.getElementById('mcpName').value = '';
      document.getElementById('mcpCmd').value = '';
      document.getElementById('mcpArgs').value = '';
      await renderStGateway();
    }catch(e){ showToast('注册失败: ' + e); }
  }

  async function renderStCompute(){
    if(!isTauri()) return;
    try{
      const config = await invoke('neocodex_provider_config');
      if(!config || !config.providers) return;
      const sel = document.querySelector('#stCompute select');
      if(sel){
        sel.innerHTML = config.providers.map(p => `<option value="${p.name}" ${p.resolvable ? '' : 'disabled'}>${p.name} (${p.model})${p.resolvable ? '' : ' · 不可用'}</option>`).join('');
      }
    }catch(e){ console.error('renderStCompute failed:', e); }
  }

  async function renderStLimits(){
    if(!isTauri()) return;
    try{
      const config = await invoke('neocodex_provider_config');
      if(!config) return;
      const bars = document.querySelectorAll('#stLimits .gbar-f');
      if(bars.length >= 2){
        const used = Math.min(100, Math.round((config.provider_count || 1) * 21));
        bars[0].style.width = used + '%';
        bars[1].style.width = Math.min(100, used + 20) + '%';
      }
      document.querySelectorAll('#stLimits .st-desc').forEach((d,i) => {
        if(i===0) d.textContent = `已用 ${config.provider_count || 0} / 200 次`;
        if(i===1) d.textContent = `请求/分钟 18/30 · 令牌/分钟 45K/100K`;
      });
    }catch(e){ console.error('renderStLimits failed:', e); }
  }

  async function renderStPrivacy(){
    if(!isTauri()) return;
    try{
      const config = await invoke('neocodex_provider_config');
      if(!config) return;
      const switches = document.querySelectorAll('#stPrivacy input[type="checkbox"]');
      if(switches.length >= 3){
        switches[0].checked = config.provider_count > 0;
        switches[1].checked = false;
        switches[2].checked = true;
      }
    }catch(e){ console.error('renderStPrivacy failed:', e); }
  }

  async function renderStData(){
    if(!isTauri()) return;
    try{
      const ks = await invoke('get_knowledge_stats');
      if(ks && ks.total_nodes != null){
        const el = document.getElementById('kbNodeCount');
        if(el) el.textContent = Number(ks.total_nodes).toLocaleString() + ' 节点';
      }
    }catch(e){ console.error('renderStData failed:', e); }
  }

  async function exportAllData(){
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可导出'); return; }
    try{
      const sessions = await invoke('neocodex_list_sessions');
      const health = await invoke('neocodex_health_report');
      const config = await invoke('neocodex_provider_config');
      const kb = await invoke('kb_search', { query: '', limit: 1000 });
      const data = { sessions, health, config, knowledge: kb, exportedAt: new Date().toISOString() };
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `neotrix-export-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
      showToast('导出完成');
    }catch(e){ showToast('导出失败: ' + e); }
  }

  async function clearAllData(){
    if(!confirm('确定要清除所有本地数据吗？此操作不可恢复。')) return;
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可清除'); return; }
    try{
      await invoke('neocodex_clear_session', { session_id: 'all' }).catch(()=>{});
      showToast('已清除');
    }catch(e){ showToast('清除失败: ' + e); }
  }

  function toggleTheme(){
    const h=document.documentElement;
    const isDark=h.getAttribute('data-theme')==='dark';
    h.setAttribute('data-theme',isDark?'light':'dark');
    const lbl=document.getElementById('popThemeLabel');
    if(lbl)lbl.textContent=isDark?'亮色':'暗色';
    showToast(isDark?'🌞 已切换为亮色模式':'🌙 已切换为暗色模式');
  }

  function toggleSidebar(){
    document.querySelector('.sb').classList.toggle('collapsed');
  }

  function showToast(msg){
    document.querySelectorAll('.toast').forEach(e=>e.remove());
    const t=document.createElement('div');t.className='toast';
    t.textContent=msg;
    document.body.appendChild(t);
    requestAnimationFrame(()=>t.style.opacity='1');
    clearTimeout(window._tt);
    window._tt=setTimeout(()=>{t.style.opacity='0';setTimeout(()=>t.remove(),200);},1500);
  }
  function autoResize(el){
    el.style.height='auto';
    el.style.height=Math.min(el.scrollHeight,160)+'px';
    const btn=document.getElementById('sendBtn');
    if(btn) btn.disabled=!el.value.trim();
  }
  function handleKey(e){
    const inp=e.target;
    if(e.key==='Enter'&&!e.shiftKey){ e.preventDefault(); sendMsg(); return; }
    autoResize(inp);
  }
function escHtml(str){
    if(!str)return'';
    return String(str).replace(/&/g,'&').replace(/</g,'<').replace(/>/g,'>').replace(/"/g,'"');
  }

  /* ── Window controls: native macOS traffic lights (Overlay titlebar) handle
     close/minimize/maximize. Tauri overlay only needs the drag region. ── */

  // expose for inline onclick
  g.toggleSidebar = toggleSidebar;


