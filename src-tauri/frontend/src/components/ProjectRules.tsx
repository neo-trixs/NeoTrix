import React, { useState, useEffect, useCallback } from "react";
import { useStore } from "../store";

const DEFAULT_RULES = [
  "Always ask before editing files",
  "Use TypeScript for new files",
  "Prefer functional components with hooks",
  "Run tests before committing",
  "Keep functions under 50 lines",
  "Use meaningful variable names",
  "Write unit tests for new logic",
  "Use async/await over raw promises",
  "Prefer early returns over nested ifs",
  "Use environment variables for secrets",
].join("\n");

const TEMPLATES: Record<string, string> = {
  "Strict (ask before write)": [
    "Always ask before editing any file",
    "Never delete files without confirmation",
    "Review diffs before applying changes",
    "Ask before installing new dependencies",
    "Require approval for network requests",
    "Never run shell commands without asking",
  ].join("\n"),
  "Relaxed (auto-edit)": [
    "Auto-edit any file without asking",
    "Skip confirmation for small changes (<10 lines)",
    "Auto-install dependencies",
    "Run tests after edits automatically",
    "Auto-format on save",
  ].join("\n"),
  "Code Review (review all changes)": [
    "Submit all changes for code review",
    "Attach test results to each change",
    "Require at least one approval before merge",
    "Run linter on all changed files",
    "Generate changelog entries automatically",
  ].join("\n"),
  "Custom": "",
};

const STORAGE_PREFIX = "neotrix_project_rules_";
const DEFAULT_STORAGE_KEY = "neotrix_default_rules";

const ProjectRules: React.FC = () => {
  const setProjectRulesVisible = useStore((s) => s.setProjectRulesVisible);
  const projectPath = useStore((s) => s.projectPath);

  const [defaultRules, setDefaultRules] = useState(DEFAULT_RULES);
  const [projectRules, setProjectRules] = useState("");
  const [activeTemplate, setActiveTemplate] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const storageKey = projectPath ? `${STORAGE_PREFIX}${projectPath}` : null;

  useEffect(() => {
    try {
      const stored = localStorage.getItem(DEFAULT_STORAGE_KEY);
      if (stored) setDefaultRules(stored);
    } catch {}
  }, []);

  useEffect(() => {
    if (!storageKey) return;
    try {
      const stored = localStorage.getItem(storageKey);
      if (stored) setProjectRules(stored);
    } catch {}
  }, [storageKey]);

  const handleSave = useCallback(() => {
    try {
      localStorage.setItem(DEFAULT_STORAGE_KEY, defaultRules);
      if (storageKey) localStorage.setItem(storageKey, projectRules);
    } catch {}
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }, [defaultRules, projectRules, storageKey]);

  const handleTemplateSelect = useCallback((name: string) => {
    const content = TEMPLATES[name];
    if (content === undefined) return;
    setProjectRules(content);
    setActiveTemplate(name);
    if (content) {
      try {
        if (storageKey) localStorage.setItem(storageKey, content);
      } catch {}
    }
  }, [storageKey]);

  return (
    <div className="overlay" onClick={() => setProjectRulesVisible(false)}>
      <div className="project-rules-panel overlay-panel" onClick={(e) => e.stopPropagation()}>
        <div className="project-rules-header">
          <div className="project-rules-header-left">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
            </svg>
            <h2>Project Rules</h2>
          </div>
          <button className="settings-close-btn" onClick={() => setProjectRulesVisible(false)} title="Close">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M4 4l8 8M12 4l-8 8" />
            </svg>
          </button>
        </div>

        <div className="project-rules-body">
          <div className="project-rules-section">
            <label className="project-rules-label">Default agent rules</label>
            <p className="project-rules-hint">These rules apply to all sessions. Each line is one rule.</p>
            <textarea
              className="project-rules-textarea"
              value={defaultRules}
              onChange={(e) => setDefaultRules(e.target.value)}
              rows={10}
              spellCheck={false}
            />
          </div>

          {projectPath && (
            <div className="project-rules-section">
              <label className="project-rules-label">
                Project-specific rules
                <span className="project-rules-project-path" title={projectPath}>
                  — {projectPath.split("/").pop() || projectPath}
                </span>
              </label>
              <p className="project-rules-hint">These rules override defaults for this project.</p>
              <textarea
                className="project-rules-textarea"
                value={projectRules}
                onChange={(e) => setProjectRules(e.target.value)}
                rows={8}
                spellCheck={false}
                placeholder="# Add project-specific rules here&#10;# Example: &#10;Use React 18 patterns&#10;Prefer named exports&#10;..."
              />
            </div>
          )}

          <div className="project-rules-section">
            <label className="project-rules-label">Template presets</label>
            <div className="project-rules-templates">
              {Object.keys(TEMPLATES).map((name) => (
                <button
                  key={name}
                  className={`project-rules-template-btn ${activeTemplate === name ? "active" : ""}`}
                  onClick={() => handleTemplateSelect(name)}
                >
                  {name}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="project-rules-footer">
          <span className="project-rules-info">
            Rules use markdown-like format • One rule per line
          </span>
          <div className="project-rules-actions">
            <button className="btn btn-secondary btn-sm" onClick={() => setProjectRulesVisible(false)}>
              Cancel
            </button>
            <button className="btn btn-primary btn-sm" onClick={handleSave}>
              {saved ? "Saved ✓" : "Save rules"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ProjectRules;
