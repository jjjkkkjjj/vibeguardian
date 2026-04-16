use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vg", about = "Protect API secrets during VibeCoding")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a command under Vibeguard (inject env vars, start proxy, mask logs)
    Run(RunArgs),
    /// Generate a vibeguard.toml template in the current directory
    Init,
    /// Store a secret in ~/.vibeguard/secrets.json
    Set(SetArgs),
    /// Show proxy status and injected key names for the current project
    Status,
}

#[derive(Args)]
pub struct RunArgs {
    /// Environment profile to use (default: "dev")
    #[arg(short, long, default_value = "dev")]
    pub profile: String,

    /// Disable log masking (not recommended)
    #[arg(long)]
    pub no_mask: bool,

    /// Disable the local reverse proxy
    #[arg(long)]
    pub no_proxy: bool,

    /// Command to run (e.g. npm run dev)
    #[arg(trailing_var_arg = true, required = true)]
    pub command: Vec<String>,
}

#[derive(Args)]
pub struct SetArgs {
    /// Secret path (e.g. stripe/secret_key)
    pub path: String,

    /// Secret value — omit to be prompted interactively (no echo)
    pub value: Option<String>,

    /// Store the secret in the project-scoped store instead of the global store.
    /// Reads the project name from vibeguard.toml in the current directory.
    #[arg(long)]
    pub project: bool,
}
