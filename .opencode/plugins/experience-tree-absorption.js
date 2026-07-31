// Unified end-of-conversation absorption plugin (experience-tree protocol).
// Fires on session.idle (no active assistant turn) to auto-absorb distilled
// experience into KB kv_store `experience` namespace.
// Session-end flow is: absorb session.json → close snapshot.
// Session-start snapshot is created manually by the agent (see SKILL.md).

const ABSORB = process.env.HOME + "/.agents/skills/experience-tree/scripts/absorb_session.py";
const PENDING = process.env.HOME + "/.neotrix/pending-absorb.json";

import { readFileSync, existsSync } from "node:fs";

export const AbsorptionPlugin = async ({ $ }) => {
  return {
    event: async ({ event }) => {
      // Auto-absorb a pending session.json when the conversation goes idle.
      if (event.type === "session.idle") {
        try {
          if (!existsSync(PENDING)) return;
          const pending = JSON.parse(readFileSync(PENDING, "utf8"));
          const cycle = pending.cycle || "unknown";
          await $`python3 ${ABSORB} absorb ${PENDING}`;
          await $`rm -f ${PENDING}`;
          await $`python3 ${ABSORB} close --cycle ${cycle}`;
        } catch (e) {
          console.error("[experience-tree] session-end absorption failed:", e);
        }
      }
    },
  };
};
