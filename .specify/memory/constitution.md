<!--
Sync Impact Report
Version: 2.0.0 → 2.0.1 (PATCH: clarify native GPUI architecture and verification)
Modified: I and IV (native accessibility and flow verification), technical constraints, paths
Affected: spec 005, AGENTS.md, README.md, live-agent docs and project verification skills
Templates: plan-template.md reads this file at run time; no template edits needed
Follow-ups: meet the existing voice requirement before connecting real marketplace services
-->

# Agely Constitution

## Core Principles

### I. Older Adults First
Agely is built for people over 65 before anyone else.
- The expert MUST never be required to fill in a form; every expert-side input MUST be reachable
  by talking to an agent (voice), with typing offered only as an alternative.
- The browser-only marketplace try-out in spec 004 MAY require typed sign-in and display-name
  fields and use suggested replies. This exception applies only while all marketplace services
  are stand-ins. The voice requirement applies before any real marketplace service is connected.
- Text MUST NOT render below 16px (`--text-xs`); primary controls MUST be at least 56px tall
  (`--control-h`); pinch zoom MUST NOT be disabled.
- Copy MUST use plain language, one decision per screen, and visible keyboard focus.
- Meaning MUST NOT be carried by colour or animation alone.
- These rules are tested, not only stated: the Playwright suite MUST check the 16px floor, focus
  rings and WCAG 2.1 AA (axe) on every chapter of the browser app. Native screens MUST verify the
  same size and focus requirements, keyboard operation and accessible names in the actual GPUI
  window. A new screen joins its platform's checks in the same change.

Rationale: almost no technology is designed for older people; Agely's supply side is them.

### II. People Decide, Agents Propose
Agents draft, suggest, match, explain and prepare. Humans confirm.
- Agents MUST NOT publish a profile, accept a match, book a session, or move money on anyone's
  behalf. Each of those MUST be an explicit, separately labelled human action.
- Every agent suggestion (skill profile, price range, match) MUST show why it was suggested and
  MUST be editable before it is confirmed.

Rationale: trust from older experts and buyers depends on staying in control.

### III. Honest by Default
- Scripted or simulated data MUST be visibly labelled as a demo wherever it appears.
- Integrations that are not connected (Stripe, Google Meet/Zoom, Supabase, voice agents) MUST
  NOT be presented as live; mocks MUST sit behind an interface that a real adapter can replace.
- Agely MUST NOT offer legal, medical or financial advice; briefs in those domains MUST be
  declined with a clear explanation.
- Identity and credentials are self-declared at launch and MUST be shown as such.
- Numbers Agely publishes about itself follow the same rule. A figure that is modelled (the
  simulator) or estimated (the cost ledger) MUST say so where it appears, and an estimate MUST
  state its method. Money paid and money estimated MUST be distinguishable.

Rationale: the product's value rests on credible expertise and credible claims.

### IV. Test-First, Verified Before Done (NON-NEGOTIABLE)
- Domain logic (matching, price-range overlap, flow state) MUST have unit tests written before
  or with the implementation, and they MUST be seen failing first.
- Each browser user-facing flow MUST have at least one Playwright journey. Optional live-agent
  states use the browser agent fixture. Each native user-facing flow MUST have Rust state tests
  and a journey through the actual GPUI window, including keyboard and accessibility checks.
- Work is done only when `tsc`, Vitest, the production build, Playwright, `cargo fmt`,
  `cargo clippy -D warnings` and `cargo test` all pass. Generated native assets MUST be current,
  and a native change MUST pass the native window checks above.
- Website pages in `site/` have no Playwright journeys. Logic there (the simulator's model, the
  ledger's rules) MUST have unit tests, and a changed page MUST be opened in a browser at desktop
  and phone width before it is called done.
- Report only what ran. A pull request says what was tested and what could not be run; a check
  that was skipped is named, not implied.

Rationale: a demo that breaks on stage costs more than the time tests take.

### V. Simple Until Proven Otherwise
- Prefer pure domain functions and platform state over new dependencies or services.
- No backend, database or network call is added until a spec requires it.
- `src/ui/tokens.css` is the single source of visual truth. Browser components MUST use its
  tokens, and native components MUST use the values generated from the same file.

Rationale: a prototype must stay easy to change while the product is found.

### VI. Owned and Attributed
- Every asset the project ships (fonts, icons, images, video, sample data) MUST be original or
  openly licensed, and MUST be listed in `ATTRIBUTION.md` in the change that adds it.
- A new dependency's licence MUST be on the allowlist in `desktop/deny.toml` (Rust) or be a
  permissive licence of the same kind (JavaScript). CI checks the Rust side.
- Content made with a generative model MUST be recorded as such in `ATTRIBUTION.md`, with the
  terms of the service that made it.
- The repository is MIT licensed. The Ajentik logo is the owner's mark and is not covered.

Rationale: a product about credible expertise cannot be careless about whose work it uses.

## Product & Technical Constraints

- Desktop app: native GPUI and Rust, currently targeting Apple Silicon Macs. Shared story and
  localization are generated as embedded offline JSON; the native theme reads the embedded
  shared CSS tokens. Optional local Codex
  operations run on background tasks.
- Browser app: React 19 + TypeScript (strict) + Vite, managed with Bun. Browser journeys and the
  marketplace try-out remain available separately from the native walkthrough.
- Website: static pages in `site/`, served at agely.dev with the app at `/demo/`. Plain HTML, CSS
  and JavaScript modules; no framework and no build step of its own.
- Target users: buyers in the United States; experts from any country.
- Future integrations (out of scope until specified): Stripe for held payments, Google Meet or
  Zoom for sessions, Supabase for data, a voice agent for interviews.
- Sessions are sold in 15-minute units; prices are expressed per 15 minutes in USD.
- Headline success metric: dollars earned by people over 65.

## Development Workflow

- Every feature of the app (`src/`, `desktop/`) follows Spec Kit: specify → clarify → plan →
  tasks → analyze → implement. Artifacts live in `specs/<feature>/`.
- Website pages in `site/` do not need a spec. Their pull request MUST say what the page is for
  and how it was checked.
- `AGENTS.md` holds the working rules for people and coding agents and MUST stay consistent with
  this file. Where they disagree, this file wins and `AGENTS.md` is corrected.
- A check that looks removable (a test, a lint rule, a CI gate) MUST say what it prevents, and
  MUST NOT be weakened to make a change pass.
- Work happens on a feature branch and lands through a pull request. CI (`ci.yml`) MUST be
  green before merge; reviewer approval is not required for this repository.
- Versions in `package.json` and `desktop/Cargo.toml` move in lockstep. The native package entry
  in `desktop/Cargo.lock` is updated with them.

## Governance

This constitution supersedes other practices in this repository. Amendments are made through a
pull request that updates this file, states the version bump and its reason, and updates any
affected specs or plans. Versioning follows semantic versioning: MAJOR for removed or redefined
principles, MINOR for new principles or sections, PATCH for clarifications. Every plan MUST
include a Constitution Check against the principles above, and any justified violation MUST be
recorded in the plan's Complexity Tracking table.

**Version**: 2.0.1 | **Ratified**: 2026-11-11 | **Last Amended**: 2026-11-21
