import { useState, useCallback, useEffect, useRef } from 'react';
import { useAppCtx } from '../App';
import { startSession, stopSession, pauseSession } from '../core/session-manager';
import { socialEvents } from '../core/social-events';
import { socialIdentity } from '../core/social-identity';
import ConsciousnessCycle from './ConsciousnessCycle';
import AbsorptionPipeline from './AbsorptionPipeline';
import { pluginRegistry } from '../core/plugin-system';

const MODELS = [
  { id: 'neotrix-default', label: 'neotrix-default' },
  { id: 'gpt-4o', label: 'GPT-4o' },
  { id: 'claude-sonnet-4', label: 'Claude Sonnet 4' },
  { id: 'deepseek-v4', label: 'DeepSeek V4' },
  { id: 'o3-mini', label: 'o3-mini' },
];

const DEPTHS = [
  { id: 'quick', label: 'Quick' },
  { id: 'deep', label: 'Deep' },
  { id: 'max', label: 'Max' },
];

const ACTIVITIES: Record<string, string[]> = {
  GATHER:  ['VSA 4096-bit sensory buffer init', 'VsaTag Self|World boundary check', 'HyperCube top-3 context retrieval', 'EntityInject fact → MemoryGraph'],
  REFLECT: ['MetaAccuracy prediction-vs-outcome', 'ECE drift computation', 'Calibration meta-d\'=1.42', 'GoalDriftIdx semantic KL scan'],
  REASON:  ['E8 64-state kernel dispatch', 'VsaReasoner bind/bundle dispatch', 'PatternMatcher analogical search', 'FreeEnergy curiosity=0.74', 'HebbianRecall associative fusion'],
  PLAN:    ['Action sequence generation', 'MCTS tree expansion', 'Counterfactual simulation', 'Resource allocation'],
  ACT:     ['Tool selection & invocation', 'Integration bus signal dispatch', 'Subsystem modulation'],
  RECORD:  ['Experience tree node commit', 'MemoryLattice CTE consolidation', 'Q-value TD(0) update'],
  METRIC:  ['IIT Φ 8-engine MIP sampling', 'P2P Banach consensus check', 'MetaAccuracy sliding window'],
  EVOLVE:  ['SEAL GEPA Pareto update', 'Mutation proposal generation', 'Pareto front diversity check'],
  SLEEP:   ['SWS critical path extraction', 'REM hierarchical compression', 'Hebbian semantic spread'],
  META:    ['Meta-epoch parameter evolution', 'Self-model snapshot @50', 'Anti-spiral stagnation check'],
  VETO:    ['VolitionEngine self-check', 'UnifiedWill governance hash', 'Free won\'t gate'],
  AWAKEN:  ['Awakening integration test', 'Subsystem health verification', 'Consciousness continuity'],
};

const STEP_ACTIVITY_TIME = 400;

