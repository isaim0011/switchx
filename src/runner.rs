use crate::config::AppConfig;
use crate::model::ProviderKind;
use crate::vault::ProfileVault;
use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub struct ProcessRunner;

impl ProcessRunner {
    pub fn run_with_profile(
        vault: &ProfileVault,
        provider: ProviderKind,
        account_name: &str,
        cmd_args: &[String],
    ) -> Result<i32> {
        if cmd_args.is_empty() {
            return Err(anyhow::anyhow!("No command specified to run"));
        }

        let profiles = vault.load_profiles()?;
        let profile = profiles
            .iter()
            .find(|p| p.provider == provider && p.name == account_name)
            .ok_or_else(|| anyhow::anyhow!("Account '{}' not found for provider '{}'", account_name, provider))?;

        let mut cmd = Command::new(&cmd_args[0]);
        if cmd_args.len() > 1 {
            cmd.args(&cmd_args[1..]);
        }

        match provider {
            ProviderKind::ClaudeCode => {
                // Point CLAUDE_CONFIG_DIR directly to the isolated session vault
                let session_dir = AppConfig::session_vault_dir(&provider.to_string(), &profile.session_id);
                cmd.env("CLAUDE_CONFIG_DIR", session_dir);
            }
            ProviderKind::Antigravity => {
                // Ensure the active session is swapped
                vault.switch_account(provider, account_name)?;
            }
            ProviderKind::Copilot => {
                // Ensure active Copilot session is swapped
                vault.switch_account(provider, account_name)?;
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn command: {:?}", cmd_args[0]))?;

        let status = child.wait()?;
        Ok(status.code().unwrap_or(0))
    }
}
