// AGENTS.md structure guard — enforces the pointer-conservation HARD RULE mechanically.
// Fires on session.idle: validates that AGENTS.md has not grown beyond the L1 ceiling,
// that its section structure matches the whitelist, and that NO per-cycle growth area
// (Experience Index table / cycle bodies) has been appended. Violations are logged to
// ~/.neotrix/agents-guard-violations.log AND echoed loudly so no session can silently
// pollute the pointer file. Complemented by the git pre-commit hook.
//
// Budget model (finalized in cycle 209 refactor):
//   AGENTS.md is a PURE GUIDANCE doc. The Experience Index section (previously sanctioned
//   to grow one row per cycle) was the root cause of chronic false "131 lines" violations:
//   pointer growth collided with the global ceiling and byte headroom. It is now FORBIDDEN
//   entirely — cycle pointers live only in the KB `experience` hub (absorb_session.py
//   hub/query). The guard therefore enforces simple total ceilings on the constant content
//   plus an explicit "no Experience Index section" check.

import { readFileSync, appendFileSync, existsSync } from "node:fs";
import { join } from "node:path";

const REPO_ROOT = process.cwd();
const AGENTS_PATH = join(REPO_ROOT, "AGENTS.md");
const LOG_PATH = process.env.HOME + "/.neotrix/agents-guard-violations.log";

// Total ceilings for the whole L1 pointer doc (now index-free, so a single budget).
const MAX_LINES = 130;
const MAX_BYTES = 22000;

// Sections allowed in AGENTS.md (L1 pointer doc). Anything else = structural violation.
const ALLOWED_SECTIONS = [
  "## Skill Routing",
  "## Architecture",
  "## Always-On Core Rules",
  "## Shared Language",
  "## Build",
  "## Test",
  "## Key Locations",
];

// Sections that are structurally forbidden — any per-cycle growth area or body dump.
const FORBIDDEN_SECTIONS = [
  "## Experience Index",
];

function logViolation(msg) {
  const line = `[${new Date().toISOString()}] ${msg}\n`;
  try {
    appendFileSync(LOG_PATH, line);
  } catch (_) {}
  console.error("\n[AGENTS-GUARD] " + msg);
}

export const AgentsGuardPlugin = async ({ $ }) => {
  return {
    event: async ({ event }) => {
      if (event.type !== "session.idle") return;
      try {
        if (!existsSync(AGENTS_PATH)) return;
        const content = readFileSync(AGENTS_PATH, "utf8");
        // Ignore the single trailing newline so wc -l semantics match the ceiling
        const lines = content.replace(/\n$/, "").split("\n");

        let violated = false;

        // 1. Total line ceiling
        if (lines.length > MAX_LINES) {
          violated = true;
          logViolation(`AGENTS.md exceeded ${MAX_LINES} lines (now ${lines.length}). AGENTS.md is guidance-only; cycle content must go to KB, never here.`);
        }

        // 2. Total byte ceiling
        if (Buffer.byteLength(content, "utf8") > MAX_BYTES) {
          violated = true;
          logViolation(`AGENTS.md exceeded ${MAX_BYTES} bytes. Violation of pointer-conservation HARD RULE.`);
        }

        // 3. Section whitelist — allowed sections only; no cycle-body sections may appear
        const h2Sections = lines
          .filter((l) => l.startsWith("## "))
          .map((l) => l.replace(/^##\s+/, "").trim());
        const unknown = h2Sections.filter((s) => !ALLOWED_SECTIONS.includes(s));
        if (unknown.length > 0) {
          violated = true;
          logViolation(`AGENTS.md contains non-whitelisted sections: ${unknown.join(", ")}. Allowed: ${ALLOWED_SECTIONS.join(", ")}.`);
        }

        // 4. Explicitly forbidden growth areas
        const growth = h2Sections.filter((s) => FORBIDDEN_SECTIONS.includes(s));
        if (growth.length > 0) {
          violated = true;
          logViolation(`AGENTS.md contains forbidden per-cycle growth area: ${growth.join(", ")}. Cycle pointers live in KB (absorb_session.py hub/query), never in AGENTS.md.`);
        }

        // 5. No inline Experience Index table (header or data rows)
        if (content.includes("## Experience Index") ||
            content.includes("| Cycle | Domain | Summary |") ||
            content.includes("| Cycle | Date |") ||
            content.includes("| Cycle | Session |")) {
          violated = true;
          logViolation(`AGENTS.md inlines an Experience Index table. Cycle pointers live in the KB experience hub, never in AGENTS.md.`);
        }

        if (violated) {
          console.error("[AGENTS-GUARD] Pointer conservation violated. Inspect ~/.neotrix/agents-guard-violations.log and revert AGENTS.md to HEAD if needed.");
        }
      } catch (e) {
        console.error("[AGENTS-GUARD] check failed:", e);
      }
    },
  };
};
