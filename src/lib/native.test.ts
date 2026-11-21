// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from "vitest";
import { getAppInfo } from "./native";

afterEach(() => {
  delete window.__AGELY_PREVIEW__;
});

describe("browser preview metadata", () => {
  it("uses the website build metadata in a plain browser", () => {
    expect(getAppInfo()).toBeNull();
  });

  it("reads metadata from an explicitly installed preview fixture", () => {
    window.__AGELY_PREVIEW__ = {
      appInfo: { name: "Agely", version: "0.0.0-e2e" },
      status: vi.fn(),
      turn: vi.fn(),
    };
    expect(getAppInfo()).toEqual({ name: "Agely", version: "0.0.0-e2e" });
  });
});
