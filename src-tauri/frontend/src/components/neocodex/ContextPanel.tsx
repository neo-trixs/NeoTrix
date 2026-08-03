import React from "react";
import { TaskPane, DiffPane, PreviewPane, TerminalPane, FileTreePanel } from "./index";
import styles from "./ContextPanel.module.css";

export interface ContextPanelProps {
  activeTab: "review" | "terminal" | "browser" | "file" | "tasks";
  taskSteps: Array<{ id: string; name: string; args: string; startedAt: number; status: "running" | "done"; success?: boolean }>;
  taskStartedAt: number | null;
  onFilePick?: (path: string) => void;
}

export function ContextPanel({
  activeTab,
  taskSteps,
  taskStartedAt,
  onFilePick,
}: ContextPanelProps) {
  return (
    <div className={styles.panel}>
      <div className={styles.content}>
        {activeTab === "tasks" && <TaskPane steps={taskSteps} startedAt={taskStartedAt} />}
        {activeTab === "review" && <DiffPane />}
        {activeTab === "browser" && <PreviewPane />}
        {activeTab === "terminal" && <TerminalPane />}
        {activeTab === "file" && <FileTreePanel onPick={onFilePick} />}
      </div>
    </div>
  );
}