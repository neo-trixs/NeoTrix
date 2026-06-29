import React from "react";

interface WorkflowStep {
  id: string;
  agentName: string;
  status: "pending" | "running" | "completed" | "failed";
  startedAt?: number;
  completedAt?: number;
  duration?: string;
}

interface WorkflowPanelProps {
  steps: WorkflowStep[];
  onStepClick: (stepId: string) => void;
  onClear: () => void;
}

const WorkflowPanel: React.FC<WorkflowPanelProps> = ({ steps, onStepClick, onClear }) => {
  return (
    <div className="workflow-panel glass-panel">
      <div className="workflow-header">
        <h3>工作流</h3>
        <span className="workflow-count">{steps.filter(s => s.status === "completed").length}/{steps.length}</span>
        <button className="btn-icon" onClick={onClear} title="清除">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3"><path d="M2 3h8M4 3V2a1 1 0 011-1h2a1 1 0 011 1v1M5 5v4M7 5v4"/></svg>
        </button>
      </div>
      <div className="workflow-steps">
        {steps.length === 0 ? (
          <div className="workflow-empty">暂无工作流</div>
        ) : steps.map(step => (
          <div key={step.id} className={`workflow-step workflow-step-${step.status}`} onClick={() => onStepClick(step.id)}>
            <div className="workflow-step-status">
              {step.status === "running" ? <span className="workflow-spinner" /> :
               step.status === "completed" ? <svg width="10" height="10" viewBox="0 0 10 10" fill="#34c759"><path d="M2 5l2 2 4-4"/></svg> :
               step.status === "failed" ? <svg width="10" height="10" viewBox="0 0 10 10" fill="#ff3b30"><path d="M2 2l6 6M8 2l-6 6"/></svg> :
               <span className="workflow-dot" />}
            </div>
            <div className="workflow-step-info">
              <span className="workflow-step-name">{step.agentName}</span>
              {step.duration && <span className="workflow-step-duration">{step.duration}</span>}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default WorkflowPanel;
export type { WorkflowStep };
