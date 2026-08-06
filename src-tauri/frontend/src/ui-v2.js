// AUTO-MIGRATED from design/previews/preview-ui-v2.html
import { invoke, listen, isTauri } from "./ipc";

/* ===== Global exposure for inline onclick handlers ===== */
const g = window;
g.autoResize = autoResize;
g.updateTokenCount = updateTokenCount;
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
g.filterSettings = filterSettings;
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
g.editMessage = editMessage;
g.saveMsgEdit = saveMsgEdit;
g.cancelMsgEdit = cancelMsgEdit;
g.deleteMessage = deleteMessage;
g.retryMessage = retryMessage;
g.copyUserContent = copyUserContent;
g.copyAssistantContent = copyAssistantContent;
g.attachAssistantCopy = attachAssistantCopy;
g.streamResume = streamResume;
g.regenPush = regenPush;
g.verNav = verNav;
g.verReset = verReset;
g.closeQM = closeQM;
g.QMUpdate = QMUpdate;
g.insertMention = insertMention;
g.runSlashCommand = runSlashCommand;
g.toggleCtxPop = toggleCtxPop;
g.renderContextMeter = renderContextMeter;
g.loadUsage = loadUsage;
g.pickAttachment = pickAttachment;
g.openRefPicker = openRefPicker;
g.closeRefPicker = closeRefPicker;
g.insertReference = insertReference;
g.addAttachChip = addAttachChip;
g.clearAttachments = clearAttachments;
g.renderThread = renderThread;
g.createSession = createSession;
g.refreshAgent = refreshAgent;
g.runAgent = runAgent;
g.runMsgCode = runMsgCode;
g.stopAgent = stopAgent;
g.stopStream = stopStream;
g.loadHypercube = loadHypercube;
g.loadSessions = loadSessions;
g.loadWsStatus = loadWsStatus;
g.loadRegistry = loadRegistry;
g.kbSearch = kbSearch;
g.sendSuggestion = sendSuggestion;
g.renderHeroSuggest = renderHeroSuggest;
g.cwFilter = cwFilter;
g.renderCowork = renderCowork;
g.togglePinSession = togglePinSession;
g.isPinnedSession = isPinnedSession;
g.renderStSecurity = renderStSecurity;
g.dayStartTs = dayStartTs;
g.groupSessionsByTime = groupSessionsByTime;
g.openSessionOps = openSessionOps;
g.renameSession = renameSession;
g.compactSession = compactSession;
g.archiveSession = archiveSession;
g.exportSession = exportSession;
g.deleteSession = deleteSession;
g.feedbackMessage = feedbackMessage;
g.searchSessions = searchSessions;
g.closeSessionOps = closeSessionOps;
g.checkForUpdate = checkForUpdate;
g.ntxConfirm = ntxConfirm;
g.openSessionFromSearch = openSessionFromSearch;
g.cycleMode = cycleMode;
g.openArchivedSessions = openArchivedSessions;
g.restoreArchived = restoreArchived;
g.deleteArchived = deleteArchived;
g.openCheckpointTimeline = openCheckpointTimeline;
g.restoreCheckpoint = restoreCheckpoint;
g.maskApiKey = maskApiKey;
g.renderStApiKey = renderStApiKey;
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
  const MODES = ['自动', '计划', '手动'];
  let currentMode = '手动';
  function cycleMode(){
    const idx = MODES.indexOf(currentMode);
    currentMode = MODES[(idx + 1) % MODES.length];
    const lbl = document.getElementById('ntxModeLabel');
    if(lbl) lbl.textContent = currentMode;
    const permMap = { '自动': 'auto', '计划': 'plan', '手动': 'manual' };
    if(isTauri()){
      const modeName = permMap[currentMode];
      // 语义对齐: 自动/手动 → Agent mode (shell 由 permission 门控, 手动经 UI 审批层);
      // 计划 → Plan mode (只读规划). 手动映射到 Shell 会与 policy_gate 的
      // manual-deny-shell 自相矛盾 (nt_io_neocodex.rs:830).
      invoke('neocodex_set_mode', { mode: modeName === 'plan' ? 'Plan' : 'Agent' }).catch(() => {});
      showToast('权限模式: ' + currentMode);
    }else{
      showToast('权限模式: ' + currentMode + '（浏览器演示）');
    }
  }
  function currentPermissionMode(){
    return { '自动': 'auto', '计划': 'plan', '手动': 'manual' }[currentMode] || 'auto';
  }
  function genParamsFromSettings(){
    const s = loadSettings();
    let temperature = Number(s['compute.temperature']);
    let maxTokens = Number(s['compute.maxTokens']);
    return {
      temperature: Number.isFinite(temperature) && temperature >= 0 && temperature <= 2 ? temperature : null,
      max_tokens: Number.isInteger(maxTokens) && maxTokens >= 256 ? maxTokens : null,
    };
  }


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
  let currentSessionId = null;

  const SETTINGS_KEY = 'neotrix.settings';
  function loadSettings(){
    try{
      const raw = localStorage.getItem(SETTINGS_KEY);
      return raw ? JSON.parse(raw) : {};
    }catch(_e){ return {}; }
  }
  function saveSetting(key, value){
    const s = loadSettings();
    s[key] = value;
    try{ localStorage.setItem(SETTINGS_KEY, JSON.stringify(s)); }catch(_e){}
    return s;
  }

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
      updateTokenCount();
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
      currentSessionId = id;
      const msgs = await invoke('neocodex_get_session_messages', { session_id: id });
      if(!Array.isArray(msgs)) return;
      switchView(document.querySelector('.segb[data-view="chat"]'), 'chat');
      renderThread(msgs);
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
    renderSidebar(view);
    syncWsTitle(view);
    if(view==='chat'){ renderHeroSuggest(); }
    if(view==='cowork') { renderCowork(); }
  }

  function syncWsTitle(view){
    const meta = {
      chat: { txt:'对话', ic:'<svg viewBox="0 0 14 14"><path d="M3 4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v4a1 1 0 0 1-1 1H7L5 10.5V9H4a1 1 0 0 1-1-1V4z" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linejoin="round"/></svg>' },
      cowork: { txt:'团队', ic:'<svg viewBox="0 0 14 14"><circle cx="5" cy="5" r="2" stroke="currentColor" stroke-width="1.1" fill="none"/><path d="M2 11.5a3 3 0 0 1 6 0" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round"/><circle cx="10" cy="5.5" r="1.6" stroke="currentColor" stroke-width="1.1" fill="none"/><path d="M8.5 11.5a3 3 0 0 1 3.5-2.9" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round"/></svg>' },
    };
    const ic = document.getElementById('wsTitleIc');
    const txt = document.getElementById('wsTitleTxt');
    if(ic) ic.innerHTML = meta[view] ? meta[view].ic : '';
    if(txt) txt.textContent = meta[view] ? meta[view].txt : '';
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
    ap._currentLoad = node.load || '';
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
      let html = escHtml(text);
      kw.forEach(k=>{ html = html.replace(new RegExp('\\b'+k+'\\b','g'),'<span class="kw">'+k+'</span>'); });
      html = html.replace(/\b[A-Z]\w+(?=\s*(?:[({<]|::))/g,m=>'<span class="fn">'+m+'</span>');
      html = html.replace(/\/\/[^\n]*/g,m=>'<span class="cm">'+m+'</span>');
      el.innerHTML = '<code style="font-family:var(--fm);font-size:10px;line-height:1.6;color:var(--tx2);">'+html+'</code>';
      return;
    }

    /* Smart markdown → HTML.
       Reuse the XSS-safe renderRichText pipeline (stash + escape, scheme-allowlisted links,
       escaped code fences) instead of the legacy hand-rolled regex renderer that failed to
       escape quotes and allowed javascript: hrefs. Single renderer = single security surface. */
    el.innerHTML = renderRichText(text);
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

  async function refreshPreview(e){
    e.stopPropagation();
    const ap = document.getElementById('filePreview');
    const path = ap ? (ap._currentLoad || ap._currentName) : null;
    if(!path){
      showToast('当前无打开文件');
      return;
    }
    if(!isTauri()){
      showToast('已刷新（浏览器模式）');
      return;
    }
    try{
      const content = await invoke('read_file', { path });
      ap._currentContent = content;
      renderPreviewContent(ap._currentName, content);
      showToast('已刷新文件');
    }catch(err){ showToast('刷新失败: ' + err); }
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
    /* Restore persisted theme before first paint */
    try{
      const saved = localStorage.getItem('neotrix.theme');
      if(saved === 'light' || saved === 'dark') document.documentElement.setAttribute('data-theme', saved);
    }catch(_e){}
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
  // Live reference so vitest/e2e can inject data then call renderCowork() directly
  Object.defineProperty(g, 'CW_DATA', { get: () => CW_DATA, set: (v) => { CW_DATA = v; }, configurable: true });
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

  /** 今天 0 点（本地时区）的 Unix 秒；offsetDays 可回退到昨天/前 N 天 */
  function dayStartTs(offsetDays = 0){
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    if(offsetDays) d.setDate(d.getDate() + offsetDays);
    return Math.floor(d.getTime() / 1000);
  }

  /* 会话时间分组 (对标 ChatGPT/Claude 侧边栏 Today/Yesterday/7d/Earlier)
     返回非空分组：[{ label:'今天', sessions:[...] }, { label:'昨天', ... }, { label:'7 天内', ... }, { label:'更早', ... }]
     无 updated_at 的会话归入「更早」；组内按 updated_at 降序。 */
  function groupSessionsByTime(sessions){
    const DAY = 86400;
    const todayStart = dayStartTs(0);
    const yesterdayStart = todayStart - DAY;
    const weekStart = todayStart - 7 * DAY;
    const bucketOf = (s) => {
      const ts = typeof (s && s.updated_at) === 'number' && s.updated_at > 0 ? s.updated_at : 0;
      if(!ts || ts < weekStart) return '更早';
      if(ts >= todayStart) return '今天';
      if(ts >= yesterdayStart) return '昨天';
      return '7 天内';
    };
    const order = ['今天', '昨天', '7 天内', '更早'];
    const buckets = order.map(label => ({ label, sessions: [] }));
    (sessions || []).forEach(s => buckets[order.indexOf(bucketOf(s))].sessions.push(s));
    buckets.forEach(b => b.sessions.sort((a, b) => (b.updated_at || 0) - (a.updated_at || 0)));
    return buckets.filter(b => b.sessions.length > 0);
  }

  /* 会话置顶（ChatGPT Pinned 对标）：localStorage 持久化，置顶会话优先展示在「置顶」组 */
  function getPinnedSessions(){
    try{ return JSON.parse(localStorage.getItem('neotrix.pinned') || '[]'); }
    catch(_e){ return []; }
  }
  function isPinnedSession(id){
    return getPinnedSessions().includes(String(id));
  }
  function togglePinSession(id, ev){
    if(ev){ ev.stopPropagation(); }
    let pins = getPinnedSessions();
    const key = String(id);
    if(pins.includes(key)) pins = pins.filter(x => x !== key);
    else pins.unshift(key);
    try{ localStorage.setItem('neotrix.pinned', JSON.stringify(pins)); }catch(_e){}
    renderCowork();
    showToast(pins.includes(key) ? '已置顶会话' : '已取消置顶');
  }

  function renderCowork(){
    const sl = document.getElementById('cwSessionList');
    if(!sl) return;
    const filtered = CW_DATA.filter(s => {
      if(cwStatusFilter === 'active') return s.status !== '已完成';
      if(cwStatusFilter === 'done') return s.status === '已完成';
      return true;
    });
    // Pinned bucket first, then time groups (ChatGPT parity)
    const pinned = filtered.filter(s => isPinnedSession(s.id));
    const rest = filtered.filter(s => !isPinnedSession(s.id));
    const groups = groupSessionsByTime(rest);
    let firstShown = true;
    const renderItems = (grp) => {
      const items = grp.sessions.map(s => {
        const pct = s.tasks > 0 ? Math.round(s.done/s.tasks*100) : 0;
        const stCls = s.status === '已完成' ? 's-done' : (s.status === '已暂停' ? 's-paused' : 's-run');
        const stTxt = s.status || '就绪';
        const msg = s.message_count ? ` · ${s.message_count} 消息` : '';
        const active = firstShown ? ' active' : '';
        firstShown = false;
        const pinBtn = `<button class="cw-pin-btn" title="置顶/取消置顶" onclick="togglePinSession('${escHtml(String(s.id))}', event)">📌</button>`;
        return `<div class="cw-sitem${active}" data-idx="${CW_DATA.indexOf(s)}" onclick="selectCwSession(${CW_DATA.indexOf(s)}, true)">
          <div class="s">
            <span class="st-dot ${stCls}"></span>
            <span class="st-t">${escHtml(s.name)}</span>
            ${pinBtn}
          </div>
          <span class="s">${s.done}/${s.tasks} 任务 · ${pct}%${msg}</span>
          <span class="s">${escHtml(stTxt)}</span>
        </div>`;
      }).join('');
      return `<div class="cw-group-h">${grp.label}</div>${items}`;
    };
    // Compose: pinned bucket first, then time buckets
    const pinnedHtml = pinned.length
      ? `<div class="cw-group-h">📌 置顶</div>` + pinned.map(s => {
          const pct = s.tasks > 0 ? Math.round(s.done/s.tasks*100) : 0;
          const stCls = s.status === '已完成' ? 's-done' : (s.status === '已暂停' ? 's-paused' : 's-run');
          const stTxt = s.status || '就绪';
          const msg = s.message_count ? ` · ${s.message_count} 消息` : '';
          const active = firstShown ? ' active' : '';
          firstShown = false;
          const pinBtn = `<button class="cw-pin-btn" title="取消置顶" onclick="togglePinSession('${escHtml(String(s.id))}', event)">📌</button>`;
          return `<div class="cw-sitem${active}" data-idx="${CW_DATA.indexOf(s)}" onclick="selectCwSession(${CW_DATA.indexOf(s)}, true)">
            <div class="s"><span class="st-dot ${stCls}"></span><span class="st-t">${escHtml(s.name)}</span>${pinBtn}</div>
            <span class="s">${s.done}/${s.tasks} 任务 · ${pct}%${msg}</span>
            <span class="s">${escHtml(stTxt)}</span>
          </div>`;
        }).join('')
      : '';
    sl.innerHTML = pinnedHtml + groups.map(renderItems).join('');
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
            const name = (typeof d === 'string') ? d : (d.name || ('交付物 #' + (i+1)));
            const kind = (typeof d === 'string') ? '' : (d.kind || '');
            items.push({ name, done: i < (c.files_created||0), fail: false, meta: kind });
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
    /* Gate: confirm the exact command before executing shell (P1-1) — LLM output is untrusted.
       Destructive/network-piping commands are flagged danger; plain commands still get a confirm. */
    const danger = /(^|\s)(rm|mv|dd|mkfs|shutdown|reboot|sudo|curl|wget|git\s+push|git\s+reset\s+--hard)\b/.test(cmd) || /\|\s*(sh|bash|sudo)\s*$/.test(cmd) || /;\s*(sh|bash|sudo)\s/.test(cmd);
    const ok = await ntxConfirm(cmd, {
      title: danger ? '执行危险命令？' : '确认执行命令',
      confirmText: danger ? '仍然执行' : '运行',
      danger,
    });
    if(!ok){ showToast('已取消'); return; }
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
  /* 按语言选取关键字集 — 多语言高亮 (Rust/JS/TS/JSON/Python/Shell) */
  const LANG_KW = {
    rust:  new Set(['fn','pub','let','mut','Result','Ok','f64','std','use','match','for','in','if','else','return','struct','impl','enum','trait','async','await','mod','crate','self','Self','where','while','loop','break','continue','const','static','type','unsafe','ref','move','Some','None','Vec','String','Box','dyn','super','as','true','false','i32','i64','usize','u64','u32']),
    js:    new Set(['function','const','let','var','if','else','return','for','while','class','import','export','from','async','await','try','catch','throw','new','this','typeof','instanceof','null','undefined','true','false','switch','case','break','continue','default','extends','super','yield','delete','in','of','void','do','static','get','set','Map','Set','Promise','Array','Object','console']),
    ts:    new Set(['function','const','let','var','if','else','return','for','while','class','import','export','from','async','await','try','catch','throw','new','this','typeof','instanceof','null','undefined','true','false','switch','case','break','continue','default','extends','super','yield','delete','in','of','void','do','static','get','set','interface','type','enum','implements','readonly','public','private','protected','abstract','namespace','declare','satisfies','keyof','infer','never','unknown','any','string','number','boolean','Map','Set','Promise','Array','Object','console']),
    json:  new Set(['true','false','null']),
    python:new Set(['def','class','import','from','if','elif','else','for','while','return','try','except','finally','with','as','lambda','pass','break','continue','global','nonlocal','yield','async','await','raise','assert','del','not','and','or','in','is','None','True','False','self','print','len','range','dict','list','set','tuple','str','int','float','bool','__init__','super']),
    shell: new Set(['if','then','else','fi','for','while','do','done','case','esac','function','export','local','return','echo','cd','ls','mkdir','rm','cp','mv','cat','grep','sed','awk','curl','wget','sudo','exit','read','set','shift','source','printf','test','true','false','&&','||']),
  };
  const DEFAULT_KW = LANG_KW.rust;
  function langKwSet(lang){
    const l = String(lang || '').toLowerCase();
    if(l.includes('rust') || l.includes('rs')) return LANG_KW.rust;
    if(l.includes('typescript') || l.includes('tsx') || l.includes('ts')) return LANG_KW.ts;
    if(l.includes('javascript') || l.includes('jsx') || l.includes('js')) return LANG_KW.js;
    if(l.includes('json')) return LANG_KW.json;
    if(l.includes('python') || l.includes('py')) return LANG_KW.python;
    if(l.includes('bash') || l.includes('sh') || l.includes('shell') || l.includes('zsh')) return LANG_KW.shell;
    return DEFAULT_KW;
  }
  function highlightCode(code, lang){
    const kwSet = langKwSet(lang);
    const esc = s => s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
    const re = /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\/\/[^\n]*|#[^\n]*|\/\*[\s\S]*?\*\/|@\w+|[A-Z]\w+|\w+|[^a-zA-Z0-9_'"\s]+|\s+)/g;
    let out = '', m;
    while((m = re.exec(code)) !== null){
      const tok = m[0];
      if(tok.startsWith('"') || tok.startsWith("'")) out += '<span class="hl">' + esc(tok) + '</span>';
      else if(tok.startsWith('//') || tok.startsWith('#') || tok.startsWith('/*')) out += '<span class="cm">' + esc(tok) + '</span>';
      else if(tok.startsWith('@')) out += '<span class="fn">' + tok + '</span>';
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

  /* Lightweight XSS-safe Markdown: tokenize inline into safe HTML placeholders,
     escape the remaining raw text, then restore. Raw \n survives for <br> post-pass.
     Inline: `code`, **bold**, *italic*, ~~strike~~, [text](url http/https/mailto).
     Block: # headings, > quotes, -/1. lists, - [ ] tasks, pipe tables, --- hr, code fences. */
  function mdInline(s){
    const esc = ss => escHtml(ss);
    const ph = [];
    const stash = r => { ph.push(r); return '\u0000' + (ph.length - 1) + '\u0000'; };
    let out = String(s);
    out = out.replace(/`([^`\n]+)`/g, (_, c) => stash('<code>' + esc(c) + '</code>'));
    out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (m, t, u) => {
      const href = String(u).replace(/"/g, '&quot;');
      return /^(https?:|mailto:)/i.test(href) && !/javascript:/i.test(href)
        ? stash('<a href="' + href + '" target="_blank" rel="noreferrer">' + esc(t) + '</a>')
        : m;
    });
    out = out.replace(/\*\*([^*\n]+)\*\*/g, (_, b) => stash('<strong>' + esc(b) + '</strong>'));
    out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, (_, p, b) => p + stash('<em>' + esc(b) + '</em>'));
    out = out.replace(/~~([^~\n]+)~~/g, (_, d) => stash('<del>' + esc(d) + '</del>'));
    out = esc(out);
    return out.replace(/\u0000(\d+)\u0000/g, (_, n) => ph[+n]);
  }

  function mdTableBlock(rawLines, start){
    const split = l => l.replace(/^\s*\|/, '').replace(/\|\s*$/, '').split('|').map(c => c.trim());
    const header = split(rawLines[start]);
    let i = start + 2;
    const rows = [];
    while(i < rawLines.length && rawLines[i].trim() && rawLines[i].includes('|')){
      rows.push(split(rawLines[i]));
      i++;
    }
    let out = '<table><thead><tr>';
    header.forEach(h => { out += '<th>' + mdInline(h || '&nbsp;') + '</th>'; });
    out += '</tr></thead><tbody>';
    rows.forEach(r => {
      out += '<tr>';
      header.forEach((_, ci) => { out += '<td>' + mdInline(r[ci] || '') + '</td>'; });
      out += '</tr>';
    });
    out += '</tbody></table>';
    return { html: out, next: i };
  }

  function renderRichText(text){
    const rawLines = String(text || '').split('\n');
    let out = '';
    let i = 0;
    let para = [];
    const flush = () => {
      if(!para.length) return;
      out += '<p>' + mdInline(para.join('\n')).replace(/\n/g, '<br>') + '</p>';
      para = [];
    };
    while(i < rawLines.length){
      const line = rawLines[i];
      const fence = line.match(/^\s*```([\w+-]*)\s*$/);
      if(fence){
        flush();
        const lang = fence[1];
        i++;
        const code = [];
        while(i < rawLines.length && !/^\s*```\s*$/.test(rawLines[i])){ code.push(rawLines[i]); i++; }
        i++;
        out += `<div class="msg-code"><div class="msg-code-h"><span class="msg-code-lang">${escHtml(lang || 'code')}</span><span class="msg-code-actions"><button class="msg-code-cp" onclick="runMsgCode(this)">运行</button><button class="msg-code-cp" onclick="copyMsgCode(this)">复制</button></span></div><pre class="msg-code-b">${highlightCode(code.join('\n'), lang)}</pre></div>`;
        continue;
      }
      if(!line.trim()){ flush(); i++; continue; }
      const hd = line.match(/^(#{1,6})\s+(.+)$/);
      if(hd){ flush(); const lvl = hd[1].length; out += `<h${lvl}>${mdInline(hd[2])}</h${lvl}>`; i++; continue; }
      if(/^>\s?/.test(line)){
        flush();
        const q = [];
        while(i < rawLines.length && /^>\s?/.test(rawLines[i])){ q.push(rawLines[i].replace(/^>\s?/, '')); i++; }
        out += '<blockquote>' + mdInline(q.join('\n')).replace(/\n/g, '<br>') + '</blockquote>';
        continue;
      }
      if(/^---+$|^\*\*\*+$|^___+$/.test(line.trim())){ flush(); out += '<hr>'; i++; continue; }
      if(line.includes('|') && i + 1 < rawLines.length && /^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)+\|?\s*$/.test(rawLines[i + 1])){
        flush();
        const tb = mdTableBlock(rawLines, i);
        out += tb.html;
        i = tb.next;
        continue;
      }
      if(/^\s*[-*+]\s+/.test(line)){
        flush();
        const items = [];
        while(i < rawLines.length && /^\s*[-*+]\s+/.test(rawLines[i])){
          let content = rawLines[i].replace(/^\s*[-*+]\s+/, '');
          const task = content.match(/^\[([ xX])\]\s+(.+)$/);
          if(task){
            const done = /^[xX]$/.test(task[1]);
            items.push(`<li class="md-task${done ? ' md-task-done' : ''}"><span class="md-cb">${done ? '✓' : '○'}</span>${mdInline(task[2])}</li>`);
          } else {
            items.push('<li>' + mdInline(content) + '</li>');
          }
          i++;
        }
        out += '<ul>' + items.join('') + '</ul>';
        continue;
      }
      if(/^\s*\d+[.)]\s+/.test(line)){
        flush();
        const items = [];
        while(i < rawLines.length && /^\s*\d+[.)]\s+/.test(rawLines[i])){
          items.push('<li>' + mdInline(rawLines[i].replace(/^\s*\d+[.)]\s+/, '')) + '</li>');
          i++;
        }
        out += '<ol>' + items.join('') + '</ol>';
        continue;
      }
      para.push(line);
      i++;
    }
    flush();
    return out || '<span></span>';
  }

  function switchProvider(val){
    showToast('切换提供者: ' + val);
  }

  /* ===== IPC-backed streaming send ===== */
  const streamSubs = new Map();
  let streamFollow = true;
  let streamActive = window._ntxStreamActive = false;
  let streamBuf = '';
  const lastUserMsgs = [];
  let recallIdx = -1;
  function streamScrollEl(){
    return document.getElementById('chatScroll');
  }
  function scrollChatToBottom(force){
    const cs = streamScrollEl();
    if(!cs) return;
    if(force || streamFollow) cs.scrollTop = cs.scrollHeight;
  }
  function renderStreamProgress(el){
    const parts = streamBuf.split(/```/);
    const open = parts.length % 2 === 0;
    if(open){
      const closed = parts.slice(0, -1).join('```');
      const tail = parts[parts.length - 1];
      el.innerHTML = renderRichText(closed) + '<pre class="msg-code-b msg-code-stream">' + escHtml(tail) + '</pre>';
    }else{
      el.innerHTML = renderRichText(streamBuf);
    }
  }
  function setStreaming(on){
    streamActive = window._ntxStreamActive = on;
    const send = document.getElementById('sendBtn');
    const stop = document.getElementById('stopBtn');
    if(send) send.disabled = on;
    if(stop) stop.style.display = on ? 'inline-flex' : 'none';
    streamFollow = on;
    const pill = document.getElementById('scrollJump');
    if(pill) pill.classList.remove('show');
  }
  function stopStream(){
    if(!isTauri()){ setStreaming(false); return; }
    invoke('neocodex_stop_stream').catch(()=>{});
    setStreaming(false);
    clearThink();
    const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
    if(el){
      el.classList.remove('streaming');
      attachUsageFooter(el.closest('.msg'));
      const msgEl = el.closest('.msg');
      if(msgEl && !msgEl.querySelector('.stream-resume')){
        const card = document.createElement('div');
        card.className = 'stream-resume';
        card.innerHTML = `<button class="sr-btn" onclick="streamResume()">继续生成</button><span class="sr-note">已保留部分输出</span>`;
        msgEl.appendChild(card);
      }
    }
    showToast('已停止生成');
  }

  async function streamResume(){
    const card = document.querySelector('#chatScroll .msg.l .stream-resume');
    if(card) card.remove();
    if(!isTauri() || !currentSessionId) return;
    // Regenerate from the last visible user message: drop the partial reply and
    // re-send the latest user prompt for a fresh completion.
    const msgs = await invoke('neocodex_get_session_messages', { session_id: currentSessionId }).catch(() => null);
    if(Array.isArray(msgs)){
      let lastUserIdx = -1;
      msgs.forEach((m,i) => { if(m.role === 'user') lastUserIdx = i; });
      if(lastUserIdx < 0){ showToast('没有可继续的用户消息'); return; }
      await invoke('neocodex_regenerate', { session_id: currentSessionId, index: lastUserIdx }).catch(e => { showToast('重试失败: ' + e); return null; });
      const refresh = await invoke('neocodex_get_session_messages', { session_id: currentSessionId }).catch(() => null);
      if(Array.isArray(refresh)) renderThread(refresh);
      const inp = document.getElementById('chatInput');
      inp.value = msgs[lastUserIdx].content || '';
      sendMsg();
    }
  }
  let thinkStart = 0;
  let thinkTimer = null;
  function clearThink(){
    if(thinkTimer){ clearInterval(thinkTimer); thinkTimer = null; }
    const th = document.querySelector('#chatScroll .msg.l .mb .think');
    if(th) th.remove();
  }
  function startThink(el){
    clearThink();
    thinkStart = Date.now();
    const span = document.createElement('span');
    span.className = 'think';
    const tick = () => { span.textContent = '思考中… ' + Math.floor((Date.now() - thinkStart) / 1000) + 's'; };
    tick();
    (el.querySelector('.mb') || el).appendChild(span);
    thinkTimer = setInterval(tick, 1000);
  }
  function ensureStreamListeners(){
    if(!isTauri() || streamSubs.size) return;
    const attach=(ev,fn)=>{
      try{
        listen(ev, fn)
          .then(un=>{ streamSubs.set(ev,un); })
          .catch(()=>{ if(ev==='neocodex_stream_token') showToast('流式事件订阅失败，消息可能无法实时显示'); });
      }catch(_e){}
    };
    attach('neocodex_stream_token', p => {
      clearThink();
      const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
      if(el){ streamBuf += String(p); renderStreamProgress(el); scrollChatToBottom(false); }
    });
    attach('neocodex_stream_end', p => {
      clearThink();
      const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
      if(el){ el.classList.remove('streaming'); streamBuf = String(p); el.innerHTML = renderRichText(String(p)); attachUsageFooter(el.closest('.msg')); attachAssistantCopy(el.closest('.msg')); }
      setStreaming(false);
      scrollChatToBottom(true);
    });
    attach('neocodex_stream_done', async () => {
      clearThink();
      setStreaming(false);
      const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
      if(el){ el.classList.remove('streaming'); attachUsageFooter(el.closest('.msg')); attachAssistantCopy(el.closest('.msg')); }
      streamBuf = '';
      await loadUsage();
      const foot=document.querySelector('#chatScroll .msg.l:last-child .msg-usage');
      if(foot) foot.textContent = '上下文 ' + Math.round(lastContextUsage * 100) + '%';
      scrollChatToBottom(true);
    });
    attach('neocodex_stream_start', p => {
      const el=document.querySelector('#chatScroll .msg.l .mb.streaming');
      if(el) startThink(el);
    });
  }

  /* 原生菜单事件接线 (lib.rs setup_menu / setup_tray emit):
     macOS 菜单 accelerator 优先于 webview keydown, 因此 Cmd+N/Cmd+,/Cmd+K/
     Cmd+Shift+U 及托盘项必须经此监听转发到前端处理, 否则快捷键全部失效. */
  function wireMenuEvents(){
    if(!isTauri() || window._ntxMenuBound) return;
    window._ntxMenuBound = true;
    const attach=(ev,fn)=>{
      try{ listen(ev, fn).catch(()=>{}); }catch(_e){}
    };
    attach('neotrix:new-session', () => createSession());
    attach('open-settings', () => openSettingsModal());
    attach('neocodex-open-palette', () => openPalette());
    attach('neocodex-check-updates', () => checkForUpdate());
    attach('sync-trigger', () => { showToast('同步触发'); refreshSessionList(); });
    attach('proxy-mode-change', (mode) => { showToast('代理模式: ' + String(mode || '')); });
    attach('open-proxy-status', () => { showToast('代理状态面板'); });
  }

  function wireStreamScroll(){
    const cs = document.getElementById('chatScroll');
    if(!cs || cs.dataset.scrollWired) return;
    cs.dataset.scrollWired = '1';
    cs.addEventListener('scroll', () => {
      const nearBottom = cs.scrollHeight - cs.scrollTop - cs.clientHeight < 48;
      const pill = document.getElementById('scrollJump');
      if(nearBottom){
        streamFollow = true;
        if(pill) pill.classList.remove('show');
      }else if(document.querySelector('#chatScroll .msg.l .mb.streaming')){
        streamFollow = false;
        if(pill) pill.classList.add('show');
      }
    });
  }

  function jumpToLatest(){
    streamFollow = true;
    const pill = document.getElementById('scrollJump');
    if(pill) pill.classList.remove('show');
    scrollChatToBottom(true);
    const inp = document.getElementById('chatInput');
    if(inp) inp.focus();
  }

  function sendMsg(){
    ensureStreamListeners();
    const inp=document.getElementById('chatInput');
    const txt=inp.value.trim();if(!txt)return;
    if(lastUserMsgs[lastUserMsgs.length-1] !== txt) lastUserMsgs.push(txt);
    recallIdx = -1;
    clearDraft();
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
    u.innerHTML=`<div class="msg-act"><button class="ma-btn" data-op="copy" title="复制内容">复制</button></div><div class="mb">${escHtml(txt)}</div>`;
    const uCopy = u.querySelector('.ma-btn[data-op="copy"]');
    uCopy.onclick = () => copyUserContent(u);
    s.appendChild(u);
    inp.value='';inp.style.height='auto';
    updateTokenCount();
    setStreaming(true);
    streamBuf = '';
    s.scrollTop=s.scrollHeight;
    openRbSidebar();
    const a=document.createElement('div');a.className='msg l';
    a.innerHTML=`<div class="msg-h"><span class="name">NeoTrix</span><span class="time">${t}</span></div><div class="mb streaming"><span class="typing"><span></span><span></span><span></span></span></div>`;
    s.appendChild(a);s.scrollTop=s.scrollHeight;

    if(!isTauri()){
      /* Browser fallback: simulated reply so the UI stays demo-able.
         标记演示模式, 避免用户误认为真实模型响应 (D8). */
      setTimeout(()=>{
        const rs=[
          '好的，我来逐步分析这个问题。',
          '收到，基于当前上下文，这是我的分析结果。',
          '没问题！我已经分析了你的请求，以下是我的思考。'
        ];
        const mb=a.querySelector('.mb');
        mb.classList.remove('streaming');
        const demo = '<span class="demo-badge">浏览器演示模式</span>\n\n' + rs[Math.floor(Math.random()*rs.length)] + '\n\n```rust\nfn main() {\n    println!("Hello, NeoTrix!");\n    let engine = ReasoningEngine::new();\n    engine.run();\n}\n```';
        mb.innerHTML = renderRichText(demo);
        setStreaming(false);
        attachUsageFooter(a);
        attachAssistantCopy(a);
      },600+Math.random()*400);
      return;
    }

    const g = genParamsFromSettings();
    const att = attachPayloads();
    // Auto-title (ChatGPT parity): first message of a fresh session names the session.
    if(isTauri() && currentSessionId){
      const prev = CW_DATA.find(x => x.id === currentSessionId);
      const msgCount = (prev && prev.message_count) || 0;
      if(msgCount === 0){
        const title = txt.replace(/\s+/g, ' ').trim().slice(0, 28);
        invoke('neocodex_rename_session', { session_id: currentSessionId, name: title }).catch(()=>{});
        if(prev) prev.name = title;
        renderCowork();
      }
    }
    const streamOpts = {
      content: txt,
      attachments: att.length ? att : null,
      regenerate: false,
      permission_mode: currentPermissionMode(),
      temperature: g.temperature,
      max_tokens: g.max_tokens,
    };
    invoke('neocodex_send_message_stream', streamOpts).catch(err=>{
      const mb=a.querySelector('.mb');
      setStreaming(false);
      if(mb){
        mb.classList.remove('streaming');
        mb.textContent='';
        // Error + retry (ChatGPT/Claude parity): keep the failed bubble, offer one-tap retry.
        const errBox = document.createElement('div');
        errBox.className = 'msg-ipc-err';
        const safeErr = String(err).slice(0, 300).replace(/sk-[A-Za-z0-9_-]{8,}/gi, 'sk-•••');
        errBox.innerHTML = '<div class="msg-ipc-err-txt">⚠️ ' + escHtml(safeErr) + '</div>' +
          '<button class="msg-ipc-retry" data-op="retry">重试</button>';
        errBox.querySelector('[data-op="retry"]').onclick = () => {
          mb.classList.add('streaming');
          mb.innerHTML = '<span class="typing"><span></span><span></span><span></span></span>';
          errBox.remove();
          invoke('neocodex_send_message_stream', streamOpts)
            .then(() => {})
            .catch(err2 => {
              mb.classList.remove('streaming');
              const safe2 = String(err2).slice(0, 300).replace(/sk-[A-Za-z0-9_-]{8,}/gi, 'sk-•••');
              mb.innerHTML = '<div class="msg-ipc-err-txt">⚠️ ' + escHtml(safe2) + '</div>';
            });
        };
        mb.appendChild(errBox);
      }
    });
    if(att.length) clearAttachments();
  }
  function clearAttachments(){
    attachList = [];
    const area = document.getElementById('ntxAttachArea');
    if(area) area.innerHTML = '';
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

  /* ===== Missing Functions ===== */
  function clearChat(){
    actions.newChat();
    showToast('已清空对话');
  }

  /* ===== Keyboard Shortcuts ===== */
  if(!window._ntxKeyBound){
    window._ntxKeyBound = true;
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
      openPalette();
    }
    if(e.key === 'Escape'){
      const openOv = document.querySelectorAll('.overlay-panel.open');
      if(openOv.length){
        openOv.forEach(p => p.classList.remove('open'));
        closePopover();
        updateTrafficVisibility();
        return;
      }
      // ChatGPT/Claude parity: Esc 中断正在生成的回复
      if(window._ntxStreamActive){
        e.preventDefault();
        stopStream();
        return;
      }
      closePopover();
      updateTrafficVisibility();
    }
    if(e.key === '?' && !e.target.closest('textarea, input')){
      showToast('快捷键: ⌘1/⌘2 切换 · ⌘, 设置 · ⌘N 新建 · ⌘F 知识库 · ⌘W 关闭 · ⌘K 搜索 · Esc 关闭 · ? 帮助');
    }
  });
  }

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
  // Persisted appearance settings (localStorage) — apply before first paint of settings
  const settings = loadSettings();
  if(settings['appearance.fontSize']) document.documentElement.style.fontSize = settings['appearance.fontSize'] + 'px';
  if(settings['appearance.reduceTransparency']) document.documentElement.classList.add('reduce-trans');
  // Send button initial state (direct call — no synthetic event needed)
  const ci0 = document.getElementById('chatInput');
  const sb0 = document.getElementById('sendBtn');
  if(ci0) autoResize(ci0);
  if(sb0) sb0.disabled = !(ci0 && ci0.value.trim());
  // Live backend: hydrate real data when inside Tauri
  wireBackend();
  wireStreamScroll();
  fusionInit();
  wireMenuEvents();



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

  function renderThread(msgs, sessionId){
    if(sessionId) currentSessionId = sessionId;
    const cs = document.getElementById('chatScroll');
    if(!cs) return;
    document.getElementById('heroSection').style.display = 'none';
    cs.style.display = 'flex';
    cs.innerHTML = '';
    lastUserMsgs.length = 0;
    recallIdx = -1;
    let visIdx = 0; // visible index: only user/assistant count (backend contract)
    (msgs || []).forEach((m, i) => {
      const t = m.timestamp ? new Date(m.timestamp * 1000).toTimeString().slice(0,5) : '';
      if(m.role === 'user'){
        if(m.content && lastUserMsgs[lastUserMsgs.length-1] !== m.content) lastUserMsgs.push(m.content);
        const idx = visIdx++;
        const u = document.createElement('div'); u.className = 'msg r';
        u.innerHTML = `<div class="msg-act"><button class="ma-btn" data-op="copy" title="复制内容">复制</button><button class="ma-btn" data-op="edit" title="编辑消息">编辑</button><button class="ma-btn" data-op="delete" title="删除消息">删除</button></div><div class="mb">${escHtml(m.content)}</div>`;
        u.dataset.vid = String(idx);
        u.querySelector('.ma-btn[data-op="copy"]').onclick = () => copyUserContent(u);
        u.querySelector('.ma-btn[data-op="edit"]').onclick = () => editMessage(idx);
        u.querySelector('.ma-btn[data-op="delete"]').onclick = () => deleteMessage(idx);
        cs.appendChild(u);
      }else if(m.role === 'assistant' || m.role === 'agent'){
        const aiIdx = cs.querySelectorAll('.msg.l').length;
        const idx = visIdx++;
        const a = document.createElement('div'); a.className = 'msg l';
        a.innerHTML = `<div class="msg-h"><span class="name">NeoTrix</span><span class="time">${t}</span></div><div class="msg-act"><button class="ma-btn" data-op="copy" title="复制内容">复制</button><button class="ma-btn" data-op="retry" title="重新生成回复">重试</button><button class="ma-btn" data-op="like" title="有帮助">👍</button><button class="ma-btn" data-op="dislike" title="需改进">👎</button><button class="ma-btn" data-op="delete" title="删除消息">删除</button></div><div class="mb">${renderRichText(m.content)}</div>`;
        a.dataset.vid = String(idx);
        a.querySelector('.ma-btn[data-op="copy"]').onclick = () => copyMessageContent(aiIdx);
        a.querySelector('.ma-btn[data-op="retry"]').onclick = () => retryMessage(idx);
        a.querySelector('.ma-btn[data-op="delete"]').onclick = () => deleteMessage(idx);
        a.querySelector('.ma-btn[data-op="like"]').onclick = () => feedbackMessage(aiIdx, 'like');
        a.querySelector('.ma-btn[data-op="dislike"]').onclick = () => feedbackMessage(aiIdx, 'dislike');
        const fb = loadFeedback()[currentSessionId + ':' + aiIdx];
        if(fb) renderFeedbackState(aiIdx, fb);
        cs.appendChild(a);
      }else if(m.role === 'tool'){
        const tcard = document.createElement('div'); tcard.className = 'tool-card';
        const head = document.createElement('div'); head.className = 'tool-head';
        head.innerHTML = `<svg viewBox="0 0 14 14" class="tool-ic"><path d="M2.5 4.5h9M3.5 4.5l.8 7a1 1 0 001 1h3.4a1 1 0 001-1l.8-7" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round"/></svg><span class="tool-name">${escHtml((m.content||'').split('**')[1] || '工具调用')}</span><span class="tool-time">${t}</span><span class="tool-toggle">▾</span>`;
        const body = document.createElement('div'); body.className = 'tool-body';
        body.innerHTML = `<pre>${escHtml((m.content||'').replace(/^\*\*.*?\*\*/, '').slice(0, 600))}</pre>`;
        tcard.appendChild(head); tcard.appendChild(body);
        head.addEventListener('click', () => tcard.classList.toggle('open'));
        cs.appendChild(tcard);
      }else if(m.role === 'system' && m.content){
        const scard = document.createElement('div'); scard.className = 'sys-card';
        scard.innerHTML = `<span class="sys-dot"></span><span class="sys-txt">${renderRichText(String(m.content).slice(0, 200))}</span>`;
        cs.appendChild(scard);
      }
    });
    isChatMode = true;
    cs.scrollTop = cs.scrollHeight;
    restoreDraft();
  }

  async function reloadThread(){
    if(!isTauri() || !currentSessionId) return;
    const msgs = await invoke('neocodex_get_session_messages', { session_id: currentSessionId }).catch(() => null);
    if(Array.isArray(msgs)) renderThread(msgs);
  }

  function msgElByVid(vid){
    const cs = document.getElementById('chatScroll');
    if(!cs) return null;
    return cs.querySelector(`[data-vid="${vid}"]`);
  }

  async function editMessage(index){
    if(!isTauri() || !currentSessionId) return;
    const u = msgElByVid(index);
    const mb = u ? u.querySelector('.mb') : null;
    const content = mb ? (mb.textContent || '') : '';
    if(!u || !mb) return;
    // inline editor replacing the bubble (replaces native prompt() with a proper composer)
    const editor = document.createElement('div');
    editor.className = 'msg-edit';
    editor.innerHTML = `<textarea rows="3" placeholder="编辑消息…">${escHtml(content)}</textarea>
      <div class="me-actions">
        <span class="me-hint">编辑后，其后消息将重新生成</span>
        <button class="me-cancel" onclick="cancelMsgEdit()">取消</button>
        <button class="me-save" onclick="saveMsgEdit(${index})">保存</button>
      </div>`;
    u.replaceWith(editor);
    editor.querySelector('textarea').focus();
    window.__msgEditIndex = index;
  }

  function cancelMsgEdit(){
    const ed = document.querySelector('#chatScroll .msg-edit');
    if(ed && window.__msgEditIndex !== undefined) reloadThread();
  }

  async function saveMsgEdit(index){
    const ed = document.querySelector('#chatScroll .msg-edit');
    if(!ed) return;
    const ta = ed.querySelector('textarea');
    const next = ta ? ta.value.trim() : '';
    if(!next){ showToast('内容为空'); return; }
    const msgs = await invoke('neocodex_edit_message', { session_id: currentSessionId, index, content: next }).catch(e => { showToast('编辑失败: ' + e); return null; });
    if(Array.isArray(msgs)) renderThread(msgs);
  }

  async function deleteMessage(index){
    if(!isTauri() || !currentSessionId) return;
    const ok = await ntxConfirm('删除该消息？此操作不可恢复。', { title: '删除消息', confirmText: '删除', danger: true });
    if(!ok) return;
    const msgs = await invoke('neocodex_delete_message', { session_id: currentSessionId, index }).catch(e => { showToast('删除失败: ' + e); return null; });
    if(Array.isArray(msgs)) renderThread(msgs);
  }

  async function retryMessage(index){
    if(!isTauri() || !currentSessionId) return;
    try{ regenPush(currentSessionId, index); }catch(_e){}
    const msgs = await invoke('neocodex_regenerate', { session_id: currentSessionId, index }).catch(e => { showToast('重试失败: ' + e); return null; });
    if(Array.isArray(msgs)){
      renderThread(msgs);
      const lastUser = [...msgs].reverse().find(m => m.role === 'user');
      if(lastUser){
        const inp = document.getElementById('chatInput');
        inp.value = lastUser.content || '';
        sendMsg();
      }
    }
  }

  async function copyMessageContent(index){
    const cs = document.getElementById('chatScroll');
    const mb = cs.querySelectorAll('.msg.l')[index]?.querySelector('.mb');
    const text = mb?.innerText || mb?.textContent || '';
    try{ await navigator.clipboard.writeText(text); showToast('已复制'); }catch(_e){ showToast('复制失败'); }
  }

  async function copyUserContent(msgEl){
    const mb = msgEl?.querySelector('.mb');
    const text = mb?.innerText || mb?.textContent || '';
    try{ await navigator.clipboard.writeText(text); showToast('已复制'); }catch(_e){ showToast('复制失败'); }
  }

  async function copyAssistantContent(msgEl){
    const mb = msgEl?.querySelector('.mb');
    const text = mb?.innerText || mb?.textContent || '';
    try{ await navigator.clipboard.writeText(text); showToast('已复制'); }catch(_e){ showToast('复制失败'); }
  }

  /* ===== Session ops (会话操作: 重命名/压缩/归档/导出/删除) ===== */
  const FB_KEY = 'neotrix.feedback';
  function loadFeedback(){
    try{ return JSON.parse(localStorage.getItem(FB_KEY) || '{}'); }catch(_e){ return {}; }
  }
  function saveFeedback(fb){
    try{ localStorage.setItem(FB_KEY, JSON.stringify(fb)); }catch(_e){}
  }

  /* ===== Composer draft persistence (per session) ===== */
  const DRAFT_KEY = 'neotrix.drafts';
  function draftMap(){
    try{ return JSON.parse(localStorage.getItem(DRAFT_KEY) || '{}'); }catch(_e){ return {}; }
  }
  function saveDraft(){
    const inp = document.getElementById('chatInput');
    if(!inp) return;
    clearTimeout(window._draftT);
    window._draftT = setTimeout(() => {
      const key = currentSessionId || '__unsaved__';
      const dm = draftMap();
      const val = inp.value.trim();
      if(val) dm[key] = inp.value;
      else delete dm[key];
      try{ localStorage.setItem(DRAFT_KEY, JSON.stringify(dm)); }catch(_e){}
    }, 300);
  }
  function clearDraft(){
    const key = currentSessionId || '__unsaved__';
    const dm = draftMap();
    if(dm[key] !== undefined){ delete dm[key]; try{ localStorage.setItem(DRAFT_KEY, JSON.stringify(dm)); }catch(_e){} }
  }
  function restoreDraft(){
    const inp = document.getElementById('chatInput');
    if(!inp || inp.value.trim()) return;
    const key = currentSessionId || '__unsaved__';
    const dm = draftMap();
    if(dm[key]){
      inp.value = String(dm[key]);
      autoResize(inp);
    }
  }

  function sessionTitleFor(id){
    const s = CW_DATA.find(x => x.id === id);
    if(s) return s.name;
    const r = recentData.chat.find(x => x.id === id);
    return r ? r.text : id;
  }

  function openSessionOps(anchorEl, id){
    const menu = document.getElementById('sessionOpsMenu');
    if(!menu) return;
    const targetId = id || currentSessionId;
    if(!targetId){ showToast('请先打开一个会话'); return; }
    document.getElementById('sesOpsName').textContent = sessionTitleFor(targetId) || targetId;
    menu.dataset.session = String(targetId);
    menu.classList.add('open');
    if(anchorEl){
      const r = anchorEl.getBoundingClientRect();
      menu.style.left = Math.max(8, Math.min(r.left, window.innerWidth - 220)) + 'px';
      menu.style.top = (r.bottom + 6) + 'px';
    }else{
      menu.style.left = '50%';
      menu.style.top = '50%';
      menu.style.transform = 'translate(-50%,-50%)';
    }
  }
  function closeSessionOps(){
    const menu = document.getElementById('sessionOpsMenu');
    if(menu) menu.classList.remove('open');
  }
  document.addEventListener('click', (e) => {
    const menu = document.getElementById('sessionOpsMenu');
    if(menu && menu.classList.contains('open') && !e.target.closest('#sessionOpsMenu') && !e.target.closest('.ses-trigger')){
      menu.classList.remove('open');
    }
  });

  async function refreshSessionList(){
    await loadSessions().catch(() => {});
    renderCowork();
  }

  async function renameSession(){
    const id = document.getElementById('sessionOpsMenu')?.dataset.session || currentSessionId;
    if(!id){ showToast('无会话'); return; }
    closeSessionOps();
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可重命名'); return; }
    const next = prompt('新会话名称:', sessionTitleFor(id));
    if(next === null || !next.trim()) return;
    try{
      await invoke('neocodex_rename_session', { session_id: id, name: next.trim() });
      showToast('已重命名');
      await refreshSessionList();
    }catch(e){ showToast('重命名失败: ' + e); }
  }

  async function compactSession(){
    const id = document.getElementById('sessionOpsMenu')?.dataset.session || currentSessionId;
    if(!id){ showToast('无会话'); return; }
    closeSessionOps();
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可压缩'); return; }
    try{
      const msgs = await invoke('neocodex_compact_session', { session_id: id, keep_messages: 20 });
      if(Array.isArray(msgs)) renderThread(msgs);
      showToast('上下文已压缩，保留最近 20 条');
    }catch(e){ showToast('压缩失败: ' + e); }
  }

  async function archiveSession(){
    const id = document.getElementById('sessionOpsMenu')?.dataset.session || currentSessionId;
    if(!id){ showToast('无会话'); return; }
    closeSessionOps();
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可归档'); return; }
    try{
      await invoke('neocodex_archive_session', { session_id: id });
      showToast('已归档会话');
      if(id === currentSessionId){ currentSessionId = null; actions.newChat(); }
      await refreshSessionList();
    }catch(e){ showToast('归档失败: ' + e); }
  }

  async function exportSession(){
    const id = document.getElementById('sessionOpsMenu')?.dataset.session || currentSessionId;
    if(!id){ showToast('无会话'); return; }
    closeSessionOps();
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可导出'); return; }
    try{
      const out = await invoke('neocodex_export_session', { session_id: id, format: null });
      const name = (sessionTitleFor(id) || 'session').replace(/[^\w\u4e00-\u9fa5-]+/g, '_');
      const blob = new Blob([String(out || '')], { type: 'text/markdown' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url; a.download = name + '.md'; a.click();
      URL.revokeObjectURL(url);
      showToast('会话已导出');
    }catch(e){ showToast('导出失败: ' + e); }
  }

  async function deleteSession(){
    const id = document.getElementById('sessionOpsMenu')?.dataset.session || currentSessionId;
    if(!id){ showToast('无会话'); return; }
    closeSessionOps();
    const ok = await ntxConfirm('确定删除该会话？此操作不可恢复。', { title: '删除会话', confirmText: '删除', danger: true });
    if(!ok) return;
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可删除'); return; }
    try{
      await invoke('neocodex_delete_session', { session_id: id });
      showToast('会话已删除');
      if(id === currentSessionId){ currentSessionId = null; actions.newChat(); }
      await refreshSessionList();
    }catch(e){ showToast('删除失败: ' + e); }
  }

  async function openArchivedSessions(){
    closeSessionOps();
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可查看归档'); return; }
    const body = document.getElementById('archivedBody');
    if(!body) return;
    body.innerHTML = '<div class="cw-empty" style="padding:24px 12px;text-align:center;color:var(--tx2);font-size:var(--fs-small)">加载中…</div>';
    openOverlay('overlayArchived');
    try{
      const list = await invoke('neocodex_list_archived');
      const arr = Array.isArray(list) ? list : [];
      if(!arr.length){
        body.innerHTML = '<div class="cw-empty" style="padding:32px 12px;text-align:center;color:var(--tx2);font-size:var(--fs-small)">暂无归档会话</div>';
        return;
      }
      body.innerHTML = arr.map(s => {
        const id = s.id || '';
        const msgs = s.message_count ? ` · ${s.message_count} 消息` : '';
        const when = s.updated_at ? fmtRelTime(s.updated_at) : '';
        return `<div class="arch-item">
          <div class="arch-info">
            <div class="arch-name">${escHtml(s.name || '会话')}</div>
            <div class="arch-meta">${msgs}${when ? ' · ' + when : ''}</div>
          </div>
          <div class="arch-actions">
            <button class="arch-btn restore" onclick="restoreArchived('${escHtml(id)}')">恢复</button>
            <button class="arch-btn del" onclick="deleteArchived('${escHtml(id)}')">删除</button>
          </div>
        </div>`;
      }).join('');
    }catch(e){
      body.innerHTML = `<div class="cw-empty" style="padding:24px 12px;text-align:center;color:var(--tx3);font-size:var(--fs-small)">加载失败: ${escHtml(e)}</div>`;
    }
  }

  async function restoreArchived(id){
    if(!id || !isTauri()) return;
    try{
      await invoke('neocodex_restore_session', { session_id: id });
      showToast('会话已恢复');
      await openArchivedSessions();
      await refreshSessionList();
    }catch(e){ showToast('恢复失败: ' + e); }
  }

  async function deleteArchived(id){
    if(!id || !isTauri()) return;
    const ok = await ntxConfirm('彻底删除该归档会话？此操作不可恢复。', { title: '删除归档会话', confirmText: '删除', danger: true });
    if(!ok) return;
    try{
      await invoke('neocodex_delete_session', { session_id: id });
      showToast('归档会话已删除');
      await openArchivedSessions();
    }catch(e){ showToast('删除失败: ' + e); }
  }

  /* ===== Checkpoint 时间线 (Claude /rewind parity) ===== */
  async function openCheckpointTimeline(){
    const id = document.getElementById('sessionOpsMenu')?.dataset.session || currentSessionId;
    closeSessionOps();
    if(!id){ showToast('请先打开一个会话'); return; }
    const body = document.getElementById('checkpointBody');
    if(!body) return;
    document.getElementById('ckTitle').textContent = '会话时间线 · ' + (sessionTitleFor(id) || id);
    body.innerHTML = '<div class="cw-empty" style="padding:24px 12px;text-align:center;color:var(--tx2);font-size:var(--fs-small)">加载中…</div>';
    openOverlay('overlayCheckpoints');
    try{
      const list = await invoke('neocodex_checkpoint_list', { session_id: id });
      const arr = Array.isArray(list) ? list : [];
      if(!arr.length){
        body.innerHTML = '<div class="cw-empty" style="padding:32px 12px;text-align:center;color:var(--tx2);font-size:var(--fs-small)">该会话暂无 checkpoint（对话推进时会自动生成）</div>';
        return;
      }
      body.innerHTML = arr.map((ck, i) => {
        const when = ck.created_at ? fmtRelTime(ck.created_at) : '';
        const msgs = ck.message_count ? ` · ${ck.message_count} 消息` : '';
        const latest = i === 0 ? '<span class="ck-latest">当前</span>' : '';
        return `<div class="ck-item">
          <div class="ck-dot"></div>
          <div class="ck-info">
            <div class="ck-name">快照 #${arr.length - i}${latest ? '（最新）' : ''}</div>
            <div class="arch-meta">${when}${msgs}</div>
          </div>
          <button class="arch-btn restore" onclick="restoreCheckpoint('${escHtml(String(ck.id))}')">回滚到此</button>
        </div>`;
      }).join('');
      body.dataset.session = String(id);
    }catch(e){
      body.innerHTML = `<div class="cw-empty" style="padding:24px 12px;text-align:center;color:var(--tx3);font-size:var(--fs-small)">加载失败: ${escHtml(e)}</div>`;
    }
  }

  async function restoreCheckpoint(checkpointId){
    const body = document.getElementById('checkpointBody');
    const sessionId = body ? (body.dataset.session || currentSessionId) : currentSessionId;
    if(!checkpointId || !sessionId || !isTauri()) return;
    const ok = await ntxConfirm('回滚到该时间点？当前会话进度将被替换（代码与对话快照）。', { title: '回滚检查点', confirmText: '回滚', danger: true });
    if(!ok) return;
    try{
      const msgs = await invoke('neocodex_checkpoint_restore', { session_id: sessionId, checkpoint_id: checkpointId });
      closeOverlay('overlayCheckpoints');
      if(Array.isArray(msgs)){
        currentSessionId = sessionId;
        switchView(document.querySelector('.segb[data-view="chat"]'), 'chat');
        renderThread(msgs);
      }
      showToast('已回滚到该时间点');
    }catch(e){ showToast('回滚失败: ' + e); }
  }

  async function feedbackMessage(index, kind){
    if(!isTauri()){ showToast('浏览器模式：反馈仅在桌面端生效'); return; }
    if(!currentSessionId){ showToast('无会话'); return; }
    const fb = loadFeedback();
    const key = currentSessionId + ':' + index;
    const cur = fb[key];
    const next = (cur === kind) ? null : kind; // toggle
    if(next === null) delete fb[key]; else fb[key] = next;
    saveFeedback(fb);
    try{
      await invoke('neocodex_feedback', { session_id: currentSessionId, text: next === null ? '' : (next === 'like' ? '👍 有帮助' : '👎 需改进') });
    }catch(_e){}
    renderFeedbackState(index, next);
    showToast(next === null ? '已撤销反馈' : (next === 'like' ? '已标记有帮助' : '已标记需改进'));
  }

  function renderFeedbackState(index, state){
    const cs = document.getElementById('chatScroll');
    const msg = cs.querySelectorAll('.msg.l')[index];
    if(!msg) return;
    msg.querySelectorAll('.ma-btn[data-op="like"],.ma-btn[data-op="dislike"]').forEach(b => b.classList.remove('on'));
    if(state){
      const btn = msg.querySelector('.ma-btn[data-op="' + state + '"]');
      if(btn) btn.classList.add('on');
    }
  }

  /* ===== Session search (会话全文搜索, 对标 Codex ⌘G/Claude ⌘K) ===== */
  let lastSessionQuery = '';
  async function searchSessions(query){
    const q = (query || '').trim();
    const res = document.getElementById('cwSearchResults');
    if(!res) return;
    lastSessionQuery = q;
    if(!q){ res.style.display = 'none'; res.innerHTML = ''; return; }
    if(!isTauri()){
      const local = CW_DATA.filter(s => (s.name || '').toLowerCase().includes(q.toLowerCase()));
      renderSessionSearchResults(local.map(s => ({ session_id: s.id, session_name: s.name, role: '', snippet: '· ' + (s.status || ''), match_count: 1 })));
      return;
    }
    try{
      const hits = await invoke('neocodex_search_sessions', { query: q });
      if(lastSessionQuery !== q) return; // stale guard
      renderSessionSearchResults(Array.isArray(hits) ? hits : []);
    }catch(_e){
      renderSessionSearchResults([]);
    }
  }

  function renderSessionSearchResults(hits){
    const res = document.getElementById('cwSearchResults');
    if(!res) return;
    if(!hits.length){ res.innerHTML = '<div class="cw-empty" style="padding:10px 4px">无匹配会话</div>'; res.style.display = 'block'; return; }
    res.innerHTML = hits.slice(0, 20).map(h => {
      const role = h.role === 'user' ? '问' : (h.role === 'agent' || h.role === 'assistant' ? '答' : '');
      const hitsTxt = h.match_count > 1 ? `<span class="re-time">${h.match_count} 处</span>` : '';
      const when = h.timestamp ? fmtRelTime(h.timestamp) : '';
      return `<div class="re-i" onclick="openSessionFromSearch('${escHtml(String(h.session_id))}')">
        <span class="dot"></span><span class="t">${escHtml(h.session_name || '会话')}${role ? ' · ' + role : ''}</span>
        <span class="re-time">${hitsTxt}${when}</span>
        <div class="kb-hit-s" style="font-size:10px;color:var(--tx3);line-height:1.4">${escHtml(String(h.snippet || '').slice(0, 80))}</div>
      </div>`;
    }).join('');
    res.style.display = 'block';
  }

  async function openSessionFromSearch(id){
    if(!isTauri() || !id) return;
    try{
      await invoke('neocodex_switch_session', { session_id: id });
      currentSessionId = id;
      const msgs = await invoke('neocodex_get_session_messages', { session_id: id });
      if(Array.isArray(msgs)){
        switchView(document.querySelector('.segb[data-view="chat"]'), 'chat');
        renderThread(msgs);
      }
      showToast('已打开搜索结果');
    }catch(_e){}
  }

  async function loadSessionMessages(id){
    if(!isTauri() || !id) return;
    try{
      await invoke('neocodex_switch_session', { session_id: id });
      currentSessionId = id;
      const msgs = await invoke('neocodex_get_session_messages', { session_id: id });
      if(Array.isArray(msgs)) renderThread(msgs);
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

  /* 统一工作区状态栏 — 跨标签实时系统状态 (NeoTrix 特性: 记忆量/能力维度/进化迭代/模型/智能体) */
  async function loadWsStatus(){
    if(!isTauri()) return;
    try{
      const [bs, h] = await Promise.all([
        invoke('brain_stats').catch(() => null),
        invoke('neocodex_health_report').catch(() => null),
      ]);
      const set = (id, v) => { const el = document.getElementById(id); if(el) el.textContent = String(v); };
      if(bs){
        set('wsMemory', '记忆 ' + (bs.memory_count ?? 0));
        set('wsDims', '能力 ' + ((bs.dimension_names && bs.dimension_names.length) || 0) + ' 维');
        set('wsIter', '进化 ' + (bs.iteration ?? bs.absorb_count ?? 0));
      }
      const m = document.getElementById('wsModel');
      if(m){
        const cur = MODEL_POOL.find(x => x.id === currentModelId);
        if(cur) m.lastChild.textContent = cur.title;
      }
      try{
        const st = await invoke('neocodex_agent_status').catch(() => null);
        const ag = document.getElementById('wsAgent');
        if(ag && st){
          if(st.running){
            ag.classList.add('run');
            ag.lastChild.textContent = st.current_task ? '运行中 · ' + String(st.current_task).slice(0, 10) : '运行中';
          }else{
            ag.classList.remove('run');
            ag.lastChild.textContent = '空闲';
          }
        }
      }catch(_e){}
      const g = document.getElementById('heroGreeting');
      if(g && !g.dataset.greeted){
        const hr = new Date().getHours();
        g.textContent = hr < 5 ? '夜深了' : (hr < 12 ? '上午好' : (hr < 14 ? '中午好' : (hr < 18 ? '下午好' : '晚上好')));
        g.dataset.greeted = '1';
      }
      const meta = document.getElementById('heroMeta');
      if(meta && !meta.dataset.filled){
        const parts = [];
        if(bs) parts.push({ ic:'<svg viewBox="0 0 12 12"><path d="M6 1l1.2 3.3 3.3 1.2-3.3 1.2L6 10l-1.2-3.3-3.3-1.2 3.3-1.2z" stroke="currentColor" stroke-width="1" fill="none" stroke-linejoin="round"/></svg>', t:`VSA HyperCube · 记忆 <b>${bs.memory_count ?? 0}</b>` });
        parts.push({ ic:'<svg viewBox="0 0 12 12"><circle cx="6" cy="6" r="4" stroke="currentColor" stroke-width="1" fill="none"/><circle cx="6" cy="6" r="1" stroke="currentColor" stroke-width="1" fill="none"/></svg>', t:`能力 <b>${((bs && bs.dimension_names && bs.dimension_names.length) || 0)}</b> 维` });
        parts.push({ ic:'<svg viewBox="0 0 12 12"><circle cx="6" cy="6" r="4.5" stroke="currentColor" stroke-width="1" fill="none"/><path d="M6 3.5V6l2 1.2" stroke="currentColor" stroke-width="1" fill="none" stroke-linecap="round"/></svg>', t:`进化 <b>${(bs ? (bs.iteration ?? bs.absorb_count ?? 0) : 0)}</b> 代` });
        meta.innerHTML = parts.map(p => `<span class="hm">${p.ic}${p.t}</span>`).join('');
        meta.dataset.filled = '1';
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
    await Promise.all([loadSessions(), loadHealth(), loadFileTree(), loadProxy(), loadUsage(), loadWsStatus(), hydrateAppVersion()]);
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

    const cicLeftCtx = document.querySelector('#viewChat .cic-left');
    if(cicLeftCtx && !document.getElementById('ntxCtxWrap')){
      const wrap = document.createElement('div');
      wrap.className = 'ctx-wrap';
      wrap.style.cssText = 'position:relative;display:inline-flex;align-items:center';
      wrap.innerHTML = `<div id="ntxCtxMeter"></div>
        <div class="ct-pop" id="ntxCtxPop"></div>`;
      cicLeftCtx.appendChild(wrap);
    }

    const cic = document.querySelector('.cic');
    if(cic && !document.getElementById('ntxToolbar')){
      const tb = document.createElement('div');
      tb.className = 'ntx-chat-toolbar';
      tb.innerHTML = `<div class="l" id="ntxAttachArea"></div><div class="r" id="ntxToolbarRight"></div>`;
      cic.insertBefore(tb, cic.querySelector('.cic-actions'));
    }

    const tbRight = document.getElementById('ntxToolbarRight');
    if(tbRight && !document.getElementById('ntxModeBtn')){
      tbRight.innerHTML += `
        <button class="ntx-mode-btn ses-trigger" id="ntxModeBtn" title="权限模式 (对标 Codex approval)" onclick="cycleMode()"><span id="ntxModeLabel">手动</span></button>
        <button class="ntx-mode-btn ses-trigger" id="ntxSessionOps" title="会话操作" onclick="openSessionOps(this)"><span>⋯</span></button>`;
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
      const state = m.online ? `<span class="ntx-mm-lat">${m.lat ? escHtml(String(m.lat)) + 'ms' : '—'}</span>` : `<span class="ntx-mm-lat off">离线</span>`;
      return `<button class="ntx-mm-item${on ? ' on' : ''}" data-id="${escHtml(m.id)}">
        <span class="ntx-mm-title"><span class="dot" style="background:${on ? 'var(--suc,#4CAF50)' : 'var(--tx3)'}"></span>${escHtml(m.title)}</span>
        <span class="ntx-mm-desc">${escHtml(m.model)}</span>${state}
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
          if(idx >= 0){
            MODEL_POOL[idx].model = p.model || MODEL_POOL[idx].model;
            MODEL_POOL[idx].online = !!p.resolvable;
            // 后端 neocodex_provider_config 不提供真实延迟 → 置 null 显示 "—",
            // 避免把 browser demo 的静态假 lat 渲染给 Tauri 用户 (D7).
            MODEL_POOL[idx].lat = null;
          }
          else if(!known.includes(p.name)){
            MODEL_POOL.push({ id: p.name, title: p.name, model: p.model || '', lat: null, online: !!p.resolvable });
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
    if(act === 'ref'){
      openRefPicker();
      return;
    }
    if(act === 'achievements'){ showAchievements(); return; }
    if(act === 'registry'){ openOverlay('overlayRegistry'); loadRegistry(); return; }
    if(act === 'hypercube'){ openOverlay('overlayHypercube'); return; }
    if(act === 'slash'){ focusComposerSlash(); return; }
    showToast('「' + { slash: '命令 (Slash)', ref: '引用上下文' }[act] + '」功能开发中');
  }

  function focusComposerSlash(){
    const inp = document.getElementById('chatInput');
    if(!inp) return;
    inp.focus();
    inp.value = '/';
    inp.selectionStart = inp.selectionEnd = 1;
    try{ QMUpdate(); }catch(_e){}
  }

  /* 引用上下文 — 列出当前会话的历史消息, 点选后以引用块插入输入框 */
  async function openRefPicker(){
    if(!isTauri() || !currentSessionId){ showToast('需在会话中引用上下文'); return; }
    const msgs = await invoke('neocodex_get_session_messages', { session_id: currentSessionId }).catch(() => null);
    let picker = document.getElementById('ntxRefPicker');
    if(!picker){
      picker = document.createElement('div');
      picker.id = 'ntxRefPicker';
      picker.className = 'ref-picker';
      picker.innerHTML = `<div class="rf-head">引用上下文 <button class="rf-close" onclick="closeRefPicker()">×</button></div><div class="rf-body"></div>`;
      const ntxWrap = document.querySelector('#viewChat .cic-plus-wrap, #viewChat .cic-left');
      (ntxWrap || document.body).appendChild(picker);
    }
    const body = picker.querySelector('.rf-body');
    const list = Array.isArray(msgs) ? msgs : [];
    const recent = list.slice(-8);
    if(!recent.length){
      body.innerHTML = '<div class="rf-empty">暂无历史消息可引用</div>';
    } else {
      body.innerHTML = recent.map((m, i) => {
        const who = m.role === 'user' ? '我' : (m.role === 'tool' ? '工具' : 'NeoTrix');
        const txt = String(m.content || '').slice(0, 90);
        return `<button class="rf-item" data-i="${i}"><span class="rf-who">${who}</span><span class="rf-txt">${escHtml(txt)}</span></button>`;
      }).join('');
      body.querySelectorAll('.rf-item').forEach(el => {
        el.addEventListener('click', () => {
          const idx = Number(el.dataset.i);
          insertReference(recent[idx]);
          closeRefPicker();
        });
      });
    }
    picker.classList.add('open');
  }

  function insertReference(msg){
    const inp = document.getElementById('chatInput');
    if(!inp) return;
    const who = msg.role === 'user' ? '我' : (msg.role === 'tool' ? '工具输出' : 'NeoTrix');
    const quote = String(msg.content || '').slice(0, 400);
    const block = `[引用·${who}] ${quote}\n\n`;
    inp.value = block + (inp.value || '');
    inp.selectionStart = inp.selectionEnd = inp.value.length;
    inp.focus(); autoResize(inp);
  }

  function closeRefPicker(){
    const picker = document.getElementById('refPicker');
    if(picker) picker.classList.remove('open');
    const el = document.getElementById('ntxRefPicker');
    if(el) el.classList.remove('open');
  }

  function addAttachChip(name, meta){
    const area = document.getElementById('ntxAttachArea');
    if(!area) return;
    if(attachList.some(f => f.name === name)) return;
    const att = { name, size: meta?.size || 0, mime_type: meta?.mime || '', data: meta?.data ?? null };
    attachList.push(att);
    const chip = document.createElement('span');
    chip.className = 'ntx-attach-chip';
    const sz = att.size ? (att.size > 1024 ? (att.size/1024).toFixed(1)+'K' : att.size+'B') : '';
    chip.innerHTML = `<svg viewBox="0 0 12 12" class="at-ic"><path d="M2.5 1h4L9.5 4v7h-7z" fill="none" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/><path d="M6.5 1v3h3" fill="none" stroke="currentColor" stroke-width="1.1"/></svg><span class="at-nm">${escHtml(name)}</span>${sz?`<span class="at-sz">${sz}</span>`:''}<span class="x" data-f="${escHtml(name)}">×</span>`;
    chip.querySelector('.x').addEventListener('click', () => {
      attachList = attachList.filter(f => f.name !== name);
      chip.remove();
    });
    area.appendChild(chip);
  }

  function attachPayloads(){
    // clip oversized payloads at the frontend too (backend enforces a 10MB cap)
    return attachList
      .filter(a => a.name && !a.name.startsWith('@'))
      .map(a => ({
        name: a.name,
        size: Number(a.size) || 0,
        mime_type: a.mime_type || '',
        data: (a.data && a.data.length > 8 * 1024 * 1024) ? a.data.slice(0, 8 * 1024 * 1024) : (a.data ?? null),
      }));
  }

  function mimeFromName(name){
    const ext = (name.split('.').pop() || '').toLowerCase();
    const map = {
      md:'text/markdown', rs:'text/plain', js:'text/javascript', ts:'text/typescript',
      tsx:'text/typescript', json:'application/json', toml:'text/plain', yml:'application/yaml',
      yaml:'application/yaml', txt:'text/plain', py:'text/x-python', go:'text/x-go',
      sh:'application/x-sh', css:'text/css', html:'text/html', csv:'text/csv',
      png:'image/png', jpg:'image/jpeg', jpeg:'image/jpeg', gif:'image/gif', webp:'image/webp', svg:'image/svg+xml',
      pdf:'application/pdf', sql:'text/plain', vue:'text/plain', svelte:'text/plain',
    };
    return map[ext] || 'application/octet-stream';
  }

  async function pickAttachment(){
    // Tauri: native file dialog + fs read. Browser: hidden <input type=file>.
    if(isTauri()){
      try{
        const picked = await invoke('plugin:dialog|open', {
          options: { multiple: true, title: '附加文件', directory: false },
        });
        const files = Array.isArray(picked) ? picked : (picked ? [picked] : []);
        for(const path of files){
          const rel = String(path).replace(/^.*\/([^/]+)$/, '$1');
          let data = null, size = 0, mime = '';
          try{
            const arr = await invoke('plugin:fs|readFile', { path });
            size = (arr && arr.length) || 0;
            if(size <= 512 * 1024){
              const bytes = new Uint8Array(arr);
              const enc = new TextDecoder('utf-8');
              data = enc.decode(bytes); // text-first; non-UTF8 garbage is acceptable
              mime = mimeFromName(rel);
            }
          }catch(_e){ /* binary/too-large: name-only attachment */ }
          addAttachChip(rel, { size, mime, data });
        }
        if(files.length) showToast('已附加 ' + files.length + ' 个文件');
        else showToast('未选择文件');
      }catch(e){ showToast('附加失败: ' + String(e).slice(0, 60)); }
      return;
    }
    let input = document.getElementById('atFileInput');
    if(!input){
      input = document.createElement('input');
      input.id = 'atFileInput';
      input.type = 'file';
      input.multiple = true;
      input.style.display = 'none';
      document.body.appendChild(input);
    }
    input.onchange = () => {
      [...(input.files || [])].slice(0, 6).forEach(async f => {
        let data = null, mime = f.type || '';
        if(f.size <= 512 * 1024){
          data = await f.text().catch(() => null);
        }
        addAttachChip(f.name, { size: f.size, mime: mime || mimeFromName(f.name), data });
      });
      input.value = '';
    };
    input.click();
  }

  let lastContextUsage = 0;
  let lastContextMeta = { turns: 0, tokens: 0, providerModel: '', costSpent: 0, costBudget: 0 };
  let contextWarned = false;
  async function loadUsage(){
    if(!isTauri()){ renderContextMeter(); return; }
    try{
      const h = await invoke('neocodex_health_report');
      lastContextUsage = Math.max(0, Math.min(1, (h && h.context_usage) || 0));
      if(h && typeof h === 'object'){
        lastContextMeta = {
          turns: (h.context_turns ?? h.turn_count ?? 0),
          tokens: (h.tokens_used ?? 0),
          tool: (h.provider_model ?? '') || (h.mode ?? ''),
          toolsSpent: (h.tool_call_count ?? 0),
          costSpent: (h.cost_spent ?? 0),
          costBudget: (h.cost_budget ?? 0),
        };
      }
      renderContextMeter();
    }catch(_e){ renderContextMeter(); }
  }

  function renderContextMeter(){
    const el = document.getElementById('ntxCtxMeter');
    if(!el) return;
    const pct = Math.round(lastContextUsage * 100);
    const warn = lastContextUsage >= 0.9;
    if(warn && !contextWarned){ contextWarned = true; showToast('上下文接近上限 ('+pct+'%)，建议 /compact 压缩'); }
    if(!warn) contextWarned = false;
    const tone = warn ? 'danger' : (lastContextUsage >= 0.7 ? 'high' : '');
    el.innerHTML = `<button class="ctx-chip ${tone}" onclick="toggleCtxPop()" title="上下文状况"
      style="--pct:${pct}%"> <span class="ctx-ring"><i></i></span>
      <span class="ctx-label">${warn ? '⚠ ' : ''}上下文 ${pct}%</span></button>`;
  }

  function toggleCtxPop(){
    const pop = document.getElementById('ntxCtxPop');
    if(!pop) return;
    const open = pop.classList.toggle('open');
    if(open) hydrateCtxPop();
  }

  function hydrateCtxPop(){
    const pop = document.getElementById('ntxCtxPop');
    if(!pop) return;
    const pct = Math.round(lastContextUsage * 100);
    const m = lastContextMeta;
    pop.innerHTML = `
      <div class="ct-pop-h"><span>上下文使用</span><span class="ct-pct">${pct}%</span></div>
      <div class="ct-bar"><i style="--pct:${pct}%"></i></div>
      <div class="ct-row"><span>对话轮次</span><b>${m.turns}</b></div>
      <div class="ct-row"><span>tokens 已用</span><b>${m.tokens.toLocaleString()}</b></div>
      <div class="ct-row"><span>工具调用</span><b>${m.toolsSpent ?? 0}</b></div>
      <div class="ct-row"><span>模型</span><b class="ct-mono">${m.tool || '—'}</b></div>
      ${m.costBudget ? `<div class="ct-row"><span>支出</span><b>$${m.costSpent.toFixed(2)} / $${m.costBudget.toFixed(2)}</b></div>` : ''}
      <div class="ct-foot"><button class="ct-act" onclick="window.compactSession()">压缩上下文 /compact</button></div>`;
  }

  function attachUsageFooter(msgEl){
    const mb = msgEl.querySelector('.mb');
    if(!mb || mb.querySelector('.msg-usage')) return;
    const el = document.createElement('div');
    el.className = 'msg-usage';
    el.textContent = '上下文 ' + Math.round(lastContextUsage * 100) + '%';
    mb.appendChild(el);
  }

  /* ChatGPT/Claude parity: 流式生成的回复完成后, 追加复制按钮(与 renderThread 的
     assistant 消息保持一致). 幂等: 已有则跳过. */
  function attachAssistantCopy(msgEl){
    if(!msgEl || msgEl.querySelector('.msg-act .ma-btn[data-op="copy"]')) return;
    const act = msgEl.querySelector('.msg-act');
    const bar = act || document.createElement('div');
    if(!act) bar.className = 'msg-act';
    const btn = document.createElement('button');
    btn.className = 'ma-btn';
    btn.dataset.op = 'copy';
    btn.title = '复制内容';
    btn.textContent = '复制';
    btn.onclick = () => copyAssistantContent(msgEl);
    bar.appendChild(btn);
    if(!act) msgEl.insertBefore(bar, msgEl.querySelector('.mb'));
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
    let isSample = false;
    if(isTauri()){
      try { data = await invoke('neocodex_get_diff'); } catch(_e){}
    }
    if(!data || !Array.isArray(data.files) || !data.files.length){
      data = SAMPLE_DIFF;
      isSample = true;
    }
    renderDiff(data, isSample);
    openOverlay('overlayDiff');
  }

  function renderDiff(data, isSample){
    const title = document.getElementById('diffTitle');
    if(title) title.textContent = '代码变更 · ' + data.files.length + ' 文件' + (isSample ? '（示例数据）' : '');
    const body = document.getElementById('diffBody');
    if(!body) return;
    if(isSample){
      const hint = document.createElement('div');
      hint.className = 'diff-sample-hint';
      hint.textContent = '⚠ 当前为示例数据（未检测到真实代码变更）。点击文件运行代码或修改文件后刷新。';
      body.appendChild(hint);
    }
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
        <div class="df-path"><svg viewBox="0 0 12 12"><path d="M10.5 5v4.5a1 1 0 01-1 1h-7a1 1 0 01-1-1v-7a1 1 0 011-1H5" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d="M7.5 1.5h3v3" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round"/><path d="M6.5 5.5l4-4" stroke="currentColor" stroke-width="1.1" fill="none" stroke-linecap="round"/></svg><span class="df-fname">${escHtml(f.path)}</span></div>
        <div class="df-actions">
          <button class="df-act accept" onclick="diffApply(${fi},'accept')">接受</button>
          <button class="df-act reject" onclick="diffApply(${fi},'reject')">放弃</button>
        </div>
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

  async function diffApply(fi, action){
    const body = document.getElementById('diffBody');
    const fileEl = body ? body.querySelectorAll('.df-file')[fi] : null;
    const pathEl = fileEl ? fileEl.querySelector('.df-fname') : null;
    if(!pathEl) return;
    const path = pathEl.textContent.trim();
    if(action === 'reject'){
      const ok = await ntxConfirm('放弃该文件的全部改动？(git restore)', { title: '放弃改动', confirmText: '放弃', danger: true });
      if(!ok) return;
    }
    try{
      if(isTauri()){
        await invoke('neocodex_apply_diff', { path, action });
      } else {
        throw new Error('仅桌面端支持');
      }
      fileEl.classList.add('df-done');
      const actions = fileEl.querySelector('.df-actions');
      if(actions){
        actions.innerHTML = `<span class="df-done-tag">${action === 'accept' ? '✓ 已接受' : '✓ 已放弃'}</span>`;
      }
    }catch(e){
      window.alert('操作失败: ' + (e && e.message ? e.message : e));
    }
  }

  window.diffApply = diffApply;

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

  /* ===== Global Command Palette (⌘K / sidebar search) ===== */
  const PAL_ACTIONS = [
    { act: 'new',      hint: '⌘N', label: '新建会话' },
    { act: 'settings', hint: '⌘,', label: '设置' },
    { act: 'diff',     hint: '/',  label: '查看 Diff' },
    { act: 'registry', hint: '◈',  label: '能力注册表' },
    { act: 'hypercube',hint: '◆',  label: '知识库' },
    { act: 'projects', hint: '▤',  label: '活跃项目' },
    { act: 'cowork',   hint: '⌘2', label: '团队 · 协同' },
  ];
  const PAL_ACT_MAP = { new: createSession, settings: openSettingsModal, diff: () => openOverlay('overlayDiff'), registry: () => { openOverlay('overlayRegistry'); loadRegistry(); }, hypercube: () => openOverlay('overlayHypercube'), projects: () => openOverlay('overlayProjects'), cowork: () => { const b = document.querySelector('.segb[data-view="cowork"]'); if(b) switchView(b, 'cowork'); } };
  let lastPalQuery = '';

  function openPalette(){
    const ov = document.getElementById('overlayPalette');
    if(!ov) return;
    ov.classList.add('open');
    updateTrafficVisibility();
    const inp = document.getElementById('palInput');
    if(inp){ inp.value = ''; palFilter(''); setTimeout(() => inp.focus(), 30); }
  }

  function closePalette(){
    const ov = document.getElementById('overlayPalette');
    if(ov) ov.classList.remove('open');
    updateTrafficVisibility();
  }

  async function palFilter(q){
    const body = document.getElementById('palBody');
    const results = document.getElementById('palResults');
    const grp = document.getElementById('palResultsGrp');
    const empty = document.getElementById('palEmpty');
    const query = (q || '').trim();
    const acts = Array.from(document.querySelectorAll('#palBody .pal-item[data-act]'));
    acts.forEach(it => {
      const label = (it.textContent || '').toLowerCase();
      it.style.display = (!query || label.includes(query.toLowerCase())) ? '' : 'none';
    });
    if(!query){
      if(results){ results.innerHTML = ''; results.style.display = 'none'; }
      if(grp) grp.style.display = 'none';
      if(empty) empty.style.display = 'none';
      return;
    }
    lastPalQuery = query;
    let hits = [];
    if(isTauri()){
      try{
        const r = await invoke('neocodex_search_sessions', { query });
        hits = Array.isArray(r) ? r : [];
      }catch(_e){ hits = []; }
    }
    if(lastPalQuery !== query) return;
    const sessionHits = hits.filter(h => h && h.session_id);
    if(grp) grp.style.display = sessionHits.length ? '' : 'none';
    if(empty) empty.style.display = (sessionHits.length ? 'none' : '');
    if(results){
      results.style.display = sessionHits.length ? '' : 'none';
      results.innerHTML = sessionHits.slice(0, 12).map(h => {
        const role = h.role === 'user' ? '问' : (h.role === 'agent' || h.role === 'assistant' ? '答' : '');
        const hitsTxt = h.match_count > 1 ? ` <span class="re-time">${h.match_count} 处</span>` : '';
        const when = h.timestamp ? fmtRelTime(h.timestamp) : '';
        return `<button class="pal-item pal-hit" data-sid="${escHtml(String(h.session_id))}" onclick="palPick(this)">
          <span class="pal-a">⇥</span><span class="pal-hit-t">${escHtml(h.session_name || '会话')}${role ? ' · ' + role : ''}</span>${hitsTxt}
          ${when ? `<span class="re-time">${when}</span>` : ''}
        </button>`;
      }).join('');
    }
  }

  function palKey(e){
    if(e.key === 'Escape'){ e.preventDefault(); e.stopPropagation(); closePalette(); return; }
    if(e.key === 'ArrowDown' || e.key === 'ArrowUp'){
      const items = Array.from(document.querySelectorAll('#palBody .pal-item:not([style*="display: none"])'));
      if(!items.length) return;
      e.preventDefault();
      const idx = items.findIndex(it => it.classList.contains('sel'));
      const next = e.key === 'ArrowDown' ? (idx + 1) % items.length : (idx - 1 + items.length) % items.length;
      items.forEach(it => it.classList.remove('sel'));
      items[next].classList.add('sel');
      items[next].scrollIntoView({ block: 'nearest' });
      return;
    }
    if(e.key === 'Enter'){
      const sel = document.querySelector('#palBody .pal-item.sel');
      if(sel){ e.preventDefault(); palPick(sel); }
    }
  }

  function palPick(el){
    const sid = el.getAttribute('data-sid');
    if(sid){ closePalette(); openSessionFromSearch(sid); return; }
    const act = el.getAttribute('data-act');
    closePalette();
    const fn = PAL_ACT_MAP[act];
    if(fn) fn();
    else if(act) showToast('功能开发中: ' + act);
  }

  /* Cmd/Ctrl+K 已在全局 keydown 处理；点击侧栏搜索按钮同开面板 */
  g.openPalette = openPalette;
  g.closePalette = closePalette;
  g.palFilter = palFilter;
  g.palKey = palKey;
  g.palPick = palPick;
  g.jumpToLatest = jumpToLatest;
  g.saveDraft = saveDraft;
  g.restoreDraft = restoreDraft;
  g.clearDraft = clearDraft;

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

  function filterSettings(q){
    const items = document.querySelectorAll('#overlaySettings .st-item');
    const query = (q || '').trim().toLowerCase();
    items.forEach(it => {
      const text = (it.textContent || '').toLowerCase();
      it.style.display = (!query || text.includes(query)) ? '' : 'none';
    });
    const grps = document.querySelectorAll('#overlaySettings .st-grp');
    grps.forEach(grp => {
      let next = grp.nextElementSibling;
      let anyVisible = false;
      while(next && !next.classList.contains('st-grp')){
        if(next.classList.contains('st-item') && next.style.display !== 'none') anyVisible = true;
        next = next.nextElementSibling;
      }
      grp.style.display = (!query || anyVisible) ? '' : 'none';
    });
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
    if(section==='security') await renderStSecurity();
    if(section==='profile') initProfileHandlers();
    if(section==='appearance') initAppearanceHandlers();
    if(section==='speech') initSpeechHandlers();
    if(section==='compute') initComputeHandlers();
    if(section==='compute') initApiKeyHandlers();
    if(section==='privacy') initPrivacyHandlers();
  }

  function initProfileHandlers(){
    const inputs = document.querySelectorAll('#stProfile input, #stProfile select');
    inputs.forEach(el => {
      const key = el.id || el.name || (el.previousElementSibling?.textContent || '').trim();
      el.onchange = () => {
        saveSetting('profile.' + key, el.value);
        showToast('已保存: ' + (el.previousElementSibling?.textContent || el.name || '设置'));
      };
    });
  }

  function initAppearanceHandlers(){
    const fontSel = document.querySelector('#stAppearance select');
    if(fontSel){
      const saved = loadSettings()['appearance.fontSize'];
      if(saved && saved !== fontSel.value) fontSel.value = saved;
      fontSel.onchange = () => {
        saveSetting('appearance.fontSize', fontSel.value);
        document.documentElement.style.fontSize = fontSel.value + 'px';
        showToast('字体大小已更改: ' + fontSel.value);
      };
    }
    const reduceTrans = document.querySelector('#stAppearance input[type="checkbox"]');
    if(reduceTrans){
      const saved = loadSettings()['appearance.reduceTransparency'];
      if(saved !== undefined) reduceTrans.checked = !!saved;
      reduceTrans.onchange = () => {
        saveSetting('appearance.reduceTransparency', reduceTrans.checked);
        document.documentElement.classList.toggle('reduce-trans', reduceTrans.checked);
        showToast(reduceTrans.checked ? '已开启减少透明效果' : '已关闭减少透明效果');
      };
    }
  }

  function initSpeechHandlers(){
    const inputs = document.querySelectorAll('#stSpeech input, #stSpeech select');
    inputs.forEach(el => {
      const key = el.id || el.name || (el.previousElementSibling?.textContent || '设置').trim();
      const saved = loadSettings()['speech.' + key];
      if(saved !== undefined){
        if(el.type === 'checkbox') el.checked = !!saved;
        else el.value = String(saved);
      }
      el.onchange = () => {
        saveSetting('speech.' + key, el.type === 'checkbox' ? el.checked : el.value);
        showToast('语音设置已更改: ' + (el.previousElementSibling?.textContent || '设置'));
      };
    });
  }

  function initComputeHandlers(){
    const providerSel = document.querySelector('#stCompute select');
    if(providerSel){
      providerSel.onchange = async () => {
        if(isTauri()){
          try{
            await invoke('neocodex_set_provider', { name: providerSel.value });
            saveSetting('compute.provider', providerSel.value);
            showToast('默认提供者已切换: ' + providerSel.value);
          }catch(e){ showToast('切换失败: ' + e); }
        }else{
          saveSetting('compute.provider', providerSel.value);
          showToast('浏览器模式：仅 Tauri 下可切换提供者');
        }
      };
    }
    const tokenSel = document.querySelector('#stMaxTokens') || document.querySelector('#stCompute select:last-of-type');
    if(tokenSel){
      const savedT = loadSettings()['compute.maxTokens'];
      if(savedT !== undefined) tokenSel.value = String(savedT);
      tokenSel.onchange = () => {
        saveSetting('compute.maxTokens', Number(tokenSel.value));
        showToast('最大 Token 已设为: ' + tokenSel.value + '（下次发送生效）');
      };
    }
    const tempRange = document.getElementById('stTemperature');
    if(tempRange){
      const savedTemp = loadSettings()['compute.temperature'];
      if(savedTemp !== undefined){
        tempRange.value = String(savedTemp);
        const tv = document.getElementById('stTemperatureVal');
        if(tv) tv.textContent = String(Number(savedTemp).toFixed(1));
      }
      tempRange.oninput = () => {
        const tv = document.getElementById('stTemperatureVal');
        if(tv) tv.textContent = Number(tempRange.value).toFixed(1);
      };
      tempRange.onchange = () => {
        saveSetting('compute.temperature', Number(tempRange.value));
        showToast('温度已设为: ' + Number(tempRange.value).toFixed(1) + '（下次发送生效）');
      };
    }
    const localInfer = document.querySelector('#stCompute input[type="checkbox"]');
    if(localInfer){
      const saved = loadSettings()['compute.localInfer'];
      if(saved !== undefined) localInfer.checked = !!saved;
      localInfer.onchange = () => {
        saveSetting('compute.localInfer', localInfer.checked);
        showToast(localInfer.checked ? '已启用本地推理引擎' : '已禁用本地推理引擎');
      };
    }
  }

  function initPrivacyHandlers(){
    const switches = document.querySelectorAll('#stPrivacy input[type="checkbox"]');
    const labels = ['对话存储', '使用数据', '本地处理'];
    switches.forEach((sw, i) => {
      const saved = loadSettings()['privacy.' + labels[i]];
      if(saved !== undefined) sw.checked = !!saved;
      sw.onchange = () => {
        saveSetting('privacy.' + labels[i], sw.checked);
        showToast((sw.checked ? '已开启' : '已关闭') + labels[i]);
      };
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
    await renderStApiKey();
  }

  /* ===== API Key mask + keychain (plan #14: Settings Provider) ===== */
  function maskApiKey(key){
    if(!key) return '';
    if(key.length <= 8) return '•'.repeat(key.length);
    return key.slice(0, 4) + '••••' + key.slice(-4);
  }

  async function renderStApiKey(){
    const input = document.getElementById('stApiKey');
    const state = document.getElementById('stApiKeyState');
    if(!input) return;
    input.value = '';
    let has = false;
    if(isTauri()){
      try{ has = !!(await invoke('has_api_key')); }catch(_e){}
    }
    if(state){
      state.textContent = has
        ? '已保存到系统钥匙串（如要更换，输入新 Key 保存即可）'
        : '未配置 API Key（保存到系统钥匙串，不落盘明文）';
    }
    input.placeholder = has ? 'sk-••••••••（已配置，覆盖保存）' : 'sk-…（留空则不配置）';
  }

  function initApiKeyHandlers(){
    const input = document.getElementById('stApiKey');
    const toggle = document.getElementById('stApiKeyToggle');
    if(!input) return;
    const state = document.getElementById('stApiKeyState');
    if(toggle){
      toggle.onclick = () => {
        const show = input.type === 'password';
        input.type = show ? 'text' : 'password';
        toggle.textContent = show ? '🙈' : '👁';
      };
    }
    input.onchange = async () => {
      const key = (input.value || '').trim();
      if(!key){
        if(isTauri()){
          try{ await invoke('delete_api_key'); }catch(_e){}
          if(state) state.textContent = '已清除 API Key';
          showToast('已清除 API Key');
        }
        return;
      }
      if(isTauri()){
        try{
          await invoke('save_api_key', { key });
          input.value = '';
          input.type = 'password';
          if(toggle) toggle.textContent = '👁';
          if(state) state.textContent = '已保存到系统钥匙串（如要更换，输入新 Key 保存即可）';
          showToast('API Key 已保存到系统钥匙串');
        }catch(e){ showToast('保存失败: ' + e); }
      }else{
        showToast('浏览器模式：仅 Tauri 下可保存到钥匙串');
      }
    };
  }

  async function renderStLimits(){
    const s = loadSettings();
    try{
      const bars = document.querySelectorAll('#stLimits .gbar-f');
      if(bars.length >= 2){
        const used = Math.min(100, Math.round((Number(s['limits.usedPct']) || 0)));
        bars[0].style.width = used + '%';
        bars[1].style.width = Math.min(100, used + 20) + '%';
      }
      document.querySelectorAll('#stLimits .st-desc').forEach((d,i) => {
        if(i===0) d.textContent = `已用 ${s['limits.used'] || 0} / ${s['limits.quota'] || 200} 次`;
        if(i===1) d.textContent = `请求/分钟 ${s['limits.rpm'] || 18}/30 · 令牌/分钟 ${s['limits.tpm'] || 45}K/100K`;
      });
    }catch(e){ console.error('renderStLimits failed:', e); }
  }

  async function renderStPrivacy(){
    const s = loadSettings();
    const switches = document.querySelectorAll('#stPrivacy input[type="checkbox"]');
    const labels = ['对话存储', '使用数据', '本地处理'];
    if(switches.length >= 3){
      switches[0].checked = s['privacy.对话存储'] !== false;
      switches[1].checked = !!s['privacy.使用数据'];
      switches[2].checked = s['privacy.本地处理'] !== false;
    }
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

  /* ===== 安全中心 (审批面板 + 审计日志) ===== */
  function permModeLabel(){
    const map = { '手动': '手动：敏感操作逐个确认', '自动': '自动：敏感操作自动放行', '计划': '计划：仅非破坏性操作自动放行' };
    return map[currentMode] || '手动：敏感操作逐个确认';
  }

  async function renderStSecurity(){
    const modeDesc = document.getElementById('stSecModeDesc');
    if(modeDesc) modeDesc.textContent = permModeLabel();
    if(!isTauri()){ return; }
    try{
      const pending = await invoke('get_pending_permissions');
      const list = document.getElementById('stSecPendingList');
      const desc = document.getElementById('stSecPendingDesc');
      if(list && Array.isArray(pending)){
        const items = pending.filter(r => r.status === 'Pending' || !r.status);
        if(items.length){
          if(desc) desc.textContent = items.length + ' 个请求等待处理';
          list.innerHTML = items.map(r => {
            const act = escHtml(String(r.action || ''));
            const tgt = escHtml(String(r.target || ''));
            const rid = escHtml(String(r.id || ''));
            const ts = r.timestamp ? fmtRelTime(Number(r.timestamp)) : '';
            return `<div style="display:flex;align-items:center;gap:8px;padding:6px 8px;border:1px solid var(--bd);border-radius:8px;background:var(--ghost)">
              <span style="flex:1;font-size:var(--fs-small);color:var(--tx2)"><b>${act}</b> → <code style="font-size:11px">${tgt}</code> <span style="color:var(--tx-meta);font-size:10.5px">${ts}</span></span>
              <button class="msg-ipc-retry" data-act="allow" data-id="${rid}" style="color:var(--suc)">允许</button>
              <button class="msg-ipc-retry" data-act="deny" data-id="${rid}" style="color:var(--err,#E5484D)">拒绝</button>
            </div>`;
          }).join('');
          list.querySelectorAll('button[data-act]').forEach(btn => {
            btn.onclick = async () => {
              const id = btn.dataset.id;
              const approved = btn.dataset.act === 'allow';
              try{
                await invoke('respond_permission', { requestId: id, approved });
                showToast(approved ? '已允许该操作' : '已拒绝该操作');
              }catch(e){ showToast('操作失败: ' + e); }
              await renderStSecurity();
            };
          });
        } else {
          if(desc) desc.textContent = '无挂起请求';
          list.innerHTML = '<span style="color:var(--tx-meta);font-size:var(--fs-caption)">所有敏感操作均已按策略处理</span>';
        }
      }
    }catch(e){ console.error('renderStSecurity pending failed:', e); }
    try{
      const audit = await invoke('get_permission_audit_log', { count: 50 });
      const al = document.getElementById('stSecAuditList');
      if(al && Array.isArray(audit)){
        if(!audit.length){
          al.innerHTML = '<span style="color:var(--tx-meta);font-size:var(--fs-caption)">暂无审计记录</span>';
          return;
        }
        al.innerHTML = audit.map(a => {
          const ts = a.timestamp ? fmtRelTime(Number(a.timestamp)) : '';
          const res = String(a.resolution || '');
          const cls = res === 'denied' ? 'var(--err,#E5484D)' : (res === 'approved' ? 'var(--suc)' : 'var(--tx3)');
          return `<div style="display:flex;gap:8px;font-size:var(--fs-caption);color:var(--tx2);padding:3px 0;border-bottom:1px solid var(--bd)">\
<span style="color:var(--tx-meta);flex-shrink:0">${ts}</span>\
<span style="flex-shrink:0;color:${cls};font-weight:var(--fw-medium)">${escHtml(res)}</span>\
<span>${escHtml(String(a.action || ''))} → ${escHtml(String(a.target || ''))}</span></div>`;
        }).join('');
      }
    }catch(e){ console.error('renderStSecurity audit failed:', e); }
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
    const ok = await ntxConfirm('确定要清除所有本地数据吗？此操作不可恢复。', { title: '清除所有数据', confirmText: '清除', danger: true });
    if(!ok) return;
    if(!isTauri()){ showToast('浏览器模式：仅 Tauri 下可清除'); return; }
    try{
      const sessions = await invoke('neocodex_list_sessions', { project_path: null }).catch(() => []);
      let cleared = 0;
      if(Array.isArray(sessions)){
        for(const s of sessions){
          if(!s || !s.id) continue;
          // neocodex_delete_session 删除会话文件本体; neocodex_clear_session 只清消息
          // 保留元数据 — 不符合"清除所有数据"语义 (D6).
          try{ await invoke('neocodex_delete_session', { session_id: String(s.id) }); cleared++; }catch(_e){}
        }
      }
      // 清空本地存储中的 UI 状态 (主题保留, 会话相关键清除)
      try{
        Object.keys(localStorage).forEach(k => {
          if(k.startsWith('neotrix.') && !k.startsWith('neotrix.theme')) localStorage.removeItem(k);
        });
      }catch(_e){}
      showToast('已清除 ' + cleared + ' 个会话');
    }catch(e){ showToast('清除失败: ' + e); }
  }

  function toggleTheme(){
    const h=document.documentElement;
    const isDark=h.getAttribute('data-theme')==='dark';
    h.setAttribute('data-theme',isDark?'light':'dark');
    try{ localStorage.setItem('neotrix.theme', isDark ? 'light' : 'dark'); }catch(_e){} // persist across restarts
    const lbl=document.getElementById('popThemeLabel');
    if(lbl)lbl.textContent=isDark?'亮色':'暗色';
    showToast(isDark?'🌞 已切换为亮色模式':'🌙 已切换为暗色模式');
  }

  /* ===== App version & updates ===== */
  async function hydrateAppVersion(){
    if(!isTauri()) return;
    try{
      const v = await invoke('neocodex_app_version');
      const el = document.getElementById('popVersion');
      if(el && v) el.textContent = 'v' + String(v);
    }catch(_e){}
  }
  async function checkForUpdate(){
    if(!isTauri()){ showToast('浏览器模式：仅桌面版可检查更新'); return; }
    try{
      const r = await invoke('neocodex_check_update');
      if(r && r.available){
        showToast('发现新版本 v' + (r.latest || '?') + ' — 请从菜单升级');
      }else{
        showToast('已是最新版本 ' + (r ? 'v' + r.current : ''));
      }
    }catch(e){ showToast('检查更新失败: ' + e); }
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

  /* App 内二次确认对话框 (替代原生 confirm). 返回 Promise<boolean>.
     ChatGPT/Claude 对齐: 桌面应用内 confirm, 而非浏览器阻塞对话框. */
  let ntxConfirmRemove = new Set();
  function ntxConfirm(message, { title = '确认操作', confirmText = '确认', danger = false, cancelText = '取消' } = {}){
    const old = document.getElementById('ntxConfirm');
    if(old) old.remove();
    const wrap = document.createElement('div');
    wrap.id = 'ntxConfirm';
    wrap.className = 'ntx-confirm';
    const bar = danger
      ? `<button class="ntx-cf-btn ntx-cf-danger" data-act="confirm">${escHtml(confirmText)}</button><button class="ntx-cf-btn" data-act="cancel">${escHtml(cancelText)}</button>`
      : `<button class="ntx-cf-btn" data-act="cancel">${escHtml(cancelText)}</button><button class="ntx-cf-btn ntx-cf-primary" data-act="confirm">${escHtml(confirmText)}</button>`;
    wrap.innerHTML = `<div class="ntx-cf-box"><div class="ntx-cf-title">${escHtml(title)}</div><div class="ntx-cf-msg">${escHtml(message)}</div><div class="ntx-cf-actions">${bar}</div></div>`;
    document.body.appendChild(wrap);
    requestAnimationFrame(()=>wrap.classList.add('open'));
    const done = (result) => { wrap.classList.remove('open'); wrap.dispatchEvent(new CustomEvent('_ntxDone')); ntxConfirmRemove.add(wrap); setTimeout(()=>{ if(ntxConfirmRemove.has(wrap)){ wrap.remove(); ntxConfirmRemove.delete(wrap); } }, 180); };
    return new Promise(resolve => {
      const settle = (val) => { done(val); resolve(val); };
      wrap.querySelector('[data-act="confirm"]').onclick = () => settle(true);
      wrap.querySelector('[data-act="cancel"]').onclick = () => settle(false);
      wrap.addEventListener('click', e => { if(e.target === wrap) settle(false); });
      const onKey = (e) => {
        if(e.key === 'Escape' && document.getElementById('ntxConfirm') === wrap){ e.preventDefault(); settle(false); }
      };
      window.addEventListener('keydown', onKey);
      wrap.addEventListener('_ntxDone', () => window.removeEventListener('keydown', onKey));
    });
  }
  /* ── Composer token counter (ChatGPT/Claude "123/4096" indicator) ──
     Word-count approximation per wave-13 plan (no tiktoken in frontend). */
  const TOKEN_LIMIT = 4096;
  function updateTokenCount(){
    const el = document.getElementById('tokCount');
    if(!el) return; // silent skip when the indicator is absent
    const inp = document.getElementById('chatInput');
    const text = (inp && inp.value) || '';
    const n = text.split(/\s+/).filter(Boolean).length;
    el.textContent = n + ' / ' + TOKEN_LIMIT;
    el.classList.toggle('over', n > TOKEN_LIMIT);
  }
  function autoResize(el){
    el.style.height='auto';
    el.style.height=Math.min(el.scrollHeight,160)+'px';
    const btn=document.getElementById('sendBtn');
    if(btn && !streamActive) btn.disabled=!el.value.trim();
    try{ QMUpdate(); }catch(_e){}
    try{ saveDraft(); }catch(_e){}
    try{ updateTokenCount(); }catch(_e){}
  }
  function handleKey(e){
    const inp=e.target;
    if(e.key==='Enter'&&!e.shiftKey){
      e.preventDefault();
      if(QKM.items.length && document.querySelector('#qmMenu .qm-item.on')){ QMExec(); return; }
      sendMsg(); return;
    }
    if(e.key==='ArrowDown'||e.key==='ArrowUp'){
      if(QKM.items.length){ e.preventDefault(); QMMove(e.key==='ArrowDown'?1:-1); return; }
      if(recallIdx !== -1 || (!inp.value.trim() && lastUserMsgs.length)){
        e.preventDefault();
        if(e.key === 'ArrowUp' && recallIdx < lastUserMsgs.length - 1) recallIdx++;
        else if(e.key === 'ArrowDown' && recallIdx > 0) recallIdx--;
        else if(e.key === 'ArrowDown' && recallIdx === 0){ recallIdx = -1; inp.value = ''; autoResize(inp); return; }
        const text = recallIdx >= 0 ? lastUserMsgs[lastUserMsgs.length - 1 - recallIdx] : '';
        inp.value = text;
        autoResize(inp);
        inp.setSelectionRange(inp.value.length, inp.value.length);
        return;
      }
    }
    if(e.key==='Escape'){
      if(QKM.items.length){ closeQM(); e.preventDefault(); return; }
      if(recallIdx !== -1){ recallIdx = -1; inp.value = ''; autoResize(inp); e.preventDefault(); return; }
    }
    autoResize(inp);
    QMUpdate();
  }

  /* ════════════════════════════════════════════════
     @-mention 胶囊 + / 命令 弹出菜单 (Cursor/ChatGPT 式)
     输入 @ 或行首 / 触发候选列表; Enter 或点击执行
     ════════════════════════════════════════════════ */
  const MENTION_TARGETS = [
    { kind:'agent', id:'@nt-core',  desc:'引导者 · 总协调' },
    { kind:'agent', id:'@nt-mind',  desc:'进化工匠 · 进化' },
    { kind:'agent', id:'@nt-memory',desc:'知识守护 · 记忆' },
    { kind:'agent', id:'@nt-world', desc:'探索 · 只读检索' },
    { kind:'agent', id:'@nt-act',   desc:'执行 · 多步实现' },
    { kind:'agent', id:'@nt-shield',desc:'影卫 · 审查' },
    { kind:'agent', id:'@nt-io',    desc:'界面使徒 · UI' },
    { kind:'agent', id:'@nt-scout', desc:'调研 · 外部检索' },
    { kind:'tool',  id:'@/diff',    desc:'查看当前工作区 Diff' },
    { kind:'tool',  id:'@/context', desc:'查看上下文使用率' },
  ];
  const SLASH_COMMANDS = [
    { id:'/new',     desc:'新建会话' },
    { id:'/clear',   desc:'清空当前对话' },
    { id:'/compact', desc:'压缩当前会话上下文' },
    { id:'/diff',    desc:'打开 Diff 面板' },
    { id:'/plan',    desc:'切换 Plan 权限模式' },
    { id:'/help',    desc:'快捷键与帮助' },
    { id:'/archive', desc:'归档会话' },
  ];

  let QKM = { mode:null, items:[], sel:0 };

  function closeQM(){ QKM = { mode:null, items:[], sel:0 }; const m=document.getElementById('qmMenu'); if(m){ m.innerHTML=''; m.style.display='none'; } }

  function QMTriggerChar(inp){
    const v=inp.value||''; const pos=inp.selectionStart==null? v.length : inp.selectionStart;
    const before=v.slice(0,pos);
    const at=/@([^@\s/]*)$/.exec(before);
    if(at) return { mode:'@', q:at[1] };
    // slash only when the current line (from line start) begins with '/'
    const lineStart = before.lastIndexOf('\n') + 1;
    const line = before.slice(lineStart);
    const sl=/^\/([a-z]*)$/.exec(line);
    if(sl && pos===before.length) return { mode:'/', q:sl[1]||'' };
    return null;
  }

  function QMUpdate(){
    const inp=document.getElementById('chatInput');
    const tr=inp ? QMTriggerChar(inp) : null;
    if(!tr){ closeQM(); return; }
    const pool = tr.mode==='@' ? MENTION_TARGETS : SLASH_COMMANDS;
    const ql=tr.q.toLowerCase();
    const items=pool.filter(it => !ql || it.id.toLowerCase().includes(ql));
    QKM = { mode:tr.mode, items, sel:0 };
    const m=document.getElementById('qmMenu');
    if(!m) return;
    m.innerHTML = items.length
      ? items.map((it,i)=>`<button type="button" class="qm-item ${i===0?'on':''}" data-i="${i}">`+
          `<span class="qm-ic">${tr.mode==='@' ? (it.kind==='agent'?'◆':'⚙') : '/'}</span>`+
          `<span class="qm-title">${escHtml(it.id)}</span>`+
          `<span class="qm-desc">${escHtml(it.desc)}</span></button>`).join('')
      : '<div class="qm-empty">无匹配项</div>';
    m.style.display = items.length ? 'block' : 'none';
    m.querySelectorAll('.qm-item').forEach(b => b.addEventListener('click', () => { QKM.sel=Number(b.dataset.i); QMExec(); }));
  }

  function QMMove(d){
    if(!QKM.items.length) return;
    QKM.sel = (QKM.sel + d + QKM.items.length) % QKM.items.length;
    const m=document.getElementById('qmMenu');
    m.querySelectorAll('.qm-item').forEach((el,i)=>el.classList.toggle('on', i===QKM.sel));
  }

  function QMExec(){
    const { mode, items, sel } = QKM;
    const it = items[sel];
    if(!it) return;
    closeQM();
    if(mode==='/') runSlashCommand(it.id);
    else insertMention(it.id);
  }

  function insertMention(id){
    const inp=document.getElementById('chatInput');
    if(!inp) return;
    const v=inp.value||''; const pos=inp.selectionStart==null? v.length : inp.selectionStart;
    const before=v.slice(0,pos);
    const at=/@([^@\s/]*)$/.exec(before);
    const base = at ? before.slice(0, before.length - at[0].length) : before;
    const after=v.slice(pos);
    inp.value = base + id + ' ' + after;
    inp.selectionStart = inp.selectionEnd = base.length + id.length + 1;
    inp.focus(); autoResize(inp);
  }

  function runSlashCommand(id){
    switch(id){
      case '/new':     createSession(); break;
      case '/clear':   actions.newChat(); showToast('已清空对话'); break;
      case '/compact': compactSession(); break;
      case '/diff':    openDiff(); break;
      case '/plan':    cycleMode(); break;
      case '/help':    showToast('快捷键: ⌘, 设置 · ⌘N 新建 · ⌘F 检索 · ⌘W 关闭'); break;
      case '/archive': openSessionOps(); break;
      default: showToast('命令: '+id);
    }
  }

  /* 重新生成版本轮播 — 快照 assistant 回复, 支持 较旧/较新 浏览 */
  const regenVersions = new Map(); // key: sessionId+':'+vid -> snapshot html[]
  const regenCursor = new Map();   // key -> current position

  function regenPush(sessionId, vid){
    const el = msgElByVid(vid);
    const mb = el && el.querySelector('.mb');
    if(!mb) return;
    const html = mb.innerHTML;
    if(!html) return;
    const key = sessionId + ':' + vid;
    const arr = regenVersions.get(key) || [];
    if(arr[arr.length-1] !== html) arr.push(html);
    regenVersions.set(key, arr);
    if(!regenCursor.has(key)) regenCursor.set(key, arr.length);
    renderVersionBar(el, sessionId, vid);
  }

  function renderVersionBar(msgEl, sessionId, vid){
    const key = sessionId + ':' + vid;
    const arr = regenVersions.get(key) || [];
    const cur = Math.max(1, Math.min(arr.length, regenCursor.get(key) || arr.length));
    if(arr.length < 2) return;
    let bar = msgEl.querySelector('.ver-bar');
    if(!bar){
      bar = document.createElement('div'); bar.className='ver-bar';
      msgEl.appendChild(bar);
    }
    bar.innerHTML = `<span>回复</span><button class="ver-ctrl" data-a="prev"><span class="caret">◀</span>较旧</button>`+
      `<span class="ver-count">${cur}/${arr.length}</span>`+
      `<button class="ver-ctrl" data-a="next">较新<span class="caret">▶</span></button>`+
      `<button class="ver-reset">重置</button>`;
    const prev=bar.querySelector('[data-a="prev"]'), next=bar.querySelector('[data-a="next"]');
    prev.disabled = cur<=1; next.disabled = cur>=arr.length;
    prev.onclick=()=>verNav(sessionId, vid, -1);
    next.onclick=()=>verNav(sessionId, vid, 1);
    bar.querySelector('.ver-reset').onclick=()=>verReset(sessionId, vid);
  }

  function verNav(sessionId, vid, delta){
    const key = sessionId + ':' + vid;
    const arr = regenVersions.get(key) || [];
    let cur = regenCursor.get(key) || arr.length;
    const next = Math.max(1, Math.min(arr.length, cur + delta));
    if(next === cur) return;
    regenCursor.set(key, next);
    const el = msgElByVid(vid); const mb = el && el.querySelector('.mb');
    if(mb && arr[next-1] !== undefined) mb.innerHTML = arr[next-1];
    renderVersionBar(el, sessionId, vid);
  }

  function verReset(sessionId, vid){
    const key = sessionId + ':' + vid;
    regenCursor.delete(key);
    const el = msgElByVid(vid);
    if(el) renderVersionBar(el, sessionId, vid);
    showToast('已显示最新回复');
  }
function escHtml(str){
    if(!str)return'';
    return String(str).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#39;');
  }

  /* ── Window controls: native macOS traffic lights (Overlay titlebar) handle
     close/minimize/maximize. Tauri overlay only needs the drag region. ── */

  // expose for inline onclick
  g.toggleSidebar = toggleSidebar;


