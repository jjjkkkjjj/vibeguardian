use anyhow::{bail, Context, Result};
use serde_json::Value;

use super::secrets;

const SECRET_SCHEME: &str = "secret://";
const PROJECT_SCOPE: &str = "project/";

/// Holds both the global and optional project-scoped secrets stores.
pub struct SecretStores {
    pub global: Value,
    pub project: Option<Value>,
}

/// Resolve a single value: if it starts with `secret://`, look it up in the
/// appropriate store; otherwise return it as-is.
///
/// - `secret://project/<path>` → project-scoped store (`~/.vibeguard/projects/<name>/secrets.json`)
/// - `secret://<path>`         → global store (`~/.vibeguard/secrets.json`) (backward-compatible)
pub fn resolve_value(raw: &str, stores: &SecretStores) -> Result<String> {
    if let Some(after_scheme) = raw.strip_prefix(SECRET_SCHEME) {
        if let Some(project_path) = after_scheme.strip_prefix(PROJECT_SCOPE) {
            let project = stores.project.as_ref().context(
                "secret://project/... reference requires a project store; \
                 ensure vibeguard.toml is present in the current directory",
            )?;
            secrets::resolve(project, project_path)
        } else {
            secrets::resolve(&stores.global, after_scheme)
        }
    } else {
        Ok(raw.to_string())
    }
}

/// Expand a template string that may contain `${secret://...}` placeholders.
/// e.g. `"Bearer ${secret://global/stripe/secret_key}"` → `"Bearer sk_live_..."`
pub fn expand_template(template: &str, stores: &SecretStores) -> Result<String> {
    let mut result = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(start) = remaining.find("${") {
        let end = remaining[start..].find('}').map(|i| start + i);
        match end {
            Some(end_idx) => {
                result.push_str(&remaining[..start]);
                let placeholder = &remaining[start + 2..end_idx]; // contents inside ${ }
                if placeholder.starts_with(SECRET_SCHEME) {
                    let resolved = resolve_value(placeholder, stores)?;
                    result.push_str(&resolved);
                } else {
                    // Not a secret reference — keep the placeholder verbatim
                    result.push_str(&remaining[start..=end_idx]);
                }
                remaining = &remaining[end_idx + 1..];
            }
            None => {
                bail!("Unclosed '${{' in template: {}", template);
            }
        }
    }
    result.push_str(remaining);
    Ok(result)
}

