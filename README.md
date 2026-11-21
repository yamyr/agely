# Agely

**Experience shouldn't retire.**

Agely is an expert network where older adults turn decades of experience into paid, 15-minute sessions. Our longer-term goal is an AI engine each expert owns: a growing collection of their approved questions, checks, and workflows.

This hackathon prototype follows Margaret, a retired procurement director, and Dev, a hardware founder, from their first conversations to a completed session and Margaret's first saved workflow.

| Try Agely | What opens |
| --- | --- |
| [Browser walkthrough](https://agely.dev/demo/) | The scripted Margaret and Dev story, with no account needed. |
| [Marketplace try-out](https://agely.dev/app) | Both roles in one browser, using simulated services. |
| [Website and demo video](https://agely.dev/) | An introduction to the project. |
| [Market simulator](https://agely.dev/simulator/) | A fictional market with adjustable assumptions. |
| [Published Mac downloads](https://agely.dev/releases/) | Release history and Apple silicon installers. |

The native Rust/GPUI rewrite is an **unreleased developer preview**. The published v0.1.3
installer still uses Tauri. To try GPUI, build this branch using the [native desktop guide](docs/desktop.md).
See the [changelog](CHANGELOG.md) for released and unreleased changes.

## The problem

Retirement does not erase the judgment someone has built over a career. We want to give older adults a way to keep contributing and earning without having to build a website, write a sales profile, or navigate a complicated marketplace.

On the other side, a founder facing an unfamiliar decision may need a short conversation with someone who has handled that problem before.

## Our approach

An agent helps an expert describe what they know, draft a profile, and choose a price. A buyer describes a problem, reviews a brief, and sees why an expert fits. Sessions are priced in 15-minute units.

After a session, a debrief turns the expert's approach into a proposed workflow. The expert reviews it before adding it to their engine. The aim is to preserve their working knowledge while they remain in control of what gets shared.

People make the decisions. Agents propose profiles, prices, matches, and workflows; people confirm publication, booking, and approval.

## Try the demo

The walkthrough is available on `main`. It uses fictional people and scripted conversations, so you can explore the full story without accounts, API keys, or external services.

1. **Margaret onboards.** Margaret Okafor, 68, describes 34 years in procurement and supplier management. Review her draft profile, edit a competency, and publish it.
2. **Margaret sets her price.** Review the suggested $60–$90 per 15 minutes and its reasons, then confirm or change the range.
3. **Dev describes his problem.** Dev Patel is preparing for his first contract-manufacturing agreement. Review his brief and $50–$100 budget per 15 minutes.
4. **Agely explains the match.** See which of Margaret's competencies fit and where their price ranges overlap. Other experts appear with reasons they did not match.
5. **They agree on a session.** Dev proposes 30 minutes at $75 per 15 minutes, or $150 total. Margaret accepts, and the demo shows a booking and simulated payment hold.
6. **Margaret's engine grows.** Mark the session as held, walk through the debrief, and approve a reusable workflow. The engine shows one session, one workflow, and $150 in simulated earnings.

Use the chapter list to jump to any part of the story. **Back** preserves your progress, and **Restart** returns to the beginning.

The [presenter guide](specs/001-demo-walkthrough/quickstart.md) includes a full dry run and validation steps.

The native desktop app can use an installed, signed-in [Codex CLI](https://github.com/openai/codex)
for Margaret's interview, Dev's brief chat and Margaret's debrief. It offers suggested replies
and free text, with retry and a scripted fallback. Browser conversations stay scripted.
The optional live mode uses your ChatGPT plan; no OpenAI API key is needed.
See the [live agent guide](docs/live-agent.md) for setup, privacy and offline testing.

## What works today

| Part | Prototype behavior |
| --- | --- |
| Expert onboarding | Scripted interview, editable profile, explicit publication, and configurable price range. |
| Buyer brief | Editable problem, session length, and budget, with confirmation before matching. |
| Matching | Client-side competency matching and price-overlap calculations, with explanations and no-match states. |
| Booking | Proposal validation, acceptance or decline, and a session summary. |
| Expert engine | Scripted debrief, workflow approval, and session and earnings summaries. |
| Navigation | Chapter jumps, back, restart, and validation of invalid prices and proposals. |
| Live agent (optional) | The three conversations can be generated live through the local Codex CLI, using its ChatGPT sign-in and no API key. Each turn offers three suggested replies, with retry and a scripted fallback. Drafts that follow stay scripted. |

Conversations stand in for a future voice interview; there is no microphone. Conversations are scripted unless the optional live agent is available, and the profile, brief and workflow drafts are always scripted. Payments, meeting links, and earnings are simulated; no money moves and no meeting is created. Identity and credentials are self-declared. Demo edits stay in memory and reset when the app restarts.

## The try-out product

Open the browser marketplace try-out at [agely.dev/app](https://agely.dev/app) or `#/app` when running the browser build locally. You can try both roles from sign-in to a simulated payment release.

1. An expert signs in, is interviewed, publishes a profile and confirms a price.
2. A buyer signs in, describes a problem, confirms a brief and is matched with the experts who are really published.
3. The buyer proposes a session; the expert accepts; the payment is held and both get a meeting link and a calendar file.
4. After the session the expert marks it as held and sees what they are owed.

Nothing outside the browser is connected yet. Sign-in, storage, the drafting agent, payments and meetings are stand-ins, each labelled on screen, and every page says what this version can't do: data stays in one browser, nothing is sent to the other person, anyone can sign in as any address on that device, and no money moves. Each stand-in sits behind an interface with a shared test suite, so a real service can replace it without changing the screens or the rules. See [spec 004](specs/004-marketplace-foundation/spec.md).

## Designed for older adults

The try-out interview offers suggested replies and optional free text. Experts type their sign-in details and display name. Shared design tokens set a minimum text size of 16px and primary control height of 56px. The interface provides visible keyboard focus and names the person whose turn it is.

Drafts remain drafts until the person confirms them. Every match explains its reasoning, and scripted data stays visibly labelled. The demo also declines requests for legal, medical, or financial advice.

These principles are recorded in the [project constitution](.specify/memory/constitution.md).

## Built with

- GPUI and Rust for the native desktop walkthrough on Apple Silicon Macs.
- React 19, TypeScript, and Vite 8 for the browser walkthrough and marketplace try-out.
- Bun for dependency management and scripts.
- Vitest and Rust tests for logic, Playwright for browser journeys, and native UI checks for the desktop app.
- GitHub Actions for frontend, browser, workflow, and Rust checks.

The desktop app renders native controls without a WebView or JavaScript runtime. It embeds
the shared story and translations as offline JSON and reads the shared CSS design tokens.
The browser walkthrough and marketplace use React. The website in `site/` uses plain HTML,
CSS and JavaScript modules.

The [native desktop guide](docs/desktop.md) explains the architecture, generated assets,
packaging and migration limits. [Spec 005](specs/005-gpui-desktop/spec.md) records the rewrite's requirements.

The foundation was ported from `yademy/elderwise-hbs`.

## Languages

The website, browser apps and native walkthrough share English source messages and 17
translated catalogs. The translations are model-generated drafts awaiting native-speaker
review. Language selection, English fallback and draft disclosures remain visible in the UI.
Prices stay in USD when the language changes.

See [translation maintenance](CONTRIBUTING.md#maintain-translations) before changing messages
or locale formats. Native builds embed their translations and need no translation download.

## Run locally

Install [Bun](https://bun.sh). Version `1.3.10` matches `package.json` and CI.

```sh
git clone https://github.com/yamyr/agely.git
cd agely
bun install --frozen-lockfile
bun run dev
```

Open `http://localhost:1420` to run the walkthrough in your browser. Browser mode does not require the Rust toolchain.

For the native desktop walkthrough, use an Apple Silicon Mac, install Rust stable through
[rustup](https://rustup.rs), and install Xcode Command Line Tools with `xcode-select --install`.
Until the GPUI migration lands on `main`, switch to its preview branch, then run:

```sh
git switch codex/agely-gpui
bun run desktop
```

This generates the shared native assets and starts GPUI. The native app does not use a web
server. The marketplace try-out remains available in the browser.

## Validate changes

Run the frontend checks and production build:

```sh
bunx tsc --noEmit
bun run test
bun run check:i18n
bun run build
```

Install Chromium, typecheck the browser tests, and run them:

```sh
bunx playwright install chromium
bunx tsc -p e2e/tsconfig.json
bun run e2e
```

Playwright starts its own Vite server on port `1493` and uses the [browser agent fixture](e2e/previewAgentStub.ts). To choose another port, run `E2E_PORT=1494 bun run e2e`. Port `1420` is reserved for browser development.

Check the website separately:

```sh
bun run check:site
```

The browser and website builds both replace `dist/`. Build the one you intend to serve.

Run the native checks on macOS with the Rust toolchain and Xcode Command Line Tools:

```sh
bun scripts/export-desktop.ts --check
cargo fmt --manifest-path desktop/Cargo.toml --check
cargo clippy --locked --manifest-path desktop/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path desktop/Cargo.toml
cargo deny --manifest-path desktop/Cargo.toml check
```

Native flow changes also need a journey through the actual GPUI window, including keyboard
focus and accessibility checks. Browser fixtures do not exercise native rendering or the Codex
process runner.

[Full CI](.github/workflows/ci.yml) checks the frontend on Ubuntu and runs Chromium and Rust checks on macOS. [Fast feedback](.github/workflows/fast-feedback.yml) checks workflows, frontend types and unit tests, and Rust formatting. Wait for full CI to pass before merging.

## Build an installer

```sh
bun run desktop:build
```

The script generates native assets and builds the Apple Silicon release binary. It writes
`macos/Agely.app`, `dmg/Agely_<version>_aarch64.dmg` and `dmg/SHA256SUMS.txt` under
`desktop/target/aarch64-apple-darwin/release/bundle/`. This developer preview has an ad hoc
signature and no Apple notarization. Building it does not publish a release.

For a frontend-only production preview, run `bun run build` followed by `bun run preview`.

## Explore the repository

| Path | Purpose |
| --- | --- |
| `src/demo/` | Walkthrough screens, fictional personas, domain rules, state, and scripted services. |
| `src/ui/` | Shared design tokens and the React error boundary. |
| `src/lib/` | Browser agent fixture boundary and error handling. |
| `desktop/` | GPUI app, native walkthrough state, embedded assets, localization and local Codex runner. |
| `e2e/` | Playwright journeys, edge cases, layout checks, and the browser agent fixture. |
| `specs/001-demo-walkthrough/` | Demo specification, data model, contracts, tasks, and presenter guide. |
| `.specify/` | Spec Kit templates, scripts, and the project constitution. |
| `docs/tooling/` | Notes on the project's optional development tools. |
| `site/` | Static landing page with the demo video, served at agely.dev; the app is at `/demo/` (`bun run build:site`). |
| `site/costs/` | Open cost ledger at agely.dev/costs/. Record a cost by adding a line to `costs.json`; `scripts/ai-usage.mjs` totals AI tokens from local agent logs. |
| `src/product/` | The try-out product at `#/app`: `ports/` (the outside services as interfaces, with contract suites), `standins/` (in-browser adapters), `usecases/` (the rules), `screens/`. |
| `specs/004-marketplace-foundation/` | Specification, plan, contracts, tasks and verification for the try-out product. |
| `specs/005-gpui-desktop/` | Native migration specification, architecture, tasks and verification. |
| `docs/desktop.md` | GPUI setup, architecture, generated assets, packaging and known limits. |
| `docs/live-agent.md` | Optional native Codex setup, process boundaries and offline fixture. |
| `site/simulator/` | Marketplace simulator at agely.dev/simulator/: the demo's matching rule run over a made-up market, with adjustable parameters. `engine.js` is the model and has unit tests. |

Features follow Spec Kit: specify, clarify, plan, tasks, analyze, and implement. Use `src/ui/tokens.css` for shared visual values. Keep versions in `package.json` and `desktop/Cargo.toml` in sync; a unit test checks them.

## What's next

The three conversations can already run live through the optional Codex agent. The next step is to generate the profile, brief and workflow drafts live as well, still behind human confirmation and with a scripted fallback. Voice input, real payments and calendars, persistent storage, and expert-owned AI workflows beyond the demo need further implementation and validation.

The outcome we want to measure is dollars earned by people over 65.

## Licence

The code is under the [MIT License](LICENSE). The Ajentik logo is not covered by it. Fonts, the
demo video and other assets are listed with their terms in [ATTRIBUTION.md](ATTRIBUTION.md).
