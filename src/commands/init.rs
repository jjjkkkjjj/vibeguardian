use anyhow::{bail, Result};

const TEMPLATE: &str = r#"[project]
name = "my-app"
default_profile = "dev"

# ── Inject Mode: environment variables ──────────────────────────────────────
# Values starting with secret:// are resolved from ~/.vibeguard/secrets.json
# All other values are injected as-is (safe for non-sensitive config).
[env.dev]
DATABASE_URL = "secret://global/supabase/dev_db_url"
NEXT_PUBLIC_API_URL = "http://localhost:8080/proxy/api"

[env.prod]
DATABASE_URL = "secret://global/supabase/prod_db_url"

# ── Proxy Mode: local reverse proxy ─────────────────────────────────────────
[proxy]
port = 8080

[[proxy.routes]]
path = "/proxy/stripe"
target = "https://api.stripe.com"
inject_headers = { Authorization = "Bearer ${secret://global/stripe/secret_key}" }

[[proxy.routes]]
path = "/proxy/openai"
target = "https://api.openai.com/v1"
inject_headers = { Authorization = "Bearer ${secret://global/openai/api_key}" }
"#;

pub fn execute() -> Result<()> {
    let path = std::path::Path::new("vibeguard.toml");
    if path.exists() {
        bail!("vibeguard.toml already exists in the current directory.");
    }
    std::fs::write(path, TEMPLATE)?;
    println!("[Vibeguard] Created vibeguard.toml — safe to commit to Git.");
    println!("            Store actual secrets with: vg set <path> [value]");
    Ok(())
}
