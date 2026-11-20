import { toMessage } from "../../lib/errors";
import { previewFixture } from "../../lib/native";
import type { AgentBrief, AgentErrorPayload, AgentStatus, AgentTurn, TranscriptLine } from "./types";

const BROWSER_PREVIEW: AgentStatus = {
  state: "unavailable",
  model: null,
  detail: "The live agent runs only in the desktop app; this browser preview uses the scripted conversations.",
  fix: "Run bun run desktop to use it.",
};

/** Browser previews stay scripted unless a journey explicitly installs an agent fixture. */
export async function getAgentStatus(recheck: boolean): Promise<AgentStatus> {
  const fixture = previewFixture();
  if (fixture === undefined) return BROWSER_PREVIEW;
  try {
    return await fixture.status(recheck);
  } catch (err) {
    return {
      state: "error",
      model: null,
      detail: `The live agent check failed: ${toMessage(err)}`,
      fix: "Select Check again.",
    };
  }
}

export async function runAgentTurn(brief: AgentBrief, transcript: readonly TranscriptLine[]): Promise<AgentTurn> {
  const fixture = previewFixture();
  if (fixture === undefined) {
    const notReady: AgentErrorPayload = { code: "not_ready", message: "The live agent runs only in the desktop app." };
    throw notReady;
  }
  return fixture.turn({ brief, transcript });
}
