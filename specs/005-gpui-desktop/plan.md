# GPUI desktop implementation plan

The native desktop moves to `desktop/` in Rust. The React application remains the browser
entry point. The former Tauri package and JavaScript bridge are removed after their callers
and verification have moved.

## Technical context

GPUI and its macOS platform backend are pinned to official upstream revision
`0a514d7ef59c6357878a244fdf8365d03c504767`. That revision includes AccessKit semantics
absent from the published GPUI 0.2.2 snapshot. The git-source policy permits this
official repository and its pinned official font-kit fork; every Git dependency requires
an exact revision. Runtime Metal shaders avoid a
separate resource path in the packaged application.

Pure Rust state owns decisions and derived matching/session/engine values. A Bun exporter
reads the existing TypeScript story and locale sources into embedded JSON. Its check mode
rejects stale generated assets. The existing Rust Codex runner becomes blocking methods
called on GPUI's background executor.

The view owns text fields, focus, locale selection and generation-scoped asynchronous
requests. A window-close or restart cannot apply an obsolete live response. The native
bundle retains `com.agely.app` and reads its version from Cargo.

## Constitution check

All human decisions and simulation disclosures remain explicit. GPUI controls expose
AccessKit roles and labels with visible keyboard focus. Native behavior is verified with
Rust state tests and an actual packaged-app journey. Browser behavior remains covered by
Playwright. No real service, payment integration or backend is introduced.

## Verification

Run asset freshness, locale checks, TypeScript and existing browser/unit tests. Run Rust
format, Clippy, tests and dependency policy, then build the native application and inspect
its version, architecture, signature and startup. Exercise the 13-step journey, edits,
chapter navigation, translated UI, IME/clipboard and deterministic agent recovery.
