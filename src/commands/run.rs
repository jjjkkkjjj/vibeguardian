use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::cli::RunArgs;
use crate::config::{project::ProjectConfig, secrets};
use crate::inject;
use crate::mask::LogMasker;

pub async fn execute(args: RunArgs) -> Result<()> {
    // ── 1. Load config and secrets ──────────────────────────────────────────
    let config = ProjectConfig::load()?;
    let store = secrets::load()?;

    // ── 2. Resolve env vars and collect secret values for masking ───────────
    let resolved_env = inject::resolve_env(&config, &store, &args.profile)?;
    let secret_values = inject::collect_secrets(&resolved_env, &config.proxy.routes, &store);
    let masker = LogMasker::new(&secret_values)?;

    // ── 4. Start proxy (unless --no-proxy) ──────────────────────────────────
    let _proxy_shutdown = if !args.no_proxy && !config.proxy.routes.is_empty() {
        Some(crate::proxy::start(config.proxy.port, config.proxy.routes, store).await?)
    } else {
        None
    };

    // ── 5. Spawn child process with injected env vars ────────────────────────
    let (program, cmd_args) = args.command.split_first().context("No command provided")?;
    let mut child = tokio::process::Command::new(program)
        .args(cmd_args)
        .envs(&resolved_env)
        .stdout(if args.no_mask {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .stderr(if args.no_mask {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .spawn()
        .with_context(|| format!("Failed to spawn '{}'", program))?;

    println!(
        "[Vibesafe] Injected {} env var(s) (profile: {})",
        resolved_env.len(),
        args.profile
    );
    if !args.no_mask {
        println!("[Vibesafe] Log masking enabled");
    }

    // ── 6. Stream stdout / stderr through the masker ─────────────────────────
    if !args.no_mask {
        let masker = std::sync::Arc::new(masker);

        if let Some(stdout) = child.stdout.take() {
            let m = masker.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    println!("{}", m.mask(&line));
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let m = masker.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("{}", m.mask(&line));
                }
            });
        }
    }

    // ── 7. Wait for child and propagate exit code ────────────────────────────
    let status = child.wait().await?;
    // _proxy_shutdown is dropped here, gracefully shutting down the proxy
    std::process::exit(status.code().unwrap_or(1));
}
