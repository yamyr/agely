# Changelog

## 0.1.3 — 2026-11-19

### Website

- Put the interactive demo, marketplace, and desktop downloads beside a shorter introduction. Added links to skip the header and jump to the story.
- Gave story text more room on phones, restored expense dates in the mobile cost ledger, and made both cost tables accessible by keyboard. In-page links move focus without reloading and links between pages keep the selected language.
- Consolidated navigation and action link sizing, improved long-heading wrapping, and added draft translations for the new navigation and release notes in all 17 supported languages.

### Desktop app

- Published an updated Apple silicon macOS walkthrough installer. This developer preview uses an ad hoc signature and has no Apple notarization.

## 0.1.2 — 2026-11-15

### Website

- Prevented repeated clicks from creating duplicate proposals or sessions in a marketplace tab. Only one proposal can be accepted for the same expert when the proposed times overlap.
- Kept keyboard focus on the action button after a failed send, withdrawal, acceptance, or decline so people can retry. Added draft translations for the withdrawal and decline errors in all 17 supported languages.

### Desktop app

- Published an updated Apple silicon macOS installer for the walkthrough. The developer preview uses an ad hoc signature and has no Apple notarization.

## 0.1.1 — 2026-11-15

### Website

- Kept the buyer’s latest time choice, rejected times skipped by a clock change, and brought the booking result into view after sending or withdrawing a proposal.
- Improved keyboard focus in the marketplace and simulator. People can request a replacement for an expired sign-in code without re-entering their email address.
- Added a localized link to the marketplace and automatic English captions to the demo video.
- Reduced the translation download to one request, kept the previous catalog URLs available, and sped up website checks while preserving coverage.

### Desktop app

- Clarified who received shipments and who owned tooling in the optional live Codex interview. Added specific guidance when Codex cannot check sign-in.
- Updated the Apple silicon macOS installer. This developer preview uses an ad hoc signature and has no Apple notarization.

## 0.1.0 — 2026-11-14

### Website

- Added the [marketplace try-out](https://agely.dev/app). In one browser you can try both roles, publish an expert profile, confirm a price, review a buyer brief, see why a match fits, book a session, and download a calendar invitation. Sign-in, meetings, and payments are simulated. The try-out keeps data in the current browser and sends no invitations or money.
- Extended English and 17 draft translations to marketplace screens. Improved language selection, localized price and duration entry, and recovery when the browser blocks storage. The simulator keeps its selected week when you change languages.
- Added a [releases page](https://agely.dev/releases/) with desktop downloads and version history.

### Desktop app

- Published the first Apple Silicon macOS installer. It includes the scripted Margaret and Dev walkthrough in English and 17 draft translations, with no account needed for the scripted experience.
- Kept optional live conversations through a locally installed and signed-in Codex CLI. This preview uses an ad hoc signature and Apple has not notarized it.

## 0.0.1 — Initial prototype

- Built the Tauri walkthrough for expert onboarding, pricing, a buyer brief, matching, booking, simulated earnings, and a reusable workflow. Added optional live Codex conversations with a scripted fallback.
- Launched the website with a demo video, a browser walkthrough, a market simulator, and a cost ledger that identifies estimates.
- Added English and 17 draft translations to the website and walkthrough. Added accessibility and browser checks, project governance, attribution, and the MIT licence.
- Used `0.0.1` as the source version. Desktop use was through a local build.
