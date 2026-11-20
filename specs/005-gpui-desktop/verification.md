# GPUI desktop verification

Verified on an Apple silicon Mac on November 21, 2026. This migration is unreleased;
the published 0.1.3 downloads and release history have not been replaced.

## Automated checks

| Check | Observed result |
| --- | --- |
| `bun run test` | 427 tests passed across 42 files. |
| Both application and e2e TypeScript checks | Passed. |
| `bun run e2e` | 350 journeys passed, including both browser window sizes and the French full walkthrough. |
| `bun run build` and `bun run check:site` | Passed. The existing Vite large-chunk warning remains. |
| `bun run check:i18n` | 913 messages, 17 complete draft catalogs, matching interpolation slots. |
| `bun scripts/export-desktop.ts --check` | Embedded story, catalogs, formatting and live briefs are current. Native message arguments are also audited. |
| `cargo fmt --manifest-path desktop/Cargo.toml --check` | Passed. |
| Stable Rust full tests | 57 passed, one real authenticated Codex test intentionally ignored. |
| Stable Rust all-target Clippy, warnings as errors | Passed. |
| `cargo deny check` in `desktop/` | Advisories, bans, licenses and sources passed. |
| `bun run desktop:build` | Apple silicon application and DMG built; plist, architecture, ad hoc signature and disk-image verification passed. |

Eight app regressions exercise actual GPUI input notifications, handlers and keyboard events.
The four original state/observer tests and two keyboard tests were each observed failing
against isolated copies with their fixes removed, then passing on current sources. Two draft
persistence tests also failed before their observer fix and passed afterwards. Two
additional tests exercise native IME composition, UTF-16 selection and CJK/emoji commits.
The pure state suite completes the story in all 18 supported languages.

The first CI frontend run exposed host ICU drift in generated locale formats. Reviewed
canonical format definitions now drive export, with unchanged native output values and
all four strict byte-comparison checks retained. Freshness passes under Bun 1.4.0,
Bun 1.3.10 and Node 26.8.1. A regression runs the actual exporter with all Intl formatting
constructors disabled; malformed or incomplete locale definitions are also rejected.

Dependency policy still rejects unknown sources and wildcard dependencies. Both official
Zed Git sources require exact revisions. Direct date handling uses `time`; GPUI still
transitively uses unmaintained `chrono`, which the existing policy permits for transitive
dependencies. No advisory exception was added.

## Native window checks

All interaction used the silent native preview, isolated from the user's existing Agely
process. The release preview executable was checked byte-for-byte against the executable
inside the packaged application. No microphone, speaker playback or real service was used.

| Scenario | Observed result |
| --- | --- |
| Full native development-build journey | All 13 screens completed: interview, profile publication, price, brief, match, proposal, acceptance, simulated payment release, debrief and workflow approval. Engine showed one workflow, one completed session and $150. |
| Editing and validation | A skill named `Supplier strategy — 日本語 café 👩🏽‍💻` saved and appeared in the match explanation. A $1 proposal was blocked outside the shared $60–$90 range; $75 sent a $150 proposal. |
| Drafts and Back navigation | In the final packaged binary, changing the budget to $70–$95 immediately updated its summary and survived Back/Next. An unsent price of $80 also survived Back/Next, keeping the $160 total and explicit Send gate. |
| Packaged-binary keyboard | Tab then Enter opened the language choices once; another Enter closed them once; Space opened them once. Confirmation displayed only its two choices, with a visible focus border. |
| Japanese and reduced window | Welcome, Engine, all 18 language choices and confirmation cancellation were checked at 1120×720 content size. Text and controls wrapped without observed horizontal overflow. |
| Saved language | Japanese was selected, the process was closed, the locale environment override was removed, and relaunch still opened in Japanese. The preference state from before QA was restored afterwards. |
| Background turn and restart | A delayed fixture turn displayed the thinking state. Restart responded immediately; returning to the interview after the old result arrived still displayed a fresh Start button. |
| Failure and finish | A failed reply displayed recovery choices. Finish cleared the error and enabled Next; the following profile visibly disclosed its scripted source. |
| Retry and fallback | Retrying a failed opening produced the fixture response. Falling back after a failed reply restored the scripted conversation at 0 of 9 lines. |
| Accessibility | The macOS accessibility tree exposed headings, named buttons, editable fields and values, status/error messages and a language dialog. |

The deterministic CLI fixture is `scripts/fixtures/codex-gpui.py`. Set `AGELY_CODEX_BIN`
to its absolute path, `AGELY_CODEX_MODEL=offline-qa-fixture`, and `AGELY_QA_DIR` to a
disposable directory. Its `mode` file selects `ready`, `delay`, `error` or `done`.
Remove these overrides to use the actual local Codex installation.

## Artifacts and limits

Local review logs are in `.omo/evidence/gpui/`; the code review is in
`.omo/evidence/gpui-parity-code-review.md`. These machine-local files are not committed.

The application is `desktop/target/aarch64-apple-darwin/release/bundle/macos/Agely.app`.
The installer and SHA-256 file are in the adjacent `bundle/dmg/` directory.
The executable links macOS/Metal frameworks and has no WebKit dependency.

Exact 960×640 native-window visual QA was not achieved; browser checks cover that size,
and the native window enforces it as its minimum. OS input-source switching and spoken
VoiceOver output were not tested. IME composition was verified through GPUI's native
input-handler API. The real authenticated Codex test was not rerun for this migration.
The new native language preference does not import the former WebKit local storage;
people upgrading from Tauri may need to select their language once.
The bundle is ad hoc signed and has no Apple notarization. No release, website merge or
production deployment was performed.
