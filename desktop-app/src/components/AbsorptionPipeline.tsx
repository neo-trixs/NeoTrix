import { useState, useCallback, useRef, useEffect } from 'react';
import { catalog, AbsorptionState } from '../core/absorption-engine';

const ABSORPTION_PHASES = [
  { id: 'scout', label: 'Scout', icon: '◈', desc: 'Structural scan — files, imports, dependencies' },
  { id: 'extract', label: 'Extract', icon: '◉', desc: 'Semantic lifting — behavioral signatures, APIs' },
  { id: 'deep-read', label: 'Deep Read', icon: '◆', desc: 'Full-text consumption of high-value targets' },
  { id: 'synthesize', label: 'Synthesize', icon: '✦', desc: 'Cross-ref, pattern extraction, defect ID' },
];

const PHASE_COLORS = ['#5e8aff', '#c86cff', '#ff6cb4', '#6cffd4'];

const STATE_LABELS: Record<AbsorptionState, string> = {
  scanned: '🔍 Scanned',
  deconstructed: '🔧 Deconstructed',
  mapped: '🔗 VSA Mapped',
  fused: '✨ Fused',
  implemented: '✅ Implemented',
};

const STATE_ORDER: AbsorptionState[] = ['scanned', 'deconstructed', 'mapped', 'fused', 'implemented'];

interface Props {
  onAbsorbResult?: (result: string) => void;
}

