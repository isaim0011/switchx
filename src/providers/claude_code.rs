use super::ProviderAdapter;
use crate::model::AccountProfile;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn default_config_file() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude.json")
    }

    pub fn default_config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
    }
}

impl Default for ClaudeCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for ClaudeCodeAdapter {
    fn id(&self) -> &'static str {
        "claude-code"
    }

    fn display_name(&self) -> &'static str {
        "Claude Code CLI"
    }

    fn target_auth_path(&self) -> Option<PathBuf> {
        let file = Self::default_config_file();
        if file.exists() {
            Some(file)
        } else {
            Some(Self::default_config_dir())
        }
    }

    fn has_active_session(&self) -> bool {
        Self::default_config_file().exists() || Self::default_config_dir().exists()
    }

    fn capture_session(&self, vault_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(vault_dir)?;

        let cfg_file = Self::default_config_file();
        if cfg_file.exists() {
            std::fs::copy(&cfg_file, vault_dir.join(".claude.json"))
                .with_context(|| format!("Failed to copy {:?}", cfg_file))?;
        }

        let cfg_dir = Self::default_config_dir();
        if cfg_dir.exists() {
            let target_profile_dir = vault_dir.join(".claude");
            std::fs::create_dir_all(&target_profile_dir)?;
            for entry in std::fs::read_dir(&cfg_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let dest = target_profile_dir.join(entry.file_name());
                    let _ = std::fs::copy(entry.path(), dest);
                }
            }
        }
        Ok(())
    }

    fn activate_session(&self, _profile: &AccountProfile, vault_dir: &Path) -> Result<()> {
        // Activate file
        let vault_file = vault_dir.join(".claude.json");
        if vault_file.exists() {
            let target_file = Self::default_config_file();
            std::fs::copy(&vault_file, &target_file)
                .with_context(|| format!("Failed to restore ~/.claude.json from {:?}", vault_file))?;
        }

        // Activate dir
        let vault_sub_dir = vault_dir.join(".claude");
        if vault_sub_dir.exists() {
            let target_dir = Self::default_config_dir();
            std::fs::create_dir_all(&target_dir)?;
            for entry in std::fs::read_dir(&vault_sub_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let dest = target_dir.join(entry.file_name());
                    let _ = std::fs::copy(entry.path(), dest);
                }
            }
        }
        Ok(())
    }
}
