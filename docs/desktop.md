# Native desktop app

Agely's desktop walkthrough uses Rust and GPUI on Apple silicon Macs. Its 13 steps follow
Margaret and Dev through onboarding, matching, a simulated booking and a saved workflow.
The window renders through GPUI and Metal, without a WebView or JavaScript runtime.

This rewrite is an unreleased developer preview. Published v0.1.3 downloads still contain
the former Tauri app. The browser walkthrough and marketplace remain React applications;
the marketplace is not part of the native app.

## Run from source

Use an Apple silicon Mac with [Bun](https://bun.sh), Rust stable installed through
[rustup](https://rustup.rs), and Xcode Command Line Tools. Bun `1.3.10` matches CI.

From the repository root:

```sh
git switch codex/agely-gpui
bun install --frozen-lockfile
bun run desktop
```

The preview branch is needed while the migration has not landed on `main`.
`bun run desktop` exports the shared assets and starts the native binary with Cargo.
It does not start Vite or need a local web server. The first build compiles GPUI and its
dependencies. Subsequent builds reuse `desktop/target/`.

The scripted walkthrough works offline and needs no account. Optional live conversations
require an installed, signed-in Codex CLI and an internet connection. They run on background
tasks so the window remains responsive. See the [live agent guide](live-agent.md) for setup,
what leaves the computer and a silent offline CLI fixture.

## Where the code lives

| Path | Responsibility |
| --- | --- |
| `desktop/src/main.rs` | Window, menus, key bindings and platform startup. |
| `desktop/src/app.rs` | UI state, navigation, focus, language confirmation and background task results. |
| `desktop/src/model.rs` | Walkthrough decisions, validation, matching, prices and derived summaries. |
| `desktop/src/screens.rs` | The 13 native walkthrough screens. |
| `desktop/src/input.rs` | Native text editing, clipboard, grapheme navigation and IME composition. |
| `desktop/src/widgets.rs` | Shared controls and focused-control scrolling. |
| `desktop/src/theme.rs` | Values parsed from the embedded shared CSS tokens. |
| `desktop/src/i18n.rs` | Embedded translations, locale formatting and the saved language preference. |
| `desktop/src/live.rs` | Conversation state and request IDs that reject stale responses. |
| `desktop/src/agent/` | Bounded local Codex process runner. |

The view owns editable fields and focus. The model validates decisions and computes results.
A late Codex reply cannot update a conversation after a restart or language change.
Publishing a profile, sending a proposal, accepting it and releasing a simulated payment
remain explicit actions.

GPUI is pinned to official Zed revision
[`0a514d7ef59c6357878a244fdf8365d03c504767`](https://github.com/zed-industries/zed/tree/0a514d7ef59c6357878a244fdf8365d03c504767/crates/gpui).
That revision provides the accessibility APIs used here. The Rust dependency policy requires
exact Git revisions and retains the licence and advisory checks in `desktop/deny.toml`.

## Shared content and translations

Run `bun run desktop:assets` after changing shared content. The exporter writes these files:

| Generated file in `desktop/assets/` | Authored source |
| --- | --- |
| `content.json` | Personas and story in `src/demo/script/`. |
| `catalogs.json` | English fallback and 17 draft catalogs in `site/i18n/catalogs/`. |
| `live_briefs.json` | Conversation instructions in `src/demo/live/briefs.ts`. |
| `formatting.json` | Reviewed locale definitions in `scripts/desktop-formatting.json`. |

Commit updated generated files with their source changes. Check freshness without rewriting:

```sh
bun scripts/export-desktop.ts --check
bun run check:i18n
```

Normal exports use reviewed locale formats, independent of the host's Intl or ICU version.
To propose new format definitions, run `bun scripts/generate-desktop-formatting.ts` to inspect
the candidate on stdout. Add `--write` only when you intend to replace the canonical input,
then review its differences and regenerate the native assets.

The theme directly embeds `src/ui/tokens.css`; the JSON exporter does not generate it.
Keep text size, spacing, colours and control heights in that shared file. Native code must
respect the 16px text minimum, 56px primary controls and visible keyboard focus.

## State and keyboard use

- Tab and Shift-Tab move through controls. Focused controls scroll into view.
- Enter and Space activate focused buttons. Escape dismisses language choices or confirmation.
- Command-Q quits the application.
- Back keeps walkthrough edits. Restart clears story progress.
- Changing language asks before restarting the story. Cancelling keeps the current edits.

Story edits remain in memory. Only the chosen language persists, in
`~/Library/Application Support/com.agely.app/locale.json`. `AGELY_LOCALE=ja bun run desktop`
overrides the startup language for testing. Remove the override to use the saved preference.
The native app does not import the former WebKit language storage, so people upgrading from
Tauri may need to choose their language once.

## Verify a change

Run from the repository root:

```sh
bun scripts/export-desktop.ts --check
cargo fmt --manifest-path desktop/Cargo.toml --check
cargo clippy --locked --manifest-path desktop/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path desktop/Cargo.toml
cargo deny --manifest-path desktop/Cargo.toml check
```

Install `cargo-deny` if needed with `cargo install --locked cargo-deny`. The real authenticated
Codex test is opt-in; normal Rust tests do not call that service.

UI changes also need a journey through the actual GPUI window. Check keyboard focus,
accessible names, Unicode editing, long translations and a reduced window size. Test delayed
Codex replies, restart, failure, retry and scripted fallback with the offline fixture.
Keep QA silent. Browser fixtures cover React behavior and do not verify native rendering.

Shared-source changes also need the browser and website checks in the
[README](../README.md#validate-changes). The migration's
[verification record](../specs/005-gpui-desktop/verification.md) lists the checks actually run
and their limits, including the authenticated Codex test that was not rerun.

## Build a Mac installer

```sh
bun run desktop:build
```

The script exports assets, compiles the locked Apple silicon release target and creates an
application bundle with its icon, licences and attribution. It checks the plist, architecture,
ad hoc signature and disk image, then writes a SHA-256 checksum.

Outputs are under `desktop/target/aarch64-apple-darwin/release/bundle/`:

- `macos/Agely.app`
- `dmg/Agely_<version>_aarch64.dmg`
- `dmg/SHA256SUMS.txt`

The package declares macOS 11 as its deployment minimum. Runtime QA has been performed on the
development Mac, not every macOS version. There is no Intel, Windows or Linux installer.
The preview is ad hoc signed and has no Apple notarization.

Building an installer does not publish it. Versions in `package.json` and `desktop/Cargo.toml`
move together, including the package entry in `desktop/Cargo.lock`. A release requires its own
explicit approval and release notes. Merging to `main` publishes the website through Cloudflare
Pages, so a merge is also a publishing action.
