use super::ProviderAdapter;
use crate::model::AccountProfile;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct CopilotAdapter;

impl CopilotAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn default_hosts_file() -> PathBuf {
        let base = if let Some(app_data) = std::env::var_os("LOCALAPPDATA") {
            PathBuf::from(app_data).join("github-copilot")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("github-copilot")
        };
        base.join("hosts.json")
    }
}

impl Default for CopilotAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for CopilotAdapter {
    fn id(&self) -> &'static str {
        "copilot"
    }

    fn display_name(&self) -> &'static str {
        "GitHub Copilot / Codex"
    }

    fn target_auth_path(&self) -> Option<PathBuf> {
        Some(Self::default_hosts_file())
    }

    fn has_active_session(&self) -> bool {
        Self::default_hosts_file().exists()
    }

    fn capture_session(&self, vault_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(vault_dir)?;
        let hosts = Self::default_hosts_file();
        if hosts.exists() {
            std::fs::copy(&hosts, vault_dir.join("hosts.json"))
                .with_context(|| format!("Failed to copy {:?}", hosts))?;
        }
        Ok(())
    }

    fn activate_session(&self, _profile: &AccountProfile, vault_dir: &Path) -> Result<()> {
        let vault_hosts = vault_dir.join("hosts.json");
        if vault_hosts.exists() {
            let target_hosts = Self::default_hosts_file();
            if let Some(parent) = target_hosts.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&vault_hosts, &target_hosts)
                .with_context(|| format!("Failed to activate Copilot hosts.json from {:?}", vault_hosts))?;
        }
        Ok(())
    }
}
