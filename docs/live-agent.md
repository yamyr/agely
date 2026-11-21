# Live agent setup

This guide describes the unreleased GPUI source tree. Package metadata remains at `0.1.3`.

The walkthrough's three conversations (Margaret's interview, Dev's brief chat, Margaret's debrief)
can be generated **live** instead of scripted. The app uses the **Codex CLI** already signed in on
your computer, with your ChatGPT account, so there is **no OpenAI API key** to create or store.

When the live agent is ready it switches on by itself. When it isn't, the scripted demo runs as
before, and the start screen tells you what's missing.

## Setup

Use an Apple Silicon Mac with Bun `1.3.10`, Rust stable and Xcode Command Line Tools.
See [local development setup](../README.md#run-locally).

1. **Install Codex** (needs Node.js):

   ```sh
   npm install -g @openai/codex
   ```

2. **Sign in with ChatGPT**:

   ```sh
   codex login
   ```

   Pick "Sign in with ChatGPT" in the browser window that opens. Check with `codex login status`.
   It should say `Logged in using ChatGPT`.

3. **Run Agely**:

   ```sh
   bun install --frozen-lockfile
   bun run desktop
   ```

   The start screen shows **"Live agent: checking…"**, then **"Live agent: ready"** with the
   model it found. Checks run in the background. Trying several models can take a few minutes;
   you can start the scripted walkthrough while the check runs.

Start the walkthrough. Each live turn offers three suggested replies. Pick one, or type your
own. Profile, brief and workflow drafts remain scripted and visibly labelled.

## Choosing the model

ChatGPT plans allow different models in Codex. If you don't choose one, Agely first tries the
`model` set in your own Codex config (`~/.codex/config.toml`), then `gpt-5.4-mini`, `gpt-5.5` and
`codex-auto-review`, and uses the first that answers. These are the app's configured candidates;
availability depends on your Codex account. Turns use **low** reasoning effort by default.
Set `AGELY_CODEX_EFFORT=medium` (or `minimal` / `high`) to change it.

To pick one yourself:

```sh
AGELY_CODEX_MODEL=gpt-5.5 bun run desktop
```

If Codex is installed somewhere unusual, point Agely at it:

```sh
AGELY_CODEX_BIN=/path/to/codex bun run desktop
```

## What it sends, and what it can touch

- Agely sends the conversation so far, fictional persona background and language instructions
  through the Codex CLI signed in on your computer.
- Each turn runs `codex exec` in an **empty temporary folder** that is deleted afterwards. It runs
  with:
  - a read-only sandbox;
  - your personal Codex config and rules ignored during execution, with only the model
    preference read beforehand;
  - apps, browser and computer-use turned off;
  - no saved session.

  The agent is told not to run commands or read files.
- Usage counts against your ChatGPT plan, like using Codex yourself.

## Troubleshooting

| Start screen says | What to do |
|-------------------|-----------|
| *Codex is not installed* | Step 1, then **Check again**. |
| *Codex is installed but not signed in* | `codex login`, then **Check again**. |
| *Codex could not check whether you are signed in* | Try `codex login status` in a terminal. If an older Codex rejects your settings, update it or restart Agely with `AGELY_CODEX_BIN` set to a working executable. Then select **Check again**. |
| *Your ChatGPT plan doesn't allow these models* | Set `AGELY_CODEX_MODEL` to a model your plan allows, restart the app, then **Check again**. |
| *The live agent check failed* | Check your internet connection, then **Check again**. |
| *Off in the browser preview* | Run the desktop app with `bun run desktop` to use the live agent. |

Each model probe has a 45-second timeout and each conversation turn has a 60-second timeout.
After a failed turn, choose **Try again** or **Use the scripted conversation**. After replying,
you can also choose **Finish conversation** when no turn is pending.

## Turning it off

Select **Turn live agent off** on the start screen. To launch with scripted conversations and
avoid a real Codex call, run `AGELY_CODEX_BIN=/nonexistent bun run desktop`.

## Implementation

- Native runner: `desktop/src/agent/`. GPUI calls `AgentState::status` and `AgentState::turn`
  on its background executor so Codex processes do not block the window.
- Native conversation state and UI integration: `desktop/src/live.rs` and `desktop/src/app.rs`.
- Shared briefs and browser preview UI: `src/demo/live/`. `bun run desktop:assets` exports the
  briefs and locale catalogues as JSON embedded in the native app.
- Specs: `specs/002-live-codex-agent/` and `specs/005-gpui-desktop/`.
- Browser Playwright tests use `e2e/previewAgentStub.ts` to exercise conversation states through
  `window.__AGELY_PREVIEW__`. The GPUI app calls Rust directly.
- An API-key provider can be added later behind the agent interface if a spec requires it.

## Developer validation

Run the Rust unit tests without contacting Codex:

```sh
cargo test --locked --manifest-path desktop/Cargo.toml
```

For a native window check without Codex credentials or service calls, use the silent offline
fixture in `scripts/fixtures/codex-gpui.py`. It needs Python 3. From the repository root:

```sh
agely_qa_dir=$(mktemp -d)
printf '%s\n' "$agely_qa_dir"
AGELY_QA_DIR="$agely_qa_dir" \
AGELY_CODEX_BIN="$PWD/scripts/fixtures/codex-gpui.py" \
AGELY_LOCALE=en \
bun run desktop
```

The fixture labels its replies as **Offline QA fixture**. It exercises native readiness,
process execution, conversation rendering and suggested replies. It does not validate Codex
authentication or service availability.

In another terminal, write a mode to the printed directory's `mode` file. The fixture reads
it at the start of each turn:

| Mode | Behavior to check |
| --- | --- |
| `ready` | A labelled response and three suggested replies. This is the default. |
| `error` | A failed turn, with retry and scripted fallback. |
| `delay` | A reply delayed three seconds, allowing restart or language change while it is pending. |

Check that restarting or confirming a language change discards a late reply. After testing,
quit the app and remove the disposable directory. No fixture step needs speakers or a microphone.

The authenticated test is separate and opt-in. It uses the installed, signed-in Codex CLI and
sends a fictional demo prompt through that account:

```sh
cargo test --locked --manifest-path desktop/Cargo.toml live_codex_check_and_one_turn -- --ignored --nocapture
```

Keep browser checks headless and muted. Voice tests must use fake capture devices rather than
speaker playback, and need separate approval before using production voice services.
