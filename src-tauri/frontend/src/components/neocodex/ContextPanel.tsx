import React from "react";
import { TaskPane, DiffPane, PreviewPane, TerminalPane, CapabilityHealthPane } from "./index";
import styles from "./ContextPanel.module.css";

export interface ContextPanelProps {
  activeTab: "task" | "diff" | "preview" | "terminal" | "capability";
  taskSteps: Array<{ id: string; name: string; args: string; startedAt: number; status: "running" | "done"; success?: boolean }>;
  taskStartedAt: number | null;
  health: any;
}

export function ContextPanel({
  activeTab,
  taskSteps,
  taskStartedAt,
  health,
}: ContextPanelProps) {
  return (
    <div className={styles.panel}>
      <div className={styles.content}>
        {activeTab === "task" && <TaskPane steps={taskSteps} startedAt={taskStartedAt} />}
        {activeTab === "diff" && <DiffPane />}
        {activeTab === "preview" && <PreviewPane />}
        {activeTab === "terminal" && <TerminalPane />}
        {activeTab === "capability" && <CapabilityHealthPane data={health} />}
      </div>
    </div>
  );
}