use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;

fn secrets_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".vibeguard").join("secrets.json"))
}

/// Load the entire secrets store as a JSON Value (object at root).
pub fn load() -> Result<Value> {
    let path = secrets_path()?;
    if !path.exists() {
        return Ok(Value::Object(Default::default()));
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let val: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid JSON in {}", path.display()))?;
    Ok(val)
}

/// Write a secret at the given dot-separated path (e.g. "stripe/secret_key").
/// Creates intermediate objects as needed.
pub fn set(path: &str, secret: &str) -> Result<()> {
    let secrets_file = secrets_path()?;
    if let Some(parent) = secrets_file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let mut root = load()?;
    let parts: Vec<&str> = path.split('/').collect();

    insert_value(&mut root, &parts, secret)?;

    let serialized = serde_json::to_string_pretty(&root)?;
    // Restrict permissions: owner read/write only (0o600)
    write_secret_file(&secrets_file, &serialized)?;
    Ok(())
}

/// Navigate the JSON tree and set the leaf value.
fn insert_value(node: &mut Value, parts: &[&str], secret: &str) -> Result<()> {
    if parts.is_empty() {
        return Ok(());
    }
    let obj = node
        .as_object_mut()
        .context("Expected JSON object at node")?;

    if parts.len() == 1 {
        obj.insert(parts[0].to_string(), Value::String(secret.to_string()));
        return Ok(());
    }

    let child = obj
        .entry(parts[0].to_string())
        .or_insert_with(|| Value::Object(Default::default()));
    insert_value(child, &parts[1..], secret)
}

/// Load the project-scoped secrets store for the given project name.
/// Returns an empty object if the file does not exist.
pub fn load_project(project_name: &str) -> Result<Value> {
    let path = project_secrets_path(project_name)?;
    if !path.exists() {
        return Ok(Value::Object(Default::default()));
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let val: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid JSON in {}", path.display()))?;
    Ok(val)
}

/// Write a secret at the given slash-separated path into the project-scoped store.
/// e.g. set_project("my-app", "stripe/secret_key", "sk_live_...")
pub fn set_project(project_name: &str, path: &str, secret: &str) -> Result<()> {
    let secrets_file = project_secrets_path(project_name)?;
    if let Some(parent) = secrets_file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let mut root = load_project(project_name)?;
    let parts: Vec<&str> = path.split('/').collect();
    insert_value(&mut root, &parts, secret)?;

    let serialized = serde_json::to_string_pretty(&root)?;
    write_secret_file(&secrets_file, &serialized)?;
    Ok(())
}

fn project_secrets_path(project_name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home
        .join(".vibeguard")
        .join("projects")
        .join(project_name)
        .join("secrets.json"))
}

/// Resolve a key path (slash-separated) within the loaded secrets store.
/// e.g. "global/stripe/secret_key" navigates root["global"]["stripe"]["secret_key"]
pub fn resolve(store: &Value, path: &str) -> Result<String> {
    let mut current = store;
    for part in path.split('/') {
        current = current
            .get(part)
            .with_context(|| format!("Secret path segment '{}' not found in secrets.json", part))?;
    }
    current
        .as_str()
        .map(|s| s.to_string())
        .with_context(|| format!("Secret at path '{}' is not a string", path))
}

#[cfg(unix)]
fn write_secret_file(path: &std::path::Path, content: &str) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[cfg(not(unix))]
fn write_secret_file(path: &std::path::Path, content: &str) -> Result<()> {
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write {}", path.display()))
}

