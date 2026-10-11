use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    Antigravity,
    ClaudeCode,
    Copilot,
}

impl ProviderKind {
    pub fn all() -> &'static [ProviderKind] {
        &[
            ProviderKind::Antigravity,
            ProviderKind::ClaudeCode,
            ProviderKind::Copilot,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderKind::Antigravity => "Google Antigravity / Cursor",
            ProviderKind::ClaudeCode => "Claude Code CLI",
            ProviderKind::Copilot => "GitHub Copilot / Codex",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderKind::Antigravity => write!(f, "antigravity"),
            ProviderKind::ClaudeCode => write!(f, "claude-code"),
            ProviderKind::Copilot => write!(f, "copilot"),
        }
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "antigravity" | "ag" | "cursor" => Ok(ProviderKind::Antigravity),
            "claude" | "claude-code" | "claudecode" => Ok(ProviderKind::ClaudeCode),
            "copilot" | "codex" | "github-copilot" => Ok(ProviderKind::Copilot),
            other => Err(anyhow::anyhow!("Unknown provider: '{}'. Valid options: antigravity, claude-code, copilot", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfile {
    pub name: String,
    pub provider: ProviderKind,
    pub email: Option<String>,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub is_active: bool,
    /// Storage path or identifier for the isolated session profile
    pub session_id: String,
    /// Optional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub timestamp: DateTime<Utc>,
    pub tokens_or_units: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountQuotaRecord {
    pub account_name: String,
    pub provider: ProviderKind,
    /// Limit capacity for the 5-hour rolling sprint window (e.g. 50 requests/credits)
    pub five_hour_capacity: u64,
    /// Limit capacity for the 7-day weekly marathon window (e.g. 500 requests/credits)
    pub weekly_capacity: u64,
    /// Recent usage timestamps
    #[serde(default)]
    pub usage_history: Vec<UsageEvent>,
}

impl AccountQuotaRecord {
    pub fn new(account_name: String, provider: ProviderKind) -> Self {
        let (five_hour_capacity, weekly_capacity) = match provider {
            ProviderKind::Antigravity => (50, 500),
            ProviderKind::ClaudeCode => (100, 1000),
            ProviderKind::Copilot => (150, 1500),
        };
        Self {
            account_name,
            provider,
            five_hour_capacity,
            weekly_capacity,
            usage_history: Vec::new(),
        }
    }
}
