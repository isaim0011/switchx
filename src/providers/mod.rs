use crate::model::AccountProfile;
use anyhow::Result;
use std::path::PathBuf;

pub mod antigravity;
pub mod claude_code;
pub mod copilot;

pub trait ProviderAdapter {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;

    /// Locate where the target application currently reads its active session / credentials
    fn target_auth_path(&self) -> Option<PathBuf>;

    /// Capture current active session from the system and snapshot into the vault session dir
    fn capture_session(&self, vault_dir: &std::path::Path) -> Result<()>;

    /// Restore or activate a session from the vault dir into the system target path
    fn activate_session(&self, profile: &AccountProfile, vault_dir: &std::path::Path) -> Result<()>;

    /// Check if target application is currently logged in or has an active session file
    fn has_active_session(&self) -> bool;
}
