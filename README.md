# vibeguardian (`vg`)

> A CLI tool that protects API secrets from AI assistants during VibeCoding

Simply wrap your normal command with `vg run -- npm run dev` to create a development environment where AI assistants cannot physically access your API keys.

日本語ドキュメント: [README_ja.md](README_ja.md)

---

## How it works

| Feature | Description |
|---|---|
| **Inject Mode** | Loads secrets from `~/.vibeguard/secrets.json` and injects them only into child process memory — never written to disk or `.env`. |
| **Proxy Mode** | Runs a local reverse proxy (default `:8080`). Requests to `localhost:8080/proxy/stripe` are forwarded to the real API with the `Authorization` header injected invisibly. |
| **Log Mask Mode** | Hooks child process stdout/stderr in real time, replacing any secret value with `***[MASKED]***` before display. |

---

## Installation

### Homebrew (macOS)

```bash
brew tap jjjkkkjjj/vibeguardian
brew install vibeguardian
```

### apt (Ubuntu / Debian)

```bash
curl -LO https://github.com/jjjkkkjjj/vibeguardian/releases/latest/download/vs-x86_64-linux.deb
sudo dpkg -i vs-x86_64-linux.deb
```

### Download (manual)

Download the latest binary from [GitHub Releases](https://github.com/jjjkkkjjj/vibeguardian/releases) and place it in your `$PATH`.

```bash
# macOS (Apple Silicon)
curl -L https://github.com/jjjkkkjjj/vibeguardian/releases/latest/download/vs-aarch64-apple-darwin.tar.gz | tar xz
sudo mv vs /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/jjjkkkjjj/vibeguardian/releases/latest/download/vs-x86_64-apple-darwin.tar.gz | tar xz
sudo mv vs /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/jjjkkkjjj/vibeguardian/releases/latest/download/vs-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv vs /usr/local/bin/
```

### Build with cargo

```bash
cargo install --git https://github.com/jjjkkkjjj/vibeguardian vs
```

---

## Quick Start

```bash
# 1. Generate vibeguard.toml in your project
vg init

# 2. Store secrets globally (never in the project)
vg set stripe/secret_key              # hidden prompt
vg set openai/api_key sk_...          # arg mode (warns about shell history)

# 3. Run your app in a protected environment
vg run -- npm run dev
vg run --profile prod -- node server.js
```

Example terminal output:

```
[Vibeguard] Proxy started at http://localhost:8080
[Vibeguard] Injected 2 env var(s) (profile: dev)
[Vibeguard] Log masking enabled
> next dev
...
```

---

## `vibeguard.toml` — Configuration

Place at your project root. **Contains no actual keys — safe to commit to Git.**

```toml
[project]
name = "my-app"
default_profile = "dev"

# ── Inject Mode ──────────────────────────────────────────────────────────────
[env.dev]
DATABASE_URL        = "secret://global/supabase/dev_db_url"   # resolved from ~/.vibeguard/secrets.json
NEXT_PUBLIC_API_URL = "http://localhost:8080/proxy/api"       # plain text is fine for proxy URLs

[env.prod]
DATABASE_URL = "secret://global/supabase/prod_db_url"

# ── Proxy Mode ───────────────────────────────────────────────────────────────
[proxy]
port = 8080  # default

[[proxy.routes]]
path   = "/proxy/stripe"
target = "https://api.stripe.com"
inject_headers = { Authorization = "Bearer ${secret://global/stripe/secret_key}" }

[[proxy.routes]]
path   = "/proxy/openai"
target = "https://api.openai.com/v1"
inject_headers = { Authorization = "Bearer ${secret://global/openai/api_key}" }
```

The AI reads `vibeguard.toml`, understands that requests go to `localhost:8080/proxy/stripe`, and writes correct code — but it can never see the actual key behind `secret://`.

---

## Command Reference

### `vg run [OPTIONS] -- <CMD>`

| Flag | Description |
|---|---|
| `-p, --profile <PROFILE>` | Environment profile to use (default: `dev`) |
| `--no-mask` | Disable log masking (not recommended) |
| `--no-proxy` | Do not start the local proxy |
| `-- <CMD>` | Command to run (e.g. `npm run dev`) |

### `vg init`

Generates a `vibeguard.toml` template in the current directory. Fails if one already exists.

### `vg set <PATH> [VALUE]`

Stores a secret in `~/.vibeguard/secrets.json`.

- Omit `VALUE` for a hidden interactive prompt
- Passing `VALUE` as an argument warns about shell history exposure

```bash
vg set stripe/secret_key          # interactive
vg set stripe/secret_key sk_...   # direct (with warning)
```

### `vg status`

Reads `vibeguard.toml` and displays the list of env var names and proxy routes — values are always masked.

---

## Security Design

- `~/.vibeguard/secrets.json` is written with `0o600` permissions (owner read/write only)
- `vibeguard.toml` never contains real secret values
- Log masking uses Aho-Corasick for O(n) linear-time replacement — no performance impact on high-volume logs

---

## Development

```bash
# Build inside Docker
docker compose run --rm dev cargo build

# Run tests
docker compose run --rm dev cargo test

# Lint
docker compose run --rm dev cargo clippy -- -D warnings

# Shorthand from host
./cargo-docker build
./cargo-docker test
./cargo-docker clippy -- -D warnings
```

### Project Layout

```
src/
├── main.rs          Entry point
├── lib.rs           Library root (used by tests/)
├── cli.rs           Clap command definitions
├── inject/          Inject Mode — env var resolution helpers
├── mask/            Log Mask Mode — Aho-Corasick masker
├── proxy/           Proxy Mode — axum reverse proxy
├── config/          Config & secrets store parser
└── commands/        Subcommand implementations
tests/
├── mask.rs          LogMasker integration tests
└── config/
    ├── resolver.rs  secret:// resolution logic tests
    └── secrets.rs   Secrets store lookup tests
```

---

## License

MIT

