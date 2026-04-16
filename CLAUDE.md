# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

vibeguardian (`vg`) is a Rust CLI tool that protects API secrets during development by isolating them from AI assistants and processes. It wraps development workflows via three mechanisms: env-var injection, local reverse proxy with header injection, and real-time log masking.

## Before writing any code

Read `docs/design.md` and `docs/cli.md` first. These are the authoritative spec. Propose a plan and wait for approval before implementing.

## Stack

- **Language:** Rust
- **CLI parsing:** `clap` (subcommands + `TrailingVarArg` for `vg run -- <cmd>`)
- **Async runtime:** `tokio`
- **HTTP proxy:** `axum` or `hyper` + `reqwest`
- **Log masking:** `aho-corasick` (O(n) multi-string matching)
- **Serialization:** `serde`, `serde_json`

## Commands

```
cargo build          # build
cargo test           # test
cargo clippy         # lint
rustfmt src/**/*.rs  # format
```

## Configuration

- `vibeguard.toml` — project-level config (safe for Git; uses `secret://` references, never actual values)
- `~/.vibeguard/secrets.json` — global secret store (never committed, never in project dir)
