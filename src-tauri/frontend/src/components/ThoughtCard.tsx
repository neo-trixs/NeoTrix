import React from "react";
import type { BrainEvent } from "../types";

interface Props {
  event: BrainEvent;
}

const StageIcon: React.FC<{ status: string }> = ({ status }) => {
  if (status === "start") return <span className="tc-icon tc-icon-start">⟳</span>;
  if (status === "done") return <span className="tc-icon tc-icon-done">✓</span>;
  if (status === "skip") return <span className="tc-icon tc-icon-skip">–</span>;
  return <span className="tc-icon tc-icon-error">⚠</span>;
};

const ThoughtCard: React.FC<Props> = ({ event }) => {
  const [expanded, setExpanded] = React.useState(false);

  if (event.kind === "stage") {
    const isActive = event.status === "start";
    return (
      <div
        className={`thought-card ${isActive ? "thought-active" : ""}`}
        onClick={() => setExpanded(!expanded)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); setExpanded(!expanded); } }}
      >
        <div className="thought-card-line">
          <StageIcon status={event.status ?? ""} />
          <span className="thought-name">{event.name}</span>
          {event.duration_ms !== undefined && (
            <span className="thought-duration">{event.duration_ms}ms</span>
          )}
          <span className={`thought-badge thought-badge-${event.status}`}>{event.status}</span>
          <span className="thought-expand">{expanded ? "▾" : "▸"}</span>
        </div>
        {expanded && (
          <div className="thought-card-detail">
            <div className="thought-detail-row">
              <span className="thought-detail-label">Status</span>
              <span>{event.status}</span>
            </div>
            {event.duration_ms !== undefined && (
              <div className="thought-detail-row">
                <span className="thought-detail-label">Duration</span>
                <span>{event.duration_ms}ms</span>
              </div>
            )}
          </div>
        )}
      </div>
    );
  }

  if (event.kind === "knowledge") {
    return (
      <div className="thought-chip">
        <span className="tc-icon tc-icon-done">+</span>
        <span>{event.concept_count} concepts</span>
        <span className="thought-chip-domain">{event.domain}</span>
      </div>
    );
  }

  return null;
};

export default ThoughtCard;
