use anyhow::{bail, Result};
use serde_json::Value;

use super::secrets;

const SECRET_SCHEME: &str = "secret://";

/// Resolve a single value: if it starts with `secret://`, look it up;
/// otherwise return it as-is.
pub fn resolve_value(raw: &str, store: &Value) -> Result<String> {
    if let Some(path) = raw.strip_prefix(SECRET_SCHEME) {
        secrets::resolve(store, path)
    } else {
        Ok(raw.to_string())
    }
}

/// Expand a template string that may contain `${secret://...}` placeholders.
/// e.g. `"Bearer ${secret://global/stripe/secret_key}"` → `"Bearer sk_live_..."`
pub fn expand_template(template: &str, store: &Value) -> Result<String> {
    let mut result = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(start) = remaining.find("${") {
        let end = remaining[start..].find('}').map(|i| start + i);
        match end {
            Some(end_idx) => {
                result.push_str(&remaining[..start]);
                let placeholder = &remaining[start + 2..end_idx]; // contents inside ${ }
                if let Some(path) = placeholder.strip_prefix(SECRET_SCHEME) {
                    let resolved = secrets::resolve(store, path)?;
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

