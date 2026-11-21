---
name: gstack-verify
description: Use while coding, before /gstack-ship - run Agely's full check set (tsc, Vitest, build, Playwright, cargo fmt/clippy/test/deny, website check), fix what breaks, repeat until green. Adapted from gstack /verify.
---

<!-- Adapted from garrytan/gstack verify/SKILL.md @ f0cd9f8c (MIT). gstack runtime
     (gstack-checks, telemetry, learnings) removed; checks hardcoded to Agely's
     constitution "done" list and .github/workflows/ci.yml. See docs/tooling/gstack.md. -->

# /gstack-verify: build, check, fix, repeat

Use this while building. It runs checks and fixes code. It never reviews, bumps a
version, pushes, or opens a PR: that is `/gstack-ship`.

## 1. The check set

These are the checks the constitution (Principle IV) and `ci.yml` require. Run them from
the repo root. Each Bash call is a fresh shell.

| # | Name | Command | Notes |
|---|------|---------|-------|
| 1 | typecheck | `bunx tsc --noEmit` | |
| 2 | unit | `bun run test` | Vitest |
| 3 | build | `bun run build` | `tsc && vite build`, creates `dist/` |
| 4 | e2e-typecheck | `bunx tsc -p e2e/tsconfig.json` | |
| 5 | e2e | `bun run e2e` | Playwright; starts its own Vite server on `E2E_PORT` (default 1493, never 1420) |
| 6 | rust-fmt | `cargo fmt --manifest-path desktop/Cargo.toml --check` | |
| 7 | rust-clippy | `cargo clippy --manifest-path desktop/Cargo.toml --all-targets -- -D warnings` | run native checks on macOS with Xcode Command Line Tools |
| 8 | rust-test | `cargo test --manifest-path desktop/Cargo.toml` | Rust state and local runner tests |
| 9 | licences | `cargo deny --manifest-path desktop/Cargo.toml check` | needs `cargo install cargo-deny`; report `ENV` if it is not installed, CI runs it regardless |
| 10 | site | `bun run check:site` | builds the website into `dist/` and checks every page; browser app and site builds replace each other's output |
| 11 | native-assets | `bun scripts/export-desktop.ts --check` | verifies embedded JSON matches shared story, locale catalogues and design tokens |
| 12 | catalogues | `bun run check:i18n` | validates all supported locale messages and interpolation slots |

Scope shortcut while iterating: if `git diff --name-only main...HEAD` plus uncommitted
changes touch nothing under `desktop/`, you may skip 6-9 during the loop, but the
final run in step 3 always runs all twelve. Never invent a command the repo does not declare;
if `package.json` scripts or `ci.yml` change, follow them instead of this table.

If `node_modules/` is missing, say so and ask before running `bun install`. If Playwright
browsers are missing (`bunx playwright install chromium` would be needed), report it as ENV
and ask; do not install on your own.

## 2. Loop: run, fix, run

Run the checks in order, recording `CHECK <name> PASS|FAIL|ENV|TIMEOUT <secs>` for each.
Show at most 40 failing lines per check.

- **FAIL:** fix the code under test. A changed source file breaking an untouched test is
  your bug, not a pre-existing one. Call a failure "possibly pre-existing" only if it was
  already red on your first run this session, before any edit (check with `git stash` only
  if the user agrees).
- **No test covers a touched file** (new domain logic with no unit test, a new browser flow
  with no Playwright journey, or a native flow without Rust state tests and actual GPUI window
  checks): that is a gap. Principle IV requires domain tests seen failing first and verification
  on the platform where the flow runs. Add the test,
  or report the gap explicitly. Never call a change green when no test covers it.
- **ENV / TIMEOUT:** a missing tool, browser, service or network. Report it with the log
  line; do not fix the environment or work around it.
- While fixing one check, rerun only that check; then a full run before you stop.

**Never** weaken, skip (`.skip`, `.only`, `#[ignore]`), delete or edit a test, a lint/tsconfig
setting, a clippy allow, or a `package.json` script to get green.

Stop after 5 full runs, when the same failure comes back twice after a fix, or on ENV/TIMEOUT.
Report what is still red and why. Before another fix to a stubborn red in your own change,
reproduce it in one focused test and trace the root cause.

## 3. Hand off

For changed native screens, also run the actual GPUI window journey, keyboard and accessibility
checks. For changed website pages, inspect desktop and phone widths as the constitution requires.

Do a final full run of all twelve checks on the current tree, with no edits after it. Print:

```
VERIFY: READY_FOR_SHIP head=<git rev-parse --short HEAD> dirty=<yes|no from git status --porcelain> checks=12/12
```

Any edit after this line invalidates it; rerun the affected checks.

Only print `READY_FOR_SHIP` when all twelve and the applicable window/page checks passed on this exact tree; otherwise print
`VERIFY: NOT_READY` with the red checks. Stop with "ready for /gstack-ship".

## Completion status

End with one of: **DONE** (all green, evidence above), **DONE_WITH_CONCERNS** (green, but
list gaps such as untested files), **BLOCKED** (state the blocker and what was tried),
**NEEDS_CONTEXT** (state exactly what is missing).
