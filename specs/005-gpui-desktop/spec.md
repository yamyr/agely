# Feature specification for the GPUI desktop migration

Created November 21, 2026. Branch `codex/agely-gpui`.
Requested by the owner with “Rewrite our app from tauri to gpui”.

## User scenarios and testing

### Story 1. Complete the native walkthrough

A person opens the macOS app, follows Margaret and Dev through all 13 existing steps,
edits drafts, and separately confirms publication, price, brief, booking and workflow.
Chapter navigation replays the scripted decisions before its destination. Restart clears
the run. Meetings, payments and earnings retain their simulation labels.

Acceptance is a complete native journey from welcome to one held session, simulated
earnings and one approved workflow, with tests covering invalid prices, restricted advice,
decline, repeated acceptance and chapter replay.

### Story 2. Read and operate the app in a supported language

English and the existing 17 draft languages are available offline. A language change asks
before restarting the walkthrough. The app retains the chosen language across launches.
Authored values, names, skill tags and user edits retain their meaning. Native text fields
support clipboard, selection, Unicode and IME input. Controls remain at least 56px high,
text at least 16px, and keyboard focus and accessibility labels are exposed.

Acceptance includes native keyboard navigation, non-Latin input and a translated journey.
Catalog integrity and generated asset freshness are checked automatically.

### Story 3. Use the optional local Codex agent

The desktop checks readiness in the background. A person can enable live interviews,
choose suggested replies or type, retry a failed turn, finish after replying, or select
the scripted fallback. Late results from a previous run or language cannot alter the
current conversation. Generated conversations continue to produce visibly scripted drafts.

Acceptance includes reuse of the existing process-runner tests and a deterministic fake
CLI fixture through the actual native entry point. A real authenticated turn is opt-in.

## Functional requirements

- Render the desktop UI directly with GPUI, with no Tauri or embedded browser dependency.
- Preserve the existing browser marketplace and scripted browser walkthrough.
- Reuse authored story data and translation catalogs through reproducible embedded assets.
- Reuse the bounded, read-only Codex CLI execution and output validation.
- Build and package an Apple silicon macOS application and disk image with version metadata.
- Preserve the existing signing disclosure and release history.
- Validate Rust, browser behavior, website rendering, dependency licensing and the native UI.

## Success criteria

The packaged GPUI app completes the existing walkthrough offline, exposes native accessible
controls, edits drafts correctly and handles live-agent failures without blocking the UI.
The website and browser test suite pass after removal of Tauri dependencies.
