use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    #[serde(default)]
    pub env: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub proxy: ProxySection,
}

#[derive(Debug, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default = "default_profile")]
    pub default_profile: String,
}

fn default_profile() -> String {
    "dev".to_string()
}

#[derive(Debug, Default, Deserialize)]
pub struct ProxySection {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub routes: Vec<ProxyRoute>,
}

fn default_port() -> u16 {
    8080
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyRoute {
    pub path: String,
    pub target: String,
    #[serde(default)]
    pub inject_headers: HashMap<String, String>,
}

impl ProjectConfig {
    pub fn load() -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string("vibeguard.toml").map_err(|_| {
            anyhow::anyhow!("vibeguard.toml not found in current directory. Run `vg init` first.")
        })?;
        let config: ProjectConfig = toml::from_str(&raw)?;
        Ok(config)
    }
}
