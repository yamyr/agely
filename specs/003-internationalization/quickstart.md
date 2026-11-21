# Quickstart: validating internationalization

Use Bun `1.3.10`. Native checks also need an Apple Silicon Mac, Rust stable and Xcode Command
Line Tools. The [desktop guide](../../docs/desktop.md) covers the current GPUI source tree and
shared assets.

## Browser and website

1. Run `bun run dev` and open `http://localhost:1420`. Choose a language by its native name.
   Confirm restart, then follow the story. Cancel a second change and check that edits remain.
2. Reload and verify the preference and document language. Try `?lang=ja`, unsupported query
   values, blocked storage and browser regional language preferences. In `#/app`, confirm a
   language change keeps sign-in and brief edits in place.
3. Run `bun run check:i18n` for source/catalogue coverage and interpolation parity.
4. Run `bun run test`, `bunx tsc --noEmit`, `bunx tsc -p e2e/tsconfig.json`, `bun run build`
   and `bun run e2e`. Browser agent states use `e2e/previewAgentStub.ts` without real Codex.
5. Run `bun run check:site`. Inspect website desktop/phone views, long German/Finnish text and
   CJK/Cyrillic rendering. Keep browser checks headless and muted. Browser and website builds
   replace each other's `dist/`; run `bun run build` again before a browser app preview.

## Native GPUI app

1. Run `AGELY_CODEX_BIN=/nonexistent bun run desktop` for the scripted walkthrough. Choose a
   language, confirm restart, complete the story and cancel another change to preserve edits.
2. Quit and relaunch to verify the saved preference. `AGELY_LOCALE=ja` can override the startup
   language for QA; remove that override before testing persistence. Browser URL parameters
   and document-language attributes apply only to browser checks.
3. After changing catalogues or shared content, regenerate and verify embedded assets:

   ```sh
   bun run desktop:assets
   bun scripts/export-desktop.ts --check
   ```

   Native number and currency formats come from reviewed `scripts/desktop-formatting.json`.
   Normal exports must not depend on the host's Intl/ICU data.
4. Run the native checks:

   ```sh
   cargo fmt --manifest-path desktop/Cargo.toml --check
   cargo clippy --locked --manifest-path desktop/Cargo.toml --all-targets -- -D warnings
   cargo test --locked --manifest-path desktop/Cargo.toml
   cargo deny --manifest-path desktop/Cargo.toml check
   ```

5. Inspect the actual native window for long translations, CJK/Cyrillic rendering, Unicode
   input, keyboard focus, accessible names and scrolling at the window sizes checked.
6. Inspect native prompt unit tests for every response language. The silent offline Codex
   fixture can check language restart while a turn is pending. See [developer validation](../../docs/live-agent.md#developer-validation).
   A real authenticated Codex conversation is optional and incurs usage; do not run it for
   verification without authorization.

Translations are model-generated drafts, not certified language review. The English video and unmodified source evidence are labelled/preserved. The advice guard recognizes representative keywords only.
