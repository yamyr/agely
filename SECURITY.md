# Security

Agely is a prototype. Marketplace sign-in, payments and meetings are simulated, and marketplace
data stays in the current browser. The native walkthrough keeps story edits in memory and saves
only its chosen language locally. No money moves. The website at agely.dev is static.

The desktop app can run a local Codex CLI using your signed-in account for optional live
conversations. The [live agent guide](docs/live-agent.md#what-it-sends-and-what-it-can-touch)
describes what the app sends and the process restrictions.

## Reporting a problem

Please report a vulnerability privately through
[GitHub's private vulnerability reporting](https://github.com/yamyr/agely/security/advisories/new),
not in a public issue. Say what you found, how to reproduce it, and what it would let someone do.

This is a small project with no security team, so there is no guaranteed response time. Reports
are read, and a confirmed problem is fixed on `main` and credited to the reporter if they want.

## What is checked automatically

- Rust dependencies are checked for known vulnerabilities and disallowed licences on every pull
  request (`cargo deny`, configured in `desktop/deny.toml`). GPUI is allowed from the official
  Zed repository at the exact revision recorded in `desktop/Cargo.toml` and `desktop/Cargo.lock`.
  Other unknown git sources and registries are rejected.
- Dependabot opens updates for JavaScript, Rust and GitHub Actions dependencies weekly, and
  security updates as they are published.
