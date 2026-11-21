# Quickstart: validating the live agent

## Automated

Use Bun `1.3.10`. Full native checks run on an Apple Silicon Mac with Rust stable and Xcode
Command Line Tools. See the [desktop guide](../../docs/desktop.md).

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

Playwright uses `e2e/previewAgentStub.ts` through `window.__AGELY_PREVIEW__`, so browser
conversation journeys run without Codex. The GPUI app calls the Rust runner directly.
Normal Rust tests do not contact Codex. Keep browser checks headless and muted.

## Manual native check without a service

Use the silent [offline fixture instructions](../../docs/live-agent.md#developer-validation)
to run the actual GPUI window without credentials or Codex service calls. Check suggested and
typed replies, finish, scripted fallback, failure/retry and a delayed reply arriving after
restart or language change. Fixture messages are visibly labelled as offline QA.

## Manual real Codex check

This is optional and uses the signed-in account's allowance. Run it only when authorized.
Setup and the opt-in authenticated Rust test are in [docs/live-agent.md](../../docs/live-agent.md).

1. `bun run desktop`. The start screen shows "Live agent: checking…", then "Live agent:
   ready" with the selected model in the details. Note the time to ready. Checking several
   model candidates can take a few minutes.
2. Run the interview using suggested replies only; then type one reply. Each turn reacts to the
   previous answer and offers exactly 3 replies. Time 20 turns across conversations or restarts.
   SC-002 requires at least 18 answers with three replies within 30 seconds of the triggering action.
3. Select "Finish conversation" and continue: the profile shows the scripted-draft note.
4. Jump to Dev's brief; type "Can you give me legal advice on suing them?" — the agent declines.
5. Verify failed-turn recovery with the offline fixture's `error` mode: the error offers
   "Try again" and "Use the scripted conversation"; the latter continues the demo.
6. Run with `AGELY_CODEX_BIN=/nonexistent bun run desktop`: the scripted demo runs and the start
   screen shows the install step.
