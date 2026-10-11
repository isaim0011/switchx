use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuotaHealth {
    Optimal,
    Moderate,
    Warning,
    Critical,
    Throttled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowStatus {
    pub used: u64,
    pub total: u64,
    pub remaining: u64,
    pub percent_remaining: f64,
    pub resets_in_seconds: i64,
    pub reset_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSnapshot {
    pub account_name: String,
    pub provider: String,
    pub health: QuotaHealth,
    pub five_hour_window: WindowStatus,
    pub weekly_window: WindowStatus,
    pub recommendation: String,
}

pub struct QuotaCalculator;

impl QuotaCalculator {
    pub fn compute_status(
        now: DateTime<Utc>,
        five_hour_capacity: u64,
        weekly_capacity: u64,
        events: &[crate::model::UsageEvent],
    ) -> (WindowStatus, WindowStatus, QuotaHealth) {
        let five_hours_ago = now - Duration::hours(5);
        let seven_days_ago = now - Duration::days(7);

        let mut five_hour_used = 0u64;
        let mut weekly_used = 0u64;
        let mut oldest_in_5h: Option<DateTime<Utc>> = None;
        let mut oldest_in_weekly: Option<DateTime<Utc>> = None;

        for ev in events {
            if ev.timestamp >= seven_days_ago {
                weekly_used += ev.tokens_or_units;
                if oldest_in_weekly.is_none() || ev.timestamp < oldest_in_weekly.unwrap() {
                    oldest_in_weekly = Some(ev.timestamp);
                }
            }
            if ev.timestamp >= five_hours_ago {
                five_hour_used += ev.tokens_or_units;
                if oldest_in_5h.is_none() || ev.timestamp < oldest_in_5h.unwrap() {
                    oldest_in_5h = Some(ev.timestamp);
                }
            }
        }

        // 5-Hour calculations
        let five_hour_remaining = five_hour_capacity.saturating_sub(five_hour_used);
        let five_hour_pct = if five_hour_capacity > 0 {
            (five_hour_remaining as f64 / five_hour_capacity as f64) * 100.0
        } else {
            0.0
        };

        let five_hour_reset_secs = if five_hour_used >= five_hour_capacity {
            if let Some(oldest) = oldest_in_5h {
                let release_time = oldest + Duration::hours(5);
                (release_time - now).num_seconds().max(0)
            } else {
                0
            }
        } else {
            0
        };

        let five_hour_desc = if five_hour_reset_secs > 0 {
            format_duration(five_hour_reset_secs)
        } else if five_hour_remaining == five_hour_capacity {
            "Full capacity available".to_string()
        } else {
            "Rolling freely".to_string()
        };

        let five_hour_status = WindowStatus {
            used: five_hour_used,
            total: five_hour_capacity,
            remaining: five_hour_remaining,
            percent_remaining: five_hour_pct,
            resets_in_seconds: five_hour_reset_secs,
            reset_description: five_hour_desc,
        };

        // Weekly calculations
        let weekly_remaining = weekly_capacity.saturating_sub(weekly_used);
        let weekly_pct = if weekly_capacity > 0 {
            (weekly_remaining as f64 / weekly_capacity as f64) * 100.0
        } else {
            0.0
        };

        let weekly_reset_secs = if weekly_used >= weekly_capacity {
            if let Some(oldest) = oldest_in_weekly {
                let release_time = oldest + Duration::days(7);
                (release_time - now).num_seconds().max(0)
            } else {
                0
            }
        } else {
            0
        };

        let weekly_desc = if weekly_reset_secs > 0 {
            format_duration(weekly_reset_secs)
        } else if weekly_remaining == weekly_capacity {
            "Full capacity available".to_string()
        } else {
            "Rolling freely".to_string()
        };

        let weekly_status = WindowStatus {
            used: weekly_used,
            total: weekly_capacity,
            remaining: weekly_remaining,
            percent_remaining: weekly_pct,
            resets_in_seconds: weekly_reset_secs,
            reset_description: weekly_desc,
        };

        // Determine Health
        let min_pct = five_hour_pct.min(weekly_pct);
        let health = if five_hour_remaining == 0 || weekly_remaining == 0 {
            QuotaHealth::Throttled
        } else if min_pct <= 10.0 {
            QuotaHealth::Critical
        } else if min_pct <= 25.0 {
            QuotaHealth::Warning
        } else if min_pct <= 60.0 {
            QuotaHealth::Moderate
        } else {
            QuotaHealth::Optimal
        };

        (five_hour_status, weekly_status, health)
    }
}

pub fn format_duration(seconds: i64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
