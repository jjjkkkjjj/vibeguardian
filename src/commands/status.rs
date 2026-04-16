use anyhow::Result;

use crate::config::project::ProjectConfig;

pub fn execute() -> Result<()> {
    let config = ProjectConfig::load()?;

    println!("Project: {}", config.project.name);
    println!("Default profile: {}", config.project.default_profile);

    println!("\n── Environment Profiles ─────────────────────────────────────");
    if config.env.is_empty() {
        println!("  (none)");
    }
    for (profile, vars) in &config.env {
        println!("  [{}]", profile);
        for (key, raw) in vars {
            // Show key names but mask any actual values; show scope label
            if raw.starts_with("secret://project/") {
                println!("    {} = {} → ***[secret:project]***", key, raw);
            } else if raw.starts_with("secret://") {
                println!("    {} = {} → ***[secret:global]***", key, raw);
            } else {
                println!("    {} = {}", key, raw);
            }
        }
    }

    println!("\n── Proxy Routes (port {}) ──────────────────────────────────", config.proxy.port);
    if config.proxy.routes.is_empty() {
        println!("  (none)");
    }
    for route in &config.proxy.routes {
        println!("  {} → {}", route.path, route.target);
        for header in route.inject_headers.keys() {
            println!("    inject header: {} = ***[secret]***", header);
        }
    }

    Ok(())
}
