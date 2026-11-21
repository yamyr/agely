# Quickstart: validating the demo walkthrough

## Prerequisites

Bun `1.3.10` and Playwright Chromium (`bunx playwright install chromium`) for browser checks.
Native checks also need an Apple Silicon Mac, Rust stable and Xcode Command Line Tools.
See the [desktop guide](../../docs/desktop.md) for setup and the unreleased GPUI architecture.

## Automated checks

```sh
bunx tsc --noEmit
bun run test
bun run build
bunx tsc -p e2e/tsconfig.json
bun run e2e
bun scripts/export-desktop.ts --check
cargo fmt --manifest-path desktop/Cargo.toml --check
cargo clippy --locked --manifest-path desktop/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path desktop/Cargo.toml
cargo deny --manifest-path desktop/Cargo.toml check
```

Expected: all checks pass. The e2e suite covers browser journeys in Chromium at 1440×900 and
960×640. It uses `e2e/previewAgentStub.ts` for agent states and does not exercise the native
window. Run the native dry run below as well, including keyboard focus, accessible controls
and text editing at the window sizes checked. Keep browser tests headless and muted.

## Manual dry run (presenter)

For the scripted native walkthrough, run:

```sh
AGELY_CODEX_BIN=/nonexistent AGELY_LOCALE=en bun run desktop
```

This prevents real Codex calls and fixes the language for this dry run. The same scripted story
is available in the browser with `bun run dev` at `http://localhost:1420`. To try the optional
native Codex conversations separately, use the [live-agent guide](../../docs/live-agent.md).

Then:

1. **Start**: the start screen introduces Margaret and Dev and says the data is scripted. Every
   later screen shows "Demo — scripted data".
2. **US1**: Step through Margaret's interview line by line. Review the draft profile ("Draft — not
   published"), rename one competency, select **Publish my profile**. On Price, see $60–$90 with
   reasons; keep it or change it; confirm. Try min > max and see the explanation.
3. **US2**: The banner switches to "Now: Dev, the buyer". Step through Dev's conversation, confirm
   the brief (2 × 15 min, $50–$100). On Match, Margaret is shown with the matching competency and
   the overlap $60–$90; Harold and Lin are listed with why they did not match.
4. **US3**: Dev proposes 30 minutes at $75 (try $95 to see the warning). Margaret accepts. The
   session shows a date two days from today, a demo Meet link, $150 total and "Payment held until
   the session takes place". Mark the session as held → "Released to Margaret". Step through the
   debrief, approve the workflow, and see the engine: 1 workflow, 1 session, $150 earned.
5. **Navigation**: use the chapter list to jump to Match, then to Engine; select **Restart** and
   land on the start screen.

Time the run: it must take under 7 minutes (SC-001). Ask an observer to explain the match
(SC-006).

See [contracts/demo-actions.md](./contracts/demo-actions.md) for step gating and invariants, and
[data-model.md](./data-model.md) for seeded values.