export default function AbsorptionPipeline({ onAbsorbResult }: Props) {
  const [activePhase, setActivePhase] = useState(0);
  const [isRunning, setIsRunning] = useState(false);
  const [phaseResults, setPhaseResults] = useState<Array<{ phase: string; summary: string; quality: number }>>([]);
  const [currentStepDetail, setCurrentStepDetail] = useState('');
  const [showPanel, setShowPanel] = useState(false);
  const timerRef = useRef<ReturnType<typeof setInterval>>();

  const patterns = catalog.all();
  const fusedFeatures = catalog.allFused();

  const runPipeline = useCallback(() => {
    setIsRunning(true);
    setActivePhase(0);
    setPhaseResults([]);
    let phase = 0;

    timerRef.current = setInterval(() => {
      if (phase >= ABSORPTION_PHASES.length) {
        clearInterval(timerRef.current);
        setIsRunning(false);
        const total = patterns.length;
        const implemented = patterns.filter(p => p.state === 'implemented').length;
        const fused = patterns.filter(p => p.state === 'fused').length;
        const summary = `${total} patterns · ${fused} fused · ${implemented} implemented · ${fusedFeatures.length} features`;
        onAbsorbResult?.(summary);
        return;
      }

      const p = ABSORPTION_PHASES[phase];
      setActivePhase(phase);
      setCurrentStepDetail(p.desc);

      const qualities = [0.6, 0.75, 0.85, 0.9];
      const summaries = [
        `Scout: ${patterns.length} patterns from ${new Set(patterns.map(x => x.source.category)).size} categories`,
        `Extract: ${patterns.filter(x => x.state !== 'scanned').length} patterns deconstructed`,
        `Deep-read: ${patterns.filter(x => x.state === 'mapped' || x.state === 'fused' || x.state === 'implemented').length} VSA-mapped patterns`,
        `Synthesize: ${fusedFeatures.length} fused features · ${fusedFeatures.filter(f => f.active).length} active`,
      ];

      setPhaseResults(prev => [...prev, { phase: p.id, summary: summaries[phase], quality: qualities[phase] }]);
      phase++;
    }, 1000);

    return () => clearInterval(timerRef.current);
  }, [onAbsorbResult, patterns.length, fusedFeatures.length]);

  useEffect(() => {
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, []);

  const countByState = (state: AbsorptionState) => patterns.filter(p => p.state === state).length;

  // Collapsed idle state
  if (!isRunning && phaseResults.length === 0) {
    return (
      <div className="absorb-container" style={{ marginBottom: 8 }}>
        <div className="absorb-mini">
          <button className="absorb-mini-btn" onClick={runPipeline} title="Absorption pipeline">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5"/>
              <line x1="12" y1="22" x2="12" y2="15.5"/>
              <polyline points="22 8.5 12 15.5 2 8.5"/>
            </svg>
          </button>
        </div>
      </div>
    );
  }

  // Collapsed done state
  if (!isRunning && phaseResults.length > 0) {
    return (
      <div className="absorb-container" style={{ marginBottom: 8 }}>
        <div className="absorb-mini">
          <button className="absorb-mini-btn" onClick={runPipeline} title="Re-run absorption">
            ↻
          </button>
          <span className="absorb-done" onClick={() => setShowPanel(v => !v)} style={{ cursor: 'pointer' }}>
            {patterns.length} patterns · {fusedFeatures.length} features {showPanel ? '▾' : '▸'}
          </span>
        </div>

        {showPanel && (
          <div className="abs-pipeline" style={{ marginTop: 6 }}>
            {/* Feature cards */}
            <div className="abs-cards" style={{ marginBottom: 8 }}>
              {fusedFeatures.map(f => (
                <div key={f.id} className="abs-card" style={{ borderLeftColor: f.active ? '#48b87a' : '#666', borderLeftWidth: 2 }}>
                  <div className="abs-card-header">
                    <span className="abs-card-type" style={{ color: f.active ? '#48b87a' : '#666' }}>
                      {f.active ? 'ACTIVE' : 'INACTIVE'}
                    </span>
                    <span style={{ fontSize: 10, fontWeight: 600, color: 'var(--text-primary)', flex: 1 }}>
                      {f.name}
                    </span>
                  </div>
                  <div className="abs-card-insight" style={{ fontSize: 9 }}>
                    {f.description}
                  </div>
                  <div style={{ fontSize: 8, color: 'var(--text-tertiary)', marginTop: 2 }}>
                    VSA: {f.vsaPrimitives.join(', ')} · {f.sourcePatterns.length} sources
                  </div>
                </div>
              ))}
            </div>

            {/* State distribution */}
            <div className="abs-flow">
              {STATE_ORDER.map(s => (
                <div key={s} className="abs-flow-item">
                  <span className="abs-flow-dot" style={{ background: s === 'implemented' ? '#48b87a' : s === 'fused' ? '#c86cff' : '#5e8aff' }} />
                  <span className="abs-flow-label">{STATE_LABELS[s]}</span>
                  <span style={{ marginLeft: 'auto', fontSize: 9, color: 'var(--text-tertiary)' }}>
                    {countByState(s)}
                  </span>
                </div>
              ))}
            </div>

            {/* Pattern list */}
            <div style={{ fontSize: 9, color: 'var(--text-tertiary)', marginTop: 4 }}>
              Sources: {patterns.map(p => p.source.name).join(', ')}
            </div>
          </div>
        )}
      </div>
    );
  }

  // Expanded pipeline view
  return (
    <div className="absorb-container" style={{ marginBottom: 8 }}>
      <div className="abs-pipeline">
        <div className="abs-phases">
          {ABSORPTION_PHASES.map((p, i) => {
            const isPast = i < activePhase;
            const isCurrent = i === activePhase;
            return (
              <div key={p.id}
                className={`abs-phase ${isPast ? 'done' : isCurrent ? 'running' : ''}`}
                style={{ borderColor: isPast || isCurrent ? PHASE_COLORS[i] : undefined }}
              >
                {p.label}
              </div>
            );
          })}
        </div>
        <div className="abs-flow">
          <div className="abs-flow-item">
            <span className="abs-flow-dot" />
            <span className="abs-flow-label">{currentStepDetail}</span>
          </div>
        </div>
        <div className="abs-cards">
          {phaseResults.map((r, i) => (
            <div key={i} className="abs-card" style={{ borderLeftColor: PHASE_COLORS[i], borderLeftWidth: 2 }}>
              {r.summary}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
