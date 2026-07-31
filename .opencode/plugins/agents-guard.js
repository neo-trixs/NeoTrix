// AGENTS.md structure guard — enforces the pointer-conservation HARD RULE mechanically.
// Fires on session.idle: validates that AGENTS.md has not grown beyond the L1 ceiling
// and that its section structure matches the whitelist. Violations are logged to
// ~/.neotrix/agents-guard-violations.log AND echoed loudly so no session can silently
// pollute the pointer file. Complemented by the git pre-commit hook.

import { readFileSync, appendFileSync, existsSync } from "node:fs";
import { join } from "node:path";

const REPO_ROOT = process.cwd();
const AGENTS_PATH = join(REPO_ROOT, "AGENTS.md");
const INDEX_PATH = join(REPO_ROOT, "experience-index.md");
const LOG_PATH = process.env.HOME + "/.neotrix/agents-guard-violations.log";

const MAX_LINES = 130;
const MAX_BYTES = 22000;

// Sections allowed in AGENTS.md (L1 pointer doc). Anything else = structural violation.
const ALLOWED_SECTIONS = [
  "## Experience Index",
  "## Skill Routing",
  "## Architecture",
  "## Always-On Core Rules",
  "## Shared Language",
  "## Build",
  "## Test",
  "## Key Locations",
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
        const lines = content.split("\n");

        let violated = false;

        // 1. Line-count ceiling (L1 pointer doc must stay thin)
        if (lines.length > MAX_LINES) {
          violated = true;
          logViolation(`AGENTS.md exceeded ${MAX_LINES} lines (now ${lines.length}). Cycle bodies must go to KB, not here.`);
        }

        // 2. Byte ceiling
        if (Buffer.byteLength(content, "utf8") > MAX_BYTES) {
          violated = true;
          logViolation(`AGENTS.md exceeded ${MAX_BYTES} bytes. Violation of pointer-conservation HARD RULE.`);
        }

        // 3. Section whitelist — no cycle-body sections may appear
        const h2Sections = lines
          .filter((l) => l.startsWith("## "))
          .map((l) => l.replace(/^##\s+/, "").trim());
        const forbidden = h2Sections.filter(
          (s) => !ALLOWED_SECTIONS.includes(s) &&
                 (s.includes("Cycle") || s.includes("Session") ||
                  s.includes("Build Baseline") || s.includes("元认知") ||
                  s.includes("吸收"))
        );
        if (forbidden.length > 0) {
          violated = true;
          logViolation(`AGENTS.md contains forbidden cycle-body sections: ${forbidden.join(", ")}. Move to KB via experience-tree, then revert.`);
        }

        // 4. Experience Index must point to the generated file, not inline a table
        if (content.includes("| Cycle | Date |")) {
          violated = true;
          logViolation(`AGENTS.md inlines an Experience Index table. Use @experience-index.md (auto-generated) instead.`);
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
