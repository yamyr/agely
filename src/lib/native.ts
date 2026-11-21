import type { AgentBrief, AgentStatus, AgentTurn, TranscriptLine } from "../demo/live/types";

export interface AppInfo {
  readonly name: string;
  readonly version: string;
}

export interface PreviewTurnRequest {
  readonly brief: AgentBrief;
  readonly transcript: readonly TranscriptLine[];
}

/**
 * Browser journeys install this fixture to exercise the live conversation UI.
 * The GPUI desktop calls Rust directly and does not use this transport.
 */
export interface BrowserPreviewFixture {
  readonly appInfo: AppInfo;
  readonly status: (recheck: boolean) => Promise<AgentStatus>;
  readonly turn: (request: PreviewTurnRequest) => Promise<AgentTurn>;
}

declare global {
  interface Window {
    __AGELY_PREVIEW__?: BrowserPreviewFixture;
  }
}

export function previewFixture(): BrowserPreviewFixture | undefined {
  return typeof window === "undefined" ? undefined : window.__AGELY_PREVIEW__;
}

/** Optional fixture metadata, otherwise the footer uses the website build version. */
export function getAppInfo(): AppInfo | null {
  return previewFixture()?.appInfo ?? null;
}
