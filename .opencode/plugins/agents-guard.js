// AGENTS.md structure guard — enforces the pointer-conservation HARD RULE mechanically.
// Fires on session.idle: validates that AGENTS.md has not grown beyond the L1 ceiling
// and that its section structure matches the whitelist. Violations are logged to
// ~/.neotrix/agents-guard-violations.log AND echoed loudly so no session can silently
// pollute the pointer file. Complemented by the git pre-commit hook.
//
// Budget model (fixed in cycle 209):
//   The "## Experience Index" section is a SANCTIONED growing section — the HARD RULE
//   explicitly allows one pointer row per cycle. It therefore gets its own budget and is
//   EXCLUDED from the constant-content line/byte ceilings. Without this split, legitimate
//   one-row-per-cycle growth kept tripping a false "131 lines" violation (the file sat at
//   exactly the global ceiling), so every cycle the guard spam-logged noise.

import { readFileSync, appendFileSync, existsSync } from "node:fs";
import { join } from "node:path";

const REPO_ROOT = process.cwd();
const AGENTS_PATH = join(REPO_ROOT, "AGENTS.md");
const LOG_PATH = process.env.HOME + "/.neotrix/agents-guard-violations.log";

// Ceilings for CONSTANT L1 content (preamble + rules + sections, index excluded).
const MAX_CONTENT_LINES = 130;
const MAX_CONTENT_BYTES = 16000;
// Own budget for the sanctioned growing Experience Index pointer table.
const MAX_INDEX_ROWS = 80;
// Overall file bound (index row bytes dominate as cycles accumulate).
const MAX_BYTES = 35000;

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

// Split the file into the Experience Index section vs. the constant-content rest.
// Returns { indexStart, indexEnd, indexRows, contentLines, content }.
function splitSections(lines) {
  const headerIdx = lines.findIndex((l) => l === "## Experience Index");
  if (headerIdx === -1) {
    return { indexStart: -1, indexEnd: -1, indexRows: 0, contentLines: lines.length, content: lines };
  }
  let end = lines.length;
  for (let i = headerIdx + 1; i < lines.length; i++) {
    if (lines[i].startsWith("## ")) {
      end = i;
      break;
    }
  }
  // Pointer rows = data rows of the table (exclude the blank line and header row).
  const indexRows = lines.slice(headerIdx + 1, end).filter((l) => /^\|\s*\d+\s*\|/.test(l)).length;
  const content = [...lines.slice(0, headerIdx), ...lines.slice(end)];
  return { indexStart: headerIdx, indexEnd: end, indexRows, contentLines: content.length, content };
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

        // 1. Constant-content line ceiling (index excluded — it is a sanctioned growth area)
        const { indexRows, contentLines, content: rest } = splitSections(lines);
        if (contentLines > MAX_CONTENT_LINES) {
          violated = true;
          logViolation(`AGENTS.md constant content exceeded ${MAX_CONTENT_LINES} lines (now ${contentLines}, index excluded). Cycle bodies must go to KB, not here.`);
        }

        // 2. Experience Index own budget
        if (indexRows > MAX_INDEX_ROWS) {
          violated = true;
          logViolation(`AGENTS.md Experience Index exceeded ${MAX_INDEX_ROWS} pointer rows (now ${indexRows}). Archive old cycles to KB summaries, then trim the table.`);
        }

        // 3. Constant-content byte ceiling (index excluded)
        const contentBytes = Buffer.byteLength(rest.join("\n"), "utf8");
        if (contentBytes > MAX_CONTENT_BYTES) {
          violated = true;
          logViolation(`AGENTS.md constant content exceeded ${MAX_CONTENT_BYTES} bytes (now ${contentBytes}, index excluded).`);
        }

        // 4. Overall byte ceiling
        if (Buffer.byteLength(content, "utf8") > MAX_BYTES) {
          violated = true;
          logViolation(`AGENTS.md exceeded ${MAX_BYTES} bytes total. Violation of pointer-conservation HARD RULE.`);
        }

        // 5. Section whitelist — no cycle-body sections may appear
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

        // 6. Experience Index must stay a pointer table — no inline cycle body tables may appear
        if (content.includes("| Cycle | Date |") || content.includes("| Cycle | Session |")) {
          violated = true;
          logViolation(`AGENTS.md inlines a non-pointer Experience Index table. Pointers live in the | Cycle | Domain | Summary | table; bodies live in KB (absorb_session.py hub/query), never in AGENTS.md.`);
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
