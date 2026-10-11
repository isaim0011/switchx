use crate::config::AppConfig;
use crate::model::{AccountProfile, AccountQuotaRecord, ProviderKind, UsageEvent};
use crate::providers::{
    antigravity::AntigravityAdapter, claude_code::ClaudeCodeAdapter, copilot::CopilotAdapter,
    ProviderAdapter,
};
use crate::quota::{QuotaCalculator, QuotaSnapshot};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct ProfileVault {
    root_dir: PathBuf,
}

impl ProfileVault {
    pub fn new() -> Self {
        let root = AppConfig::root_dir();
        let _ = fs::create_dir_all(&root);
        Self { root_dir: root }
    }

    pub fn with_root(root: PathBuf) -> Self {
        let _ = fs::create_dir_all(&root);
        Self { root_dir: root }
    }

    fn profiles_path(&self) -> PathBuf {
        self.root_dir.join("profiles.json")
    }

    fn quotas_path(&self) -> PathBuf {
        self.root_dir.join("quotas.json")
    }

    pub fn load_profiles(&self) -> Result<Vec<AccountProfile>> {
        let p = self.profiles_path();
        if !p.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&p).with_context(|| format!("Failed to read {:?}", p))?;
        let profiles = serde_json::from_str(&content).unwrap_or_default();
        Ok(profiles)
    }

    pub fn save_profiles(&self, profiles: &[AccountProfile]) -> Result<()> {
        let p = self.profiles_path();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(profiles)?;
        fs::write(&p, data)?;
        Ok(())
    }

    pub fn load_quotas(&self) -> Result<Vec<AccountQuotaRecord>> {
        let p = self.quotas_path();
        if !p.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&p).with_context(|| format!("Failed to read {:?}", p))?;
        let quotas = serde_json::from_str(&content).unwrap_or_default();
        Ok(quotas)
    }

    pub fn save_quotas(&self, quotas: &[AccountQuotaRecord]) -> Result<()> {
        let p = self.quotas_path();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(quotas)?;
        fs::write(&p, data)?;
        Ok(())
    }

    pub fn get_adapter(&self, provider: ProviderKind) -> Box<dyn ProviderAdapter> {
        match provider {
            ProviderKind::Antigravity => Box::new(AntigravityAdapter::new()),
            ProviderKind::ClaudeCode => Box::new(ClaudeCodeAdapter::new()),
            ProviderKind::Copilot => Box::new(CopilotAdapter::new()),
        }
    }

    pub fn add_account(
        &self,
        provider: ProviderKind,
        name: &str,
        email: Option<String>,
        label: Option<String>,
        capture_current: bool,
    ) -> Result<AccountProfile> {
        let mut profiles = self.load_profiles()?;
        if profiles.iter().any(|p| p.provider == provider && p.name == name) {
            return Err(anyhow::anyhow!(
                "Account '{}' already exists for provider '{}'",
                name,
                provider
            ));
        }

        let session_id = format!("{}_{}", provider, name);
        let vault_dir = self.root_dir.join("providers").join(provider.to_string()).join(&session_id);

        let adapter = self.get_adapter(provider);
        if capture_current && adapter.has_active_session() {
            adapter.capture_session(&vault_dir)?;
        } else {
            fs::create_dir_all(&vault_dir)?;
        }

        let is_first = !profiles.iter().any(|p| p.provider == provider);
        let profile = AccountProfile {
            name: name.to_string(),
            provider,
            email,
            label,
            created_at: Utc::now(),
            last_used: None,
            is_active: is_first,
            session_id,
            metadata: std::collections::HashMap::new(),
        };

        profiles.push(profile.clone());
        self.save_profiles(&profiles)?;

        // Initialize quota record
        let mut quotas = self.load_quotas()?;
        if !quotas.iter().any(|q| q.provider == provider && q.account_name == name) {
            quotas.push(AccountQuotaRecord::new(name.to_string(), provider));
            self.save_quotas(&quotas)?;
        }

        Ok(profile)
    }

    pub fn switch_account(&self, provider: ProviderKind, name: &str) -> Result<()> {
        let mut profiles = self.load_profiles()?;
        let target_idx = profiles
            .iter()
            .position(|p| p.provider == provider && p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Account '{}' not found for provider '{}'", name, provider))?;

        let adapter = self.get_adapter(provider);
        let session_id = profiles[target_idx].session_id.clone();
        let vault_dir = self.root_dir.join("providers").join(provider.to_string()).join(&session_id);

        adapter.activate_session(&profiles[target_idx], &vault_dir)?;

        for p in profiles.iter_mut().filter(|p| p.provider == provider) {
            p.is_active = false;
        }
        profiles[target_idx].is_active = true;
        profiles[target_idx].last_used = Some(Utc::now());

        self.save_profiles(&profiles)?;
        Ok(())
    }

    pub fn record_usage(&self, provider: ProviderKind, name: &str, units: u64) -> Result<()> {
        let mut quotas = self.load_quotas()?;
        let record = if let Some(rec) = quotas.iter_mut().find(|q| q.provider == provider && q.account_name == name) {
            rec
        } else {
            quotas.push(AccountQuotaRecord::new(name.to_string(), provider));
            quotas.last_mut().unwrap()
        };

        record.usage_history.push(UsageEvent {
            timestamp: Utc::now(),
            tokens_or_units: units,
        });

        // Prune older than 8 days
        let cutoff = Utc::now() - chrono::Duration::days(8);
        record.usage_history.retain(|ev| ev.timestamp >= cutoff);

        self.save_quotas(&quotas)?;
        Ok(())
    }

    pub fn get_account_quota_snapshots(&self, provider: Option<ProviderKind>) -> Result<Vec<QuotaSnapshot>> {
        let profiles = self.load_profiles()?;
        let quotas = self.load_quotas()?;
        let now = Utc::now();

        let mut snapshots = Vec::new();
        for p in &profiles {
            if let Some(filter_provider) = provider {
                if p.provider != filter_provider {
                    continue;
                }
            }

            let quota_rec = quotas
                .iter()
                .find(|q| q.provider == p.provider && q.account_name == p.name)
                .cloned()
                .unwrap_or_else(|| AccountQuotaRecord::new(p.name.clone(), p.provider));

            let (five_h, weekly, health) = QuotaCalculator::compute_status(
                now,
                quota_rec.five_hour_capacity,
                quota_rec.weekly_capacity,
                &quota_rec.usage_history,
            );

            let rec_text = match health {
                crate::quota::QuotaHealth::Optimal => "Healthy - Ready for heavy workloads",
                crate::quota::QuotaHealth::Moderate => "Good - Moderate headroom remaining",
                crate::quota::QuotaHealth::Warning => "Caution - Nearing 20% sprint threshold",
                crate::quota::QuotaHealth::Critical => "Urgent - Less than 10% remaining! Consider switching",
                crate::quota::QuotaHealth::Throttled => "Rate-limited! Recommend switching to an alt account",
            };

            snapshots.push(QuotaSnapshot {
                account_name: p.name.clone(),
                provider: p.provider.to_string(),
                health,
                five_hour_window: five_h,
                weekly_window: weekly,
                recommendation: rec_text.to_string(),
            });
        }

        Ok(snapshots)
    }

    pub fn pick_best_account(&self, provider: ProviderKind) -> Result<Option<String>> {
        let snapshots = self.get_account_quota_snapshots(Some(provider))?;
        if snapshots.is_empty() {
            return Ok(None);
        }

        let mut sorted = snapshots;
        sorted.sort_by(|a, b| {
            b.five_hour_window
                .remaining
                .cmp(&a.five_hour_window.remaining)
        });

        Ok(sorted.first().map(|s| s.account_name.clone()))
    }
}

impl Default for ProfileVault {
    fn default() -> Self {
        Self::new()
    }
}
