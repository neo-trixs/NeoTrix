import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import styles from "./PlanPage.module.css";

interface Plan {
  id: string;
  name: string;
  status: string;
  steps: number;
  current_step: number;
  created: string;
}

const PlanPage: React.FC = () => {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);
  const [newPlanName, setNewPlanName] = useState("");
  const [selectedPlan, setSelectedPlan] = useState<Plan | null>(null);
  const [detailSteps, setDetailSteps] = useState<string[]>([]);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [currentStep, setCurrentStep] = useState<number>(0);

  const showStatus = (msg: string) => {
    setStatusMsg(msg);
    setTimeout(() => setStatusMsg(null), 3000);
  };

  const loadPlans = async () => {
    try {
      const result = await invoke<string>("plan_list");
      setPlans(JSON.parse(result));
    } catch (e) {
      console.error("Failed to load plans:", e);
    }
  };

  useEffect(() => {
    loadPlans().finally(() => setLoading(false));
  }, []);

  const handleCreate = async () => {
    if (!newPlanName.trim()) return;
    try {
      const result = await invoke<string>("plan_create", { name: newPlanName.trim() });
      showStatus(result);
      setNewPlanName("");
      await loadPlans();
    } catch (e) {
      showStatus(String(e));
    }
  };

  const handleSelect = async (plan: Plan) => {
    setSelectedPlan(plan);
    setCurrentStep(plan.current_step);
    try {
      const result = await invoke<string>("plan_steps", { planId: plan.id });
      setDetailSteps(JSON.parse(result));
    } catch {
      setDetailSteps([]);
    }
  };

  const handleStep = async () => {
    if (!selectedPlan) return;
    try {
      const result = await invoke<string>("plan_step", { planId: selectedPlan.id });
      showStatus(result);
      await handleSelect(selectedPlan);
      await loadPlans();
    } catch (e) {
      showStatus(String(e));
    }
  };

  const handleComplete = async () => {
    if (!selectedPlan) return;
    try {
      const result = await invoke<string>("plan_complete", { planId: selectedPlan.id });
      showStatus(result);
      setSelectedPlan(null);
      await loadPlans();
    } catch (e) {
      showStatus(String(e));
    }
  };

  const statusColor = (s: string) => {
    switch (s) {
      case "active": return "var(--nt-success)";
      case "completed": return "var(--nt-primary)";
      case "failed": return "var(--nt-danger)";
      default: return "var(--nt-text-muted)";
    }
  };

  return (
    <div className={styles.page} data-testid="plan-page">
      {statusMsg && <div className={styles.toast}>{statusMsg}</div>}

      <div className={styles.topBar}>
        <div className={styles.topLeft}>
          <h2>Plans</h2>
          <p className={styles.subtitle}>E8-guided plan mode — create, step through, and complete plans</p>
        </div>
        <div className={styles.createRow}>
          <input
            className={styles.createInput}
            placeholder="New plan name..."
            value={newPlanName}
            onChange={(e) => setNewPlanName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleCreate()}
            data-testid="plan-new-name"
          />
          <button className="btn-primary" onClick={handleCreate} disabled={!newPlanName.trim()} data-testid="plan-create-btn">
            + Create
          </button>
        </div>
      </div>

      <div className={styles.splitLayout}>
        <div className={styles.planList}>
          <h3 className={styles.sectionTitle}>All Plans ({plans.length})</h3>
          {loading ? (
            <div className={styles.loading}>Loading...</div>
          ) : plans.length === 0 ? (
            <div className={styles.empty}>No plans yet. Create one above.</div>
          ) : (
            <div className={styles.planCards}>
              {plans.map((plan) => (
                <button
                  key={plan.id}
                  className={`${styles.planCard} ${selectedPlan?.id === plan.id ? styles.planCardActive : ""}`}
                  onClick={() => handleSelect(plan)}
                  data-testid={`plan-card-${plan.id}`}
                >
                  <div className={styles.planCardHeader}>
                    <span className={styles.planName}>{plan.name}</span>
                    <span className={styles.planStatus} style={{ color: statusColor(plan.status) }}>
                      {plan.status}
                    </span>
                  </div>
                  <div className={styles.planCardBody}>
                    <span className={styles.planSteps}>{plan.current_step}/{plan.steps} steps</span>
                    <span className={styles.planDate}>{plan.created}</span>
                  </div>
                  <div className={styles.planProgress}>
                    <div
                      className={styles.planProgressFill}
                      style={{ width: `${plan.steps > 0 ? (plan.current_step / plan.steps) * 100 : 0}%` }}
                    />
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className={styles.planDetail}>
          {!selectedPlan ? (
            <div className={styles.empty}>Select a plan to view details</div>
          ) : (
            <div className={styles.detailContent}>
              <div className={styles.detailHeader}>
                <div>
                  <h3>{selectedPlan.name}</h3>
                  <span className={styles.detailStatus} style={{ color: statusColor(selectedPlan.status) }}>
                    {selectedPlan.status} — step {selectedPlan.current_step}/{selectedPlan.steps}
                  </span>
                </div>
                <div className={styles.detailActions}>
                  <button
                    className="btn-primary"
                    onClick={handleStep}
                    disabled={selectedPlan.status !== "active"}
                    data-testid="plan-step-btn"
                  >
                    Step
                  </button>
                  <button
                    className="btn-secondary"
                    onClick={handleComplete}
                    disabled={selectedPlan.status !== "active"}
                    data-testid="plan-complete-btn"
                  >
                    Complete
                  </button>
                </div>
              </div>
              <div className={styles.stepsList}>
                {detailSteps.map((step, i) => (
                  <div key={i} className={`${styles.stepItem} ${i < currentStep ? styles.stepDone : i === currentStep ? styles.stepCurrent : styles.stepPending}`}>
                    <div className={styles.stepIndex}>{i + 1}</div>
                    <div className={styles.stepContent}>
                      <span className={styles.stepText}>{step}</span>
                      {i < currentStep && <span className={styles.stepBadge}>done</span>}
                      {i === currentStep && selectedPlan.status === "active" && <span className={styles.stepBadgeCurrent}>current</span>}
                    </div>
                  </div>
                ))}
                {detailSteps.length === 0 && <div className={styles.empty}>No steps defined for this plan.</div>}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PlanPage;
