import type { Page } from "@playwright/test";
import type { AgentStatus, AgentTurn } from "../src/demo/live/types";
import type { BrowserPreviewFixture } from "../src/lib/native";

/**
 * Exercises browser conversation UI with explicit fixture responses.
 * A sequence takes the next entry on each call and repeats the final entry.
 * An error entry rejects the call so retry and fallback use the normal UI path.
 */
declare global {
  interface Window {
    __AGELY_PREVIEW__?: BrowserPreviewFixture;
    __previewCalls: string[];
  }
}

export const NOT_INSTALLED = {
  state: "not_installed",
  model: null,
  detail: "Codex is not installed on this computer, so the scripted conversations are used.",
  fix: "Install it with: npm install -g @openai/codex — then select Check again.",
};

export const READY = {
  state: "ready",
  model: "test-model",
  detail: "Live agent ready, using test-model through your Codex sign-in.",
  fix: null,
};

export const CANNED_RESPONSES: Readonly<Record<string, unknown>> = {
  app_info: { name: "Agely", version: "0.0.0-e2e" },
  agent_status: NOT_INSTALLED,
  agent_turn: { message: "Hello from the fixture agent.", options: ["One", "Two", "Three"], done: false },
};

export function sequence(...entries: readonly unknown[]): unknown {
  return { __sequence: entries };
}

export function failure(code: string, message: string): unknown {
  return { __error: { code, message } };
}

export function collectProblems(page: Page): readonly string[] {
  const problems: string[] = [];
  page.on("pageerror", (error) => problems.push(`pageerror: ${error.message}`));
  page.on("console", (message) => {
    if (message.type() === "error" && message.text().startsWith("preview fixture:")) {
      problems.push(message.text());
    }
  });
  return problems;
}

export async function installPreviewAgentStub(page: Page, overrides: Readonly<Record<string, unknown>> = {}): Promise<void> {
  await page.addInitScript((canned: Readonly<Record<string, unknown>>) => {
    const cursors: Record<string, number> = {};
    window.__previewCalls = [];
    const invalid = (command: string): never => {
      const message = `preview fixture: invalid or missing response for ${command}`;
      console.error(message);
      throw new Error(message);
    };
    const next = (command: string): unknown => {
      window.__previewCalls.push(command);
      if (!Object.hasOwn(canned, command)) return invalid(command);
      let value = canned[command];
      if (value !== null && typeof value === "object" && "__sequence" in value) {
        const entries: unknown = value.__sequence;
        if (!Array.isArray(entries) || entries.length === 0) return invalid(command);
        const index = Math.min(cursors[command] ?? 0, entries.length - 1);
        cursors[command] = index + 1;
        value = entries[index];
      }
      if (value !== null && typeof value === "object" && "__error" in value) throw value.__error;
      return value;
    };

    const appInfo = next("app_info");
    if (
      appInfo === null || typeof appInfo !== "object" ||
      !("name" in appInfo) || typeof appInfo.name !== "string" ||
      !("version" in appInfo) || typeof appInfo.version !== "string"
    ) {
      return invalid("app_info");
    }

    window.__AGELY_PREVIEW__ = {
      appInfo: { name: appInfo.name, version: appInfo.version },
      status: async (): Promise<AgentStatus> => {
        const value = next("agent_status");
        if (
          value === null || typeof value !== "object" ||
          !("state" in value) || !("model" in value) ||
          !("detail" in value) || !("fix" in value)
        ) return invalid("agent_status");
        const { state, model, detail, fix } = value;
        if (
          state !== "checking" && state !== "ready" && state !== "not_installed" &&
          state !== "not_signed_in" && state !== "model_unavailable" &&
          state !== "unavailable" && state !== "error"
        ) return invalid("agent_status");
        if (
          (model !== null && typeof model !== "string") ||
          typeof detail !== "string" || (fix !== null && typeof fix !== "string")
        ) return invalid("agent_status");
        return { state, model, detail, fix };
      },
      turn: async (): Promise<AgentTurn> => {
        const value = next("agent_turn");
        if (
          value === null || typeof value !== "object" ||
          !("message" in value) || typeof value.message !== "string" ||
          !("done" in value) || typeof value.done !== "boolean" ||
          !("options" in value) || !Array.isArray(value.options)
        ) return invalid("agent_turn");
        const options: unknown[] = value.options;
        if (!options.every((option) => typeof option === "string")) return invalid("agent_turn");
        return { message: value.message, options, done: value.done };
      },
    };
  }, { ...CANNED_RESPONSES, ...overrides });
}