export default function ReasoningView() {
  const { setIsGenerating, activeSessionId, viewMode } = useAppCtx();
  const [model, setModel] = useState('neotrix-default');
  const [depth, setDepth] = useState('deep');
  const [searchEnabled, setSearchEnabled] = useState(false);
  const [thinkingEnabled, setThinkingEnabled] = useState(true);
  const [showModelPicker, setShowModelPicker] = useState(false);
  const [isGenerating, setIsGeneratingLocal] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [copied, setCopied] = useState(false);
  const [inputText, setInputText] = useState('');
  const [pluginCount, setPluginCount] = useState(0);
  const [logs, setLogs] = useState<Array<{ step: string; label: string; ts: number }>>([]);
  const [socialMoments, setSocialMoments] = useState<Array<{ from: string; text: string; ts: number }>>([]);
  const [showSocial, setShowSocial] = useState(true);
  const [currentStep, setCurrentStep] = useState(0);
  const [cScore, setCScore] = useState(0.5);
  const [coherence, setCoherence] = useState(0.5);
  const [reflexivity, setReflexivity] = useState(0.3);
  const [cycleEmotion, setCycleEmotion] = useState('neutral');
  const logEndRef = useRef<HTMLDivElement>(null);
  const timerRef = useRef<ReturnType<typeof setInterval>>();
  const metricTimerRef = useRef<ReturnType<typeof setInterval>>();

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  useEffect(() => {
    setPluginCount(pluginRegistry.all().length);
  }, []);

  // Fetch social moments periodically
  useEffect(() => {
    const update = () => {
      const moments = socialEvents.query({ filter: 'moments', limit: 6 });
      setSocialMoments(moments.map(m => ({
        from: m.pubkey === socialIdentity.pubkey
          ? (socialIdentity.profile?.name ?? 'You')
          : (socialIdentity.getContact(m.pubkey)?.alias ?? m.pubkey.slice(0, 12)),
        text: m.content,
        ts: m.created_at,
      })));
    };
    update();
    const iv = setInterval(update, 5000);
    return () => clearInterval(iv);
  }, []);

  // simulation state (refs so they survive pause/resume without re-render)
  const stepIdxRef = useRef(0);
  const activityIdxRef = useRef(0);
  const steps = ['GATHER', 'REFLECT', 'REASON', 'PLAN', 'ACT', 'RECORD', 'METRIC', 'EVOLVE', 'SLEEP', 'META', 'VETO', 'AWAKEN'];
  const emotions = ['curious', 'reflective', 'neutral', 'neutral', 'neutral', 'happy', 'reflective', 'neutral', 'reflective', 'neutral', 'neutral', 'happy'];

  function runSimulationTick() {
    if (stepIdxRef.current >= steps.length) {
      clearInterval(timerRef.current);
      clearInterval(metricTimerRef.current);
      setIsGeneratingLocal(false);
      setIsGenerating(false);
      stopSession(activeSessionId);
      return;
    }
    const step = steps[stepIdxRef.current];
    const activities = ACTIVITIES[step];
    const label = activities[activityIdxRef.current % activities.length];
    setLogs(prev => [...prev, { step, label, ts: Date.now() }]);
    setCurrentStep(stepIdxRef.current);
    setCycleEmotion(emotions[stepIdxRef.current] || 'neutral');

    activityIdxRef.current++;
    if (activityIdxRef.current >= activities.length) {
      activityIdxRef.current = 0;
      stepIdxRef.current++;
    }
  }

  function startSimulation() {
    stepIdxRef.current = 0;
    activityIdxRef.current = 0;
    setLogs([]);
    setCurrentStep(0);
    setCScore(0.3);
    setCoherence(0.4);
    setReflexivity(0.2);
    setCycleEmotion('curious');

    timerRef.current = setInterval(runSimulationTick, STEP_ACTIVITY_TIME);
    metricTimerRef.current = setInterval(() => {
      setCScore(prev => Math.min(0.95, Math.max(0.15, prev + (Math.random() - 0.5) * 0.08)));
      setCoherence(prev => Math.min(0.9, Math.max(0.2, prev + (Math.random() - 0.5) * 0.06)));
      setReflexivity(prev => Math.min(0.9, Math.max(0.1, prev + (Math.random() - 0.5) * 0.05)));
    }, 800);
  }

  function startGeneration() {
    if (isPaused) {
      setIsPaused(false);
      startSession(activeSessionId);
      timerRef.current = setInterval(runSimulationTick, STEP_ACTIVITY_TIME);
      metricTimerRef.current = setInterval(() => {
        setCScore(prev => Math.min(0.95, Math.max(0.15, prev + (Math.random() - 0.5) * 0.08)));
        setCoherence(prev => Math.min(0.9, Math.max(0.2, prev + (Math.random() - 0.5) * 0.06)));
        setReflexivity(prev => Math.min(0.9, Math.max(0.1, prev + (Math.random() - 0.5) * 0.05)));
      }, 800);
      return;
    }
    setIsGeneratingLocal(true);
    setIsGenerating(true);
    setIsPaused(false);
    startSession(activeSessionId);
    startSimulation();
  }

  function pauseGeneration() {
    clearInterval(timerRef.current);
    clearInterval(metricTimerRef.current);
    pauseSession(activeSessionId);
    setIsPaused(true);
  }

  function stopGeneration() {
    clearInterval(timerRef.current);
    clearInterval(metricTimerRef.current);
    stopSession(activeSessionId);
    setIsGeneratingLocal(false);
    setIsGenerating(false);
    setIsPaused(false);
  }

  const handleCopy = useCallback(() => {
    const text = logs.map(l => `  ${l.step.padEnd(10)}${l.label}`).join('\n');
    navigator.clipboard.writeText(text || '> NeoTrix consciousness cycle output');
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }, [logs]);

  function handleUserSubmit() {
    const text = inputText.trim();
    if (!text || isGenerating) return;
    setLogs(prev => [...prev, { step: 'USER', label: text, ts: Date.now() }]);
    setInputText('');
    startGeneration();
  }

  function formatTime(ts: number) {
    const d = new Date(ts);
    return `${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`;
  }

  // viewMode filtering
  const filteredLogs = logs.length > 0
    ? viewMode === 'summary'
      ? aggregateSummary(logs)
      : logs
    : [];

  function aggregateSummary(entries: typeof logs) {
    const grouped: Record<string, { step: string; count: number; lastTs: number }> = {};
    for (const l of entries) {
      if (!grouped[l.step]) grouped[l.step] = { step: l.step, count: 0, lastTs: l.ts };
      grouped[l.step].count++;
      grouped[l.step].lastTs = l.ts;
    }
    return Object.values(grouped).map(g => ({
      step: g.step,
      label: `${g.step} (${g.count} ops)`,
      ts: g.lastTs,
    }));
  }

  return (
    <div className={`reasoning-view reasoning-view-${viewMode}`}>
      <div className="reasoning-output">
        <div className="reasoning-output-inner">
          {logs.length > 0 ? (
            <>
              <div className="output-header">
                <span className="output-count">{filteredLogs.length} {viewMode === 'summary' ? 'phases' : 'activities'}</span>
                <div className="output-actions">
                  <button className="output-btn" onClick={handleCopy} title="Copy log">
                    {copied ? (
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="20 6 9 17 4 12" /></svg>
                    ) : (
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
                    )}
                  </button>
                </div>
              </div>
              <div className="activity-log">
              {filteredLogs.map((l, i) => (
                <div key={i} className={`activity-entry activity-${l.step.toLowerCase()}`}>
                  <span className="activity-time">{formatTime(l.ts)}</span>
                  <span className="activity-step">{l.step}</span>
                  <span className="activity-label">{l.label}</span>
                </div>
              ))}
              {isGenerating && (
                <div className="activity-entry">
                  <span className="activity-time">{formatTime(Date.now())}</span>
                  <span className="activity-step">...</span>
                  <span className="activity-label thinking-dots">
                    <span>.</span><span>.</span><span>.</span>
                  </span>
                </div>
              )}
              <div ref={logEndRef} />
            </div>

            {/* Social stream below activity log (when idle) */}
            {!isGenerating && showSocial && socialMoments.length > 0 && (
              <div className="social-stream">
                <div className="social-stream-header">
                  <span className="social-stream-title">Recent Moments</span>
                  <button className="social-stream-toggle" onClick={() => setShowSocial(false)}>
                    <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                    </svg>
                  </button>
                </div>
                {socialMoments.map((m, i) => (
                  <div key={i} className="activity-entry activity-social">
                    <span className="activity-time">
                      {new Date(m.ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </span>
                    <span className="activity-step social-from">{m.from}</span>
                    <span className="activity-label social-text">{m.text}</span>
                  </div>
                ))}
              </div>
            )}
            </>
          ) : (
            <div className="activity-placeholder">
              <div className="placeholder-text">{`> NeoTrix v2026 — Consciousness Cycle awaiting input
  Press Enter or click Send to begin processing.`}</div>

              {/* Social stream injected into idle view */}
              {showSocial && socialMoments.length > 0 && (
                <div className="social-stream">
                  <div className="social-stream-header">
                    <span className="social-stream-title">Recent Moments</span>
                    <button className="social-stream-toggle" onClick={() => setShowSocial(false)}>
                      <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                      </svg>
                    </button>
                  </div>
                  {socialMoments.map((m, i) => (
                    <div key={i} className={`activity-entry activity-social`}>
                      <span className="activity-time">
                        {new Date(m.ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                      </span>
                      <span className="activity-step social-from">{m.from}</span>
                      <span className="activity-label social-text">{m.text}</span>
                    </div>
                  ))}
                </div>
              )}
              {!showSocial && (
                <button className="social-stream-reveal" onClick={() => setShowSocial(true)}>
                  Show recent moments
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      {isGenerating && (
        <ConsciousnessCycle
          activeStep={currentStep}
          isRunning={isGenerating}
          cScore={cScore}
          coherence={coherence}
          reflexivity={reflexivity}
          emotion={cycleEmotion}
        />
      )}

      <div className="input-area">
        <AbsorptionPipeline onAbsorbResult={(result) => {
          setLogs(prev => [...prev, { step: 'ABSORB', label: `Absorbed: ${result.slice(0, 60)}...`, ts: Date.now() }]);
        }} />

        <div className="input-glass">
          <div className="input-toolbar">
            <div style={{ position: 'relative' }}>
              <button className="toolbar-btn" onClick={() => setShowModelPicker(v => !v)}>
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="12" cy="12" r="3" />
                  <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
                </svg>
                <span className="model-name-text">{MODELS.find(m => m.id === model)!.label}</span>
              </button>
              {showModelPicker && (
                <div className="model-dropdown">
                  {MODELS.map(m => (
                    <button
                      key={m.id}
                      className={`model-option ${m.id === model ? 'selected' : ''}`}
                      onClick={() => { setModel(m.id); setShowModelPicker(false); }}
                    >
                      {m.label}
                    </button>
                  ))}
                </div>
              )}
            </div>

            {DEPTHS.map(d => (
              <button
                key={d.id}
                className={`toolbar-btn depth-btn ${depth === d.id ? 'active' : ''}`}
                onClick={() => setDepth(d.id)}
              >
                {d.label}
              </button>
            ))}

            <button className="toolbar-btn" title={`${pluginCount} plugins active`} style={{ opacity: 0.5, fontSize: 8 }}>
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/>
              </svg>
              {pluginCount}
            </button>

            <button
              className={`toolbar-btn ${thinkingEnabled ? 'active' : ''}`}
              onClick={() => setThinkingEnabled(v => !v)}
              title={thinkingEnabled ? 'Disable thinking' : 'Enable thinking'}
            >
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M9.5 2A2.5 2.5 0 0 1 12 4.5v.5" />
                <path d="M12 2a7 7 0 0 1 7 7v1" />
                <path d="M19 11v3a7 7 0 0 1-7 7" />
                <line x1="5" y1="12" x2="5" y2="13" />
                <line x1="9" y1="12" x2="9" y2="13" />
              </svg>
            </button>

            <button
              className={`toolbar-btn ${searchEnabled ? 'active' : ''}`}
              onClick={() => setSearchEnabled(v => !v)}
              title={searchEnabled ? 'Disable search' : 'Enable search'}
            >
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
              </svg>
              <span style={{ fontSize: 10 }}>Search</span>
            </button>
          </div>

          <div className="input-row">
            <span className="input-prompt">{'>'}</span>
            <input
              type="text"
              className="input-field"
              placeholder={isGenerating ? 'Generating...' : ''}
              value={inputText}
              onChange={e => setInputText(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter' && !e.metaKey) { e.preventDefault(); handleUserSubmit(); } }}
              autoFocus
            />
            {isGenerating ? (
              <>
                {isPaused ? (
                  <button className="send-btn" onClick={startGeneration} title="Resume">
                    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="6 3 20 12 6 21 6 3" />
                    </svg>
                  </button>
                ) : (
                  <button className="pause-btn" onClick={pauseGeneration} title="Pause">
                    <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor">
                      <rect x="6" y="4" width="4" height="16" rx="1" />
                      <rect x="14" y="4" width="4" height="16" rx="1" />
                    </svg>
                  </button>
                )}
                <button className="stop-btn" onClick={stopGeneration} title="Stop">
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor">
                    <rect x="6" y="6" width="12" height="12" rx="2" />
                  </svg>
                </button>
              </>
            ) : (
              <button className="send-btn" onClick={startGeneration} title="Send">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="22" y1="2" x2="11" y2="13" />
                  <polygon points="22 2 15 22 11 13 2 9 22 2" />
                </svg>
              </button>
            )}
          </div>
        </div>

        <div className="input-meta">
          <span className="input-shortcut">Enter 发送</span>
          <span className="input-shortcut-sep">·</span>
          <span className="input-shortcut">{'⌘'}Enter 换行</span>
          <span className="input-shortcut-sep">·</span>
          <span className="input-shortcut">{'⌘'}K 命令</span>
          <span className="input-shortcut-sep">·</span>
          <span className="input-shortcut">{'⌘'}B 面板</span>
          <span className="input-shortcut-sep">·</span>
          <span className="input-shortcut">{'⌘'}; 侧边对话</span>
        </div>
      </div>
    </div>
  );
}
