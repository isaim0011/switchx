use clap::{Parser, Subcommand};
use colored::*;
use switchx::{
    model::ProviderKind,
    quota::{QuotaHealth, QuotaSnapshot},
    runner::ProcessRunner,
    vault::ProfileVault,
};

#[derive(Parser)]
#[command(name = "switchx")]
#[command(about = "Universal subscription account switcher and rolling quota monitor for Antigravity, Claude Code, and Copilot/Codex", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an account profile (e.g. switchx add claude work --capture)
    Add {
        /// Target provider: antigravity, claude-code, copilot
        provider: ProviderKind,
        /// Unique account nickname (e.g. work, personal, alt1)
        name: String,
        /// Associated email
        #[arg(short, long)]
        email: Option<String>,
        /// Descriptive label (e.g. "Work Pro Subscription")
        #[arg(short, long)]
        label: Option<String>,
        /// Capture current logged in session from this machine immediately
        #[arg(short, long)]
        capture: bool,
    },
    /// Switch active account (e.g. switchx switch claude work)
    Switch {
        /// Target provider: antigravity, claude-code, copilot
        provider: ProviderKind,
        /// Account nickname
        name: String,
    },
    /// List all registered accounts across providers
    List {
        /// Optional provider filter
        #[arg(short, long)]
        provider: Option<ProviderKind>,
    },
    /// Show live quota status, 5-hour rolling windows, weekly limits, and countdown timers
    Status {
        /// Optional provider filter
        #[arg(short, long)]
        provider: Option<ProviderKind>,
    },
    /// Auto-switch to the account with the highest available quota
    AutoNext {
        /// Target provider
        provider: ProviderKind,
    },
    /// Record token or unit consumption for testing / tracking
    Record {
        provider: ProviderKind,
        name: String,
        #[arg(short, long, default_value_t = 1)]
        units: u64,
    },
    /// Run a command isolated with the given account profile (e.g. switchx run claude work -- claude)
    Run {
        provider: ProviderKind,
        name: String,
        #[arg(last = true)]
        command: Vec<String>,
    },
}

fn print_snapshot(s: &QuotaSnapshot) {
    let health_badge = match s.health {
        QuotaHealth::Optimal => " OPTIMAL ".on_green().bold().black(),
        QuotaHealth::Moderate => " MODERATE ".on_blue().bold().white(),
        QuotaHealth::Warning => " WARNING ".on_yellow().bold().black(),
        QuotaHealth::Critical => " CRITICAL ".on_bright_red().bold().white(),
        QuotaHealth::Throttled => " THROTTLED ".on_red().bold().white(),
    };

    println!(
        "\n{} Account: {} ({}) [{}]",
        "•".bright_cyan(),
        s.account_name.bold().bright_white(),
        s.provider.bright_yellow(),
        health_badge
    );

    println!(
        "  5-Hour Sprint Window:  {}/{} used ({:.1}% remaining) - Reset in: {}",
        s.five_hour_window.used,
        s.five_hour_window.total,
        s.five_hour_window.percent_remaining,
        s.five_hour_window.reset_description.bright_magenta()
    );

    println!(
        "  Weekly Marathon Cap:   {}/{} used ({:.1}% remaining) - Reset in: {}",
        s.weekly_window.used,
        s.weekly_window.total,
        s.weekly_window.percent_remaining,
        s.weekly_window.reset_description.bright_magenta()
    );

    println!("  Advice:                {}", s.recommendation.italic().bright_black());
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let vault = ProfileVault::new();

    match cli.command {
        Commands::Add {
            provider,
            name,
            email,
            label,
            capture,
        } => {
            let profile = vault.add_account(provider, &name, email, label, capture)?;
            println!(
                "{} Successfully registered account '{}' for {}",
                "✔".bright_green(),
                profile.name.bold(),
                provider.display_name().bright_cyan()
            );
        }
        Commands::Switch { provider, name } => {
            vault.switch_account(provider, &name)?;
            println!(
                "{} Switched {} active session to '{}'",
                "✔".bright_green(),
                provider.display_name().bright_cyan(),
                name.bold()
            );
        }
        Commands::List { provider } => {
            let profiles = vault.load_profiles()?;
            let filtered: Vec<_> = profiles
                .into_iter()
                .filter(|p| provider.map(|filter| p.provider == filter).unwrap_or(true))
                .collect();

            if filtered.is_empty() {
                println!("{}", "No accounts configured yet. Run 'switchx add <provider> <name>' to start.".yellow());
                return Ok(());
            }

            println!("\n{}", "Registered AI Subscription Accounts:".bold().underline());
            for p in filtered {
                let status_icon = if p.is_active { "● ACTIVE".green() } else { "○ idle".dimmed() };
                println!(
                    "  [{}] {} ({}) - {}",
                    status_icon,
                    p.name.bold().bright_white(),
                    p.provider.to_string().bright_yellow(),
                    p.email.as_deref().unwrap_or("no email configured").dimmed()
                );
            }
            println!();
        }
        Commands::Status { provider } => {
            let snapshots = vault.get_account_quota_snapshots(provider)?;
            if snapshots.is_empty() {
                println!("{}", "No accounts found to monitor. Run 'switchx add' first.".yellow());
                return Ok(());
            }

            println!("\n{}", "=== Universal Quota & Rate Limit Matrix ===".bold().bright_cyan());
            for s in &snapshots {
                print_snapshot(s);
            }
            println!();
        }
        Commands::AutoNext { provider } => {
            if let Some(best) = vault.pick_best_account(provider)? {
                vault.switch_account(provider, &best)?;
                println!(
                    "{} Auto-Failover: Swapped {} to '{}' (highest available quota)",
                    "⚡".bright_yellow(),
                    provider.display_name().bright_cyan(),
                    best.bold()
                );
            } else {
                println!("{}", "No accounts available to auto-switch.".red());
            }
        }
        Commands::Record {
            provider,
            name,
            units,
        } => {
            vault.record_usage(provider, &name, units)?;
            println!(
                "{} Recorded {} units of usage for {}::{}",
                "✔".bright_green(),
                units,
                provider,
                name
            );
        }
        Commands::Run {
            provider,
            name,
            command,
        } => {
            let exit_code = ProcessRunner::run_with_profile(&vault, provider, &name, &command)?;
            std::process::exit(exit_code);
        }
    }

    Ok(())
}
