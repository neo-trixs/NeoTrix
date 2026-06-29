import { useMemo } from 'react';
import { getSession, generatePlan, approvePlanStep, rejectPlanStep } from '../core/session-manager';

export default function PlanMode({ sessionId, task, onClose }: { sessionId: string; task?: string; onClose: () => void }) {
  const plan = useMemo(() => {
    const s = getSession(sessionId);
    if (!s?.plan && task) generatePlan(sessionId, task);
    return s?.plan || [];
  }, [sessionId, task]);

  if (plan.length === 0) {
    return (
      <div className="plan-mode">
        <div className="plan-mode-header">
          <h3>Plan Mode</h3>
          <button className="plan-close" onClick={onClose}>
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div className="plan-empty">No active plan. Submit a task to generate one.</div>
      </div>
    );
  }

  const stats = {
    total: plan.length,
    approved: plan.filter(p => p.status === 'approved' || p.status === 'done').length,
    rejected: plan.filter(p => p.status === 'rejected').length,
    pending: plan.filter(p => p.status === 'pending').length,
  };

  return (
    <div className="plan-mode">
      <div className="plan-mode-header">
        <h3>Plan Mode</h3>
        <div className="plan-stats">
          <span className="plan-stat pending">{stats.pending} pending</span>
          <span className="plan-stat approved">{stats.approved} approved</span>
          {stats.rejected > 0 && <span className="plan-stat rejected">{stats.rejected} rejected</span>}
        </div>
        <button className="plan-close" onClick={onClose}>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div className="plan-steps">
        {plan.map(step => (
          <div key={step.id} className={`plan-step plan-step-${step.status}`}>
            <div className="plan-step-icon">
              {step.status === 'pending' && <div className="plan-step-pending-ring" />}
              {step.status === 'approved' && (
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#48b87a" strokeWidth="2.5">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              )}
              {step.status === 'executing' && (
                <div className="plan-step-executing-pulse" />
              )}
              {step.status === 'done' && (
                <svg width="12" height="12" viewBox="0 0 24 24" fill="#5e8aff" stroke="none">
                  <circle cx="12" cy="12" r="10" />
                  <polyline points="20 6 9 17 4 12" stroke="#fff" strokeWidth="2" fill="none" />
                </svg>
              )}
              {step.status === 'rejected' && (
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#e04e4e" strokeWidth="2.5">
                  <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              )}
            </div>

            <div className="plan-step-body">
              <div className="plan-step-header">
                <span className="plan-step-action">{step.action}</span>
                <span className="plan-step-status-tag">{step.status}</span>
              </div>
              <div className="plan-step-detail">{step.detail}</div>
            </div>

            {step.status === 'pending' && (
              <div className="plan-step-actions">
                <button className="plan-action-btn approve" onClick={() => approvePlanStep(sessionId, step.id)} title="Approve">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="#48b87a" strokeWidth="2.5">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                </button>
                <button className="plan-action-btn reject" onClick={() => rejectPlanStep(sessionId, step.id)} title="Reject">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="#e04e4e" strokeWidth="2.5">
                    <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              </div>
            )}

            {step.status === 'approved' && (
              <div className="plan-step-actions">
                <span className="plan-approved-label">Approved</span>
              </div>
            )}
          </div>
        ))}
      </div>

      <div className="plan-footer">
        {stats.pending === 0 && stats.rejected === 0 && (
          <div className="plan-ready">All steps approved. Ready to execute.</div>
        )}
        {stats.pending === 0 && stats.rejected > 0 && (
          <div className="plan-blocked">{stats.rejected} step(s) rejected. Adjust and regenerate.</div>
        )}
      </div>
    </div>
  );
}
