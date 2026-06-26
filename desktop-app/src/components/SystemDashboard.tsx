import { useState, useEffect } from 'react';
import ConsciousnessCycle from './ConsciousnessCycle';
import RepoUnderstandingPanel from './RepoUnderstandingPanel';

interface Props { compact?: boolean }

export default function SystemDashboard({ compact }: Props) {
  const [sysStep, setSysStep] = useState(0);
  const sysRunning = true;
  const [liveMetrics, setLiveMetrics] = useState({
    phi: 3.42, freeEnergy: 0.74, metaD: 1.42,
    hebbian: 87, p2p: 0.92, vsaBind: 4.2,
    cScore: 0.68, coherence: 0.72, reflexivity: 0.55,
  });
  const emotions = ['neutral', 'curious', 'reflective', 'happy', 'neutral', 'reflective', 'curious', 'neutral', 'reflective', 'neutral', 'neutral', 'happy'];

  useEffect(() => {
    const ti = setInterval(() => {
      setSysStep(prev => (prev + 1) % 12);
      setLiveMetrics(prev => ({
        phi: +(prev.phi + (Math.random() - 0.5) * 0.4).toFixed(2),
        freeEnergy: +(prev.freeEnergy + (Math.random() - 0.5) * 0.08).toFixed(2),
        metaD: +(prev.metaD + (Math.random() - 0.5) * 0.1).toFixed(2),
        hebbian: Math.min(99, Math.max(60, prev.hebbian + (Math.random() - 0.5) * 4)),
        p2p: +(prev.p2p + (Math.random() - 0.5) * 0.04).toFixed(2),
        vsaBind: +(prev.vsaBind + (Math.random() - 0.5) * 0.6).toFixed(1),
        cScore: +(prev.cScore + (Math.random() - 0.5) * 0.06).toFixed(2),
        coherence: +(prev.coherence + (Math.random() - 0.5) * 0.05).toFixed(2),
        reflexivity: +(prev.reflexivity + (Math.random() - 0.5) * 0.04).toFixed(2),
      }));
    }, 1400);
    return () => clearInterval(ti);
  }, []);

  return (
    <div className={`${compact ? 'sidebar-placeholder' : 'system-view'}`}>
      {!compact && (
        <ConsciousnessCycle
          activeStep={sysStep}
          isRunning={sysRunning}
          cScore={liveMetrics.cScore}
          coherence={liveMetrics.coherence}
          reflexivity={liveMetrics.reflexivity}
          emotion={emotions[sysStep]}
        />
      )}

      <div className="dash-section">
        {compact ? (
          <div className="sidebar-label">System Metrics</div>
        ) : (
          <h4>Three-Loop Architecture</h4>
        )}
        {!compact && (
          <div className="loop-cards">
            {[
              { name: 'Small Loop', color: '#6c8cff', steps: 'GATHER → REFLECT → REASON', rate: 'tick-level', status: 'active 3/5 feedbacks' },
              { name: 'Big Loop', color: '#ff6cb4', steps: 'PLAN → ACT → RECORD → METRIC', rate: 'cycle-level', status: '2/4 feedbacks' },
              { name: 'Meta Loop', color: '#c86cff', steps: 'EVOLVE → SLEEP → META → VETO → AWAKEN', rate: 'epoch-level', status: '4/5 feedbacks' },
            ].map(l => (
              <div key={l.name} className="loop-card" style={{ borderLeftColor: l.color }}>
                <div className="loop-card-header">
                  <span className="loop-card-name">{l.name}</span>
                  <span className="loop-card-status">{l.status}</span>
                </div>
                <div className="loop-card-steps">{l.steps}</div>
                <div className="loop-card-rate">{l.rate}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="dash-section">
        {!compact && <h4>Consciousness Metrics</h4>}
        <div className="metrics-grid" style={compact ? { gridTemplateColumns: '1fr 1fr', gap: 3 } : undefined}>
          {[
            { label: 'IIT Φ', value: liveMetrics.phi.toFixed(2), sub: '8-engine parallel' },
            { label: 'FreeEnergy', value: liveMetrics.freeEnergy.toFixed(2), sub: 'curiosity drive' },
            { label: "Meta-d'", value: liveMetrics.metaD.toFixed(2), sub: 'calibration' },
            { label: 'Hebbian', value: `${liveMetrics.hebbian}%`, sub: 'distillation' },
            { label: 'P2P Consensus', value: liveMetrics.p2p.toFixed(2), sub: 'Banach λ=0.3' },
            { label: 'VSA Bind', value: `${liveMetrics.vsaBind.toFixed(1)}K/s`, sub: 'MAP default' },
          ].map(m => (
            <div key={m.label} className="metric-card" style={compact ? { padding: 5 } : undefined}>
              <span className="metric-label">{m.label}</span>
              <span className="metric-value" style={compact ? { fontSize: 13 } : undefined}>{m.value}</span>
              {!compact && <span className="metric-sub">{m.sub}</span>}
            </div>
          ))}
        </div>
      </div>

      {!compact && (
        <div className="dash-section">
          <h4>Governance</h4>
          <div className="gov-list">
            {[
              { key: '#A4F2', action: 'SEAL mutate proposal', status: 'committed', verifications: '3/3' },
              { key: '#A4F1', action: 'GEPA Pareto update', status: 'committed', verifications: '3/3' },
              { key: '#A4F0', action: 'ModulationCommand:ExploreMore', status: 'pending', verifications: '1/3' },
            ].map(g => (
              <div key={g.key} className="gov-item">
                <span className="gov-key">{g.key}</span>
                <span className="gov-action">{g.action}</span>
                <span className={`gov-status status-${g.status === 'committed' ? 'running' : 'paused'}`}>{g.status}</span>
                <span className="gov-ver">{g.verifications}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {!compact && <RepoUnderstandingPanel />}
    </div>
  );
}
