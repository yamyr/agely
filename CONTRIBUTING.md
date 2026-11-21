# Contributing

Thanks for looking at Agely. It is a hackathon prototype of an expert network for older adults,
and contributions from people and from coding agents are both welcome on the same terms.

## Before you start

- Read [AGENTS.md](AGENTS.md). It has the working rules, the commands and a map of the
  repository, and it applies to everyone.
- The principles behind those rules are in [the constitution](.specify/memory/constitution.md).
  A change that breaks one needs to say so and explain why.

## Making a change

1. Branch from `main`. Changes reach `main` only through a pull request.
2. A change to the app (`src/`, `desktop/`) follows Spec Kit, with its artifacts in
   `specs/<feature>/`. A change to the website (`site/`) does not need a spec.
3. Run the checks in AGENTS.md. A bug fix comes with a test that fails without it.
4. In the pull request, say what changed, what you tested, and what you could not run.
5. If you add a font, image, video or other asset, add it to [ATTRIBUTION.md](ATTRIBUTION.md)
   in the same change. If you add a dependency, check its licence is permissive.

CI must be green before a merge. Merging to `main` publishes the website.

## Run the app

Use Bun `1.3.10`, matching `package.json` and CI. For the native GPUI app, use an Apple Silicon
Mac with Rust stable and Xcode Command Line Tools. From the repository root:

```sh
bun install --frozen-lockfile
bun run desktop
```

The GPUI migration is unreleased and package metadata remains at `0.1.3`. These commands run
the native source tree. The React browser app starts with `bun run dev` on port `1420`.
Playwright starts its own browser server on port `1493`. The [desktop guide](docs/desktop.md)
covers native architecture and packaging.

The native walkthrough lives in `desktop/`. The browser walkthrough and marketplace live in
`src/`. Both use the authored story and translation catalogues. `bun run desktop` generates
the embedded JSON before compiling. After changing shared content, run:

```sh
bun run desktop:assets
bun scripts/export-desktop.ts --check
```

Edit the shared sources rather than generated files in `desktop/assets/`. Reviewed native
number and currency formats live in `scripts/desktop-formatting.json`; normal exports must
not depend on the host's Intl/ICU data. Native visual values come directly from the embedded
`src/ui/tokens.css`.

## Verify a change

Run the full check set in [AGENTS.md](AGENTS.md#commands) before opening a pull request.
Full native Rust checks run on macOS. A native screen change also needs a journey through
the actual GPUI window, including keyboard focus, scrolling, accessible names and text input.
Record the window sizes and languages checked. See the [accessibility checklist](.claude/skills/a11y-check/SKILL.md).

Browser tests use `e2e/previewAgentStub.ts` to model live-agent replies. They cover browser
conversation states. Native process execution and native rendering need their own Rust tests
and window checks. The [live-agent guide](docs/live-agent.md#developer-validation) describes a
silent offline Codex fixture and the separate opt-in authenticated test.

Keep tests silent. Use muted headless browsers, never play generated speech through the Mac's
speakers, and close any test browser that holds a microphone when finished.

## Licence

By contributing you agree that your work is released under the [MIT License](LICENSE).

## Security

See [SECURITY.md](SECURITY.md) for how to report a vulnerability.

## Maintain translations

English source messages are the catalog keys. Add each new message to the relevant sorted
source inventory in `site/i18n/messages/`, then add its draft translation to every file in
`site/i18n/catalogs/`. Keep numbered slots such as `{0}` unchanged; their order may change.
Run `bun run check:i18n` to detect missing translations and lost slots, then regenerate the
native assets with `bun run desktop:assets` and check their freshness.

Use complete sentences when interpolating values. Render translations as text, never as HTML.
Keep domain IDs, user-written text, names, URLs and canonical ledger evidence separate from
translated labels. Browser values use the shared `site/i18n/core.js` helpers. Native values use
`desktop/src/i18n.rs` with the reviewed formatting data. Prices stay in USD and a language
choice does not change the time zone.

The current catalogs are model-generated drafts and need human language review. Missing or
malformed entries fall back to the English source. Keep the visible draft/fallback disclosure
until the language-review process changes. Update `ATTRIBUTION.md` when adding generated text.

The walkthrough asks before restarting in another language. Cancelling preserves edits.
Confirming starts a fresh story and discards pending replies from the previous run. The
marketplace changes language in place, including during sign-in or while editing a brief.
Keep the fixed reply values and skill tags stable when editing their translations. Use
`e2e/i18n.spec.ts` and `e2e/product-i18n.spec.ts` to verify both behaviors. Run the
accessibility checks and inspect long translations in both the browser and the native window
when their content or layout changes.

The offline restricted-advice guard recognizes example keywords in the supported languages.
It is a demo rule, not a complete classifier. Add both positive business examples and negative
advice requests when extending it. Do not describe its keyword coverage as general safety.
