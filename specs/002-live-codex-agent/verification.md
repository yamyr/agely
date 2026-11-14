# Live Codex verification

## Agely 0.1.0 on 2026-11-14

The authenticated check used the release source at `6ac985d` on Apple Silicon macOS, with the
local Codex CLI already signed in through ChatGPT. The test used the fictional demo persona.

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml \
  agent::tests::live_codex_check_and_one_turn -- --exact --ignored --nocapture
```

One test passed. The readiness check selected `codex-auto-review`. The first English interview
turn returned three suggestions in 4.89 seconds. Total test runtime including readiness was
14.02 seconds.

### Packaged desktop walkthrough

The published Apple Silicon build reached the live-ready state with `codex-auto-review`. The
**Check again** action reached ready in 9.10 seconds. Suggested replies and a typed reply
advanced the interview. **Finish conversation** led to the labelled scripted profile draft.
When asked a fictional legal question in Dev’s brief, the agent declined and redirected to
commercial concerns. A French interview opening and follow-up both displayed French responses
and three suggestions. English was restored after the check.

Twenty open turns displayed an answer and exactly three suggested replies within 30 seconds,
meeting SC-002. Nineteen measured from the triggering action to the first observation of the
rendered response in 4.49–6.16 seconds. One typed turn took at most 12.45 seconds, including
delay between automation calls. Four other responses ended the conversation in 4.99–7.87
seconds. These rendered the closing message with reply controls hidden and were excluded from
the 20 open turns. Three additional observations were excluded because timing began after
dispatch or polling was interrupted. No Codex retry or timeout occurred.

| Scenario | Open-turn times in seconds |
| --- | --- |
| Typed interview reply | ≤12.45 |
| Dev’s brief | 4.78, 5.16, 5.17, 5.24 |
| Interview with suggested replies | 5.54, 5.96, 5.01 |
| Interview with explicit finish | 5.51, 5.26 |
| Margaret’s debrief | 4.49, 5.66, 5.70, 5.75 |
| French interview | 5.68, 5.45 |
| Final English interview | 5.25, 5.26, 6.16, 5.71 |

Two generated details drifted from the fictional scenario. One suggested reply swapped who
owned the tooling, and one paraphrase swapped who received the shipments. This run establishes
the checked interactions and timings. The generated dialogue still needs review for fidelity
to the scenario.
