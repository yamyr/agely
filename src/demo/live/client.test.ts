// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from "vitest";
import { BRIEFS } from "./briefs";
import { getAgentStatus, runAgentTurn } from "./client";

const status = vi.fn();
const turn = vi.fn();

function installFixture() {
  window.__AGELY_PREVIEW__ = {
    appInfo: { name: "Agely", version: "0.0.0-e2e" },
    status,
    turn,
  };
}

afterEach(() => {
  status.mockReset();
  turn.mockReset();
  delete window.__AGELY_PREVIEW__;
});

describe("browser live conversation client", () => {
  it("reports the browser preview as unavailable without calling an agent", async () => {
    await expect(getAgentStatus(false)).resolves.toMatchObject({ state: "unavailable" });
    await expect(runAgentTurn(BRIEFS.interview, [])).rejects.toMatchObject({ code: "not_ready" });
    expect(status).not.toHaveBeenCalled();
    expect(turn).not.toHaveBeenCalled();
  });

  it("passes a recheck request to the explicit preview fixture", async () => {
    installFixture();
    status.mockResolvedValue({ state: "ready", model: "m", detail: "ok", fix: null });
    await expect(getAgentStatus(true)).resolves.toMatchObject({ state: "ready", model: "m" });
    expect(status).toHaveBeenCalledWith(true);
  });

  it("turns a failed status call into an error status", async () => {
    installFixture();
    status.mockRejectedValue(new Error("fixture unavailable"));
    await expect(getAgentStatus(false)).resolves.toMatchObject({ state: "error", detail: expect.stringContaining("fixture unavailable") });
  });

  it("sends the brief and transcript for a turn", async () => {
    installFixture();
    const response = { message: "Hi", options: ["a", "b", "c"], done: false };
    turn.mockResolvedValue(response);
    const transcript = [{ speaker: "agent" as const, text: "Hello" }];
    await expect(runAgentTurn(BRIEFS.brief, transcript)).resolves.toEqual(response);
    expect(turn).toHaveBeenCalledWith({ brief: BRIEFS.brief, transcript });
  });
});
