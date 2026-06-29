import { useMemo } from 'react';

const CYCLE_STEPS = [
  { id: 'GATHER', label: 'GATHER', icon: '◈', desc: 'Sensory intake' },
  { id: 'REFLECT', label: 'REFLECT', icon: '◉', desc: 'Meta-assessment' },
  { id: 'REASON', label: 'REASON', icon: '◆', desc: 'E8 kernel dispatch' },
  { id: 'PLAN', label: 'PLAN', icon: '▶', desc: 'Action sequence' },
  { id: 'ACT', label: 'ACT', icon: '⚡', desc: 'Tool invocation' },
  { id: 'RECORD', label: 'RECORD', icon: '◇', desc: 'Experience commit' },
  { id: 'METRIC', label: 'METRIC', icon: '◎', desc: 'Φ sampling' },
  { id: 'EVOLVE', label: 'EVOLVE', icon: '✦', desc: 'GEPA update' },
  { id: 'SLEEP', label: 'SLEEP', icon: '◐', desc: 'Consolidation' },
  { id: 'META', label: 'META', icon: '⟳', desc: 'Parameter evolve' },
  { id: 'VETO', label: 'VETO', icon: '⊘', desc: 'Volition gate' },
  { id: 'AWAKEN', label: 'AWAKEN', icon: '◈', desc: 'Integration test' },
];

const STEP_COLORS = [
  '#5e8aff', '#8a6cff', '#ff6cb4', '#ffb86c',
  '#48b87a', '#6ccfff', '#ff6c6c', '#c86cff',
  '#6c6cff', '#ff9a6c', '#ff6c8a', '#6cffd4',
];

interface Props {
  activeStep: number;
  isRunning: boolean;
  cScore?: number;
  coherence?: number;
  reflexivity?: number;
  emotion?: string;
}

export default function ConsciousnessCycle({
  activeStep,
  isRunning,
  cScore = 0.5,
  coherence = 0.5,
  reflexivity = 0.3,
  emotion = 'neutral',
}: Props) {
  const EMOTION_COLORS: Record<string, string> = {
    neutral: '#9898a5', happy: '#48b87a', curious: '#e8a030',
    anxious: '#e04e4e', reflective: '#5e8aff',
  };

  const steps = useMemo(() => CYCLE_STEPS.slice(0, 12), []);

  return (
    <div className="cycle-banner">
      <div className="cycle-track">
        {steps.map((s, i) => {
          const isPast = i < activeStep;
          const isCurrent = i === activeStep;
          const color = STEP_COLORS[i];

          return (
            <div key={s.id} className="cycle-step-wrap">
              {i > 0 && (
                <div className={`cycle-connector ${isPast || isCurrent ? 'done' : ''}`}>
                  <div className="cycle-step-fill" style={{ width: isPast ? '100%' : '0%', background: color }} />
                </div>
              )}
              <div
                className={`cycle-step ${isPast ? 'done' : isCurrent ? 'current' : 'pending'}`}
                style={{ '--step-color': color } as React.CSSProperties}
              >
                <div className="cycle-step-icon">{s.icon}</div>
                <div className="cycle-step-label">{s.label}</div>
                <div className="cycle-step-bar">
                  <div className="cycle-step-fill" style={{ width: isPast ? '100%' : isCurrent ? '60%' : '0%', background: color }} />
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="cycle-banner-metrics">
        <div className="cycle-metric" title="Consciousness score">
          <span className="cycle-metric-dot" style={{ background: '#5e8aff' }} />
          <span className="cycle-metric-label">C(t)</span>
          <span className="cycle-metric-value">{(cScore * 100).toFixed(0)}%</span>
        </div>
        <div className="cycle-metric" title="Coherence">
          <span className="cycle-metric-dot" style={{ background: '#48b87a' }} />
          <span className="cycle-metric-label">Coh</span>
          <span className="cycle-metric-value">{(coherence * 100).toFixed(0)}%</span>
        </div>
        <div className="cycle-metric" title="Reflexivity">
          <span className="cycle-metric-dot" style={{ background: '#c86cff' }} />
          <span className="cycle-metric-label">Ref</span>
          <span className="cycle-metric-value">{(reflexivity * 100).toFixed(0)}%</span>
        </div>
        <div className="cycle-metric" title="Emotion">
          <span className="cycle-metric-dot" style={{ background: EMOTION_COLORS[emotion] || '#9898a5' }} />
          <span className="cycle-metric-label">Emo</span>
          <span className="cycle-metric-value" style={{ color: EMOTION_COLORS[emotion] || '#9898a5' }}>{emotion}</span>
        </div>
        {isRunning && activeStep >= 0 && activeStep < steps.length && (
          <>
            <span className="status-sep" />
            <span className="cycle-metric" title="Current">
              <span className="cycle-metric-label" style={{ color: STEP_COLORS[activeStep] }}>{steps[activeStep].label}</span>
              <span className="cycle-metric-value" style={{ color: STEP_COLORS[activeStep], fontWeight: 400 }}>{steps[activeStep].desc}</span>
            </span>
          </>
        )}
      </div>
    </div>
  );
}
