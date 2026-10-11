use super::ProviderAdapter;
use crate::model::AccountProfile;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct AntigravityAdapter;

impl AntigravityAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Primary IDE state storage file for Google Antigravity and Cursor
    fn get_state_db_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(app_data) = std::env::var_os("APPDATA") {
            let app_data = PathBuf::from(app_data);
            // Antigravity standalone
            candidates.push(app_data.join("Antigravity").join("User").join("globalStorage").join("state.vscdb"));
            // Cursor
            candidates.push(app_data.join("Cursor").join("User").join("globalStorage").join("state.vscdb"));
            // VS Code
            candidates.push(app_data.join("Code").join("User").join("globalStorage").join("state.vscdb"));
        }
        candidates
    }
}

impl Default for AntigravityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for AntigravityAdapter {
    fn id(&self) -> &'static str {
        "antigravity"
    }

    fn display_name(&self) -> &'static str {
        "Google Antigravity & Cursor"
    }

    fn target_auth_path(&self) -> Option<PathBuf> {
        for candidate in Self::get_state_db_candidates() {
            if candidate.exists() {
                return Some(candidate);
            }
        }
        Self::get_state_db_candidates().into_iter().next()
    }

    fn has_active_session(&self) -> bool {
        self.target_auth_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn capture_session(&self, vault_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(vault_dir)
            .with_context(|| format!("Failed to create vault dir: {:?}", vault_dir))?;

        if let Some(target) = self.target_auth_path() {
            if target.exists() {
                let backup_file = vault_dir.join("state.vscdb");
                std::fs::copy(&target, &backup_file).with_context(|| {
                    format!("Failed to snapshot {:?} into {:?}", target, backup_file)
                })?;
            }
        }
        // Write vault metadata
        let marker = vault_dir.join("antigravity_profile.json");
        let meta = serde_json::json!({
            "captured_at": chrono::Utc::now().to_rfc3339(),
            "type": "antigravity_session"
        });
        std::fs::write(&marker, serde_json::to_string_pretty(&meta)?)?;
        Ok(())
    }

    fn activate_session(&self, _profile: &AccountProfile, vault_dir: &Path) -> Result<()> {
        let vault_db = vault_dir.join("state.vscdb");
        if !vault_db.exists() {
            return Err(anyhow::anyhow!("Vault session file {:?} does not exist", vault_db));
        }

        if let Some(target) = self.target_auth_path() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Atomic copy
            let tmp_target = target.with_extension("vscdb.tmp");
            std::fs::copy(&vault_db, &tmp_target)?;
            std::fs::rename(&tmp_target, &target).or_else(|_| {
                // Fallback for Windows if rename fails due to lock
                std::fs::copy(&vault_db, &target).map(|_| ())
            })?;
        }
        Ok(())
    }
}
