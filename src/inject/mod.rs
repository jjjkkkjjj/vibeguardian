use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::config::{
    project::{ProjectConfig, ProxyRoute},
    resolver,
};

/// Resolve all environment variable values for the given profile.
/// Values starting with `secret://` are looked up in the secrets store;
/// all other values are returned as-is.
pub fn resolve_env(
    config: &ProjectConfig,
    store: &Value,
    profile: &str,
) -> Result<HashMap<String, String>> {
    let profile_env = config.env.get(profile).cloned().unwrap_or_default();
    let mut resolved = HashMap::new();
    for (key, raw_value) in &profile_env {
        let value = resolver::resolve_value(raw_value, store)
            .with_context(|| format!("Failed to resolve env var '{}'", key))?;
        resolved.insert(key.clone(), value);
    }
    Ok(resolved)
}

/// Gather all resolved secret values (env vars + proxy inject headers) so the
/// log masker can redact them from child-process output.
pub fn collect_secrets(
    resolved_env: &HashMap<String, String>,
    routes: &[ProxyRoute],
    store: &Value,
) -> Vec<String> {
    let mut secrets: Vec<String> = resolved_env.values().cloned().collect();
    for route in routes {
        for raw_header in route.inject_headers.values() {
            if let Ok(v) = resolver::expand_template(raw_header, store) {
                secrets.push(v);
            }
        }
    }
    secrets
}
