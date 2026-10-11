# PLAN.md - SwitchX (Universal AI Account & Quota Router)

## 1. Goal
Build **SwitchX** in Rust: a professional, ultra-fast, cross-platform subscription & account switcher and quota manager.
Outperforms single-editor extensions (`venomchampion`, `BoyGR`) by managing accounts and tracking rolling quota limits (5-hour & weekly) across:
- **Google Antigravity IDE & Cursor**
- **Claude Code CLI** (`claude`)
- **GitHub Copilot / Codex**

## 2. Architecture & File Layout
```text
switchx/
├── Cargo.toml
├── PLAN.md
├── src/
│   ├── main.rs                   # CLI entrypoint with Clap subcommands
│   ├── lib.rs                    # Library root
│   ├── config.rs                 # Storage directory, config schema
│   ├── model.rs                  # Core data models (Account, QuotaState, ProviderKind)
│   ├── quota/
│   │   ├── mod.rs                # Universal quota engine & rolling-window tracker
│   │   └── calculator.rs         # 5-hour and weekly window calculations with exact countdowns
│   ├── providers/
│   │   ├── mod.rs                # Provider trait
│   │   ├── antigravity.rs        # Antigravity IDE / Cursor state manager
│   │   ├── claude_code.rs        # Claude Code CLAUDE_CONFIG_DIR profile manager
│   │   └── copilot.rs            # GitHub Copilot hosts.json profile manager
│   ├── vault.rs                  # Encrypted/safe profile storage manager
│   └── runner.rs                 # Transparent command launcher (injects profile env & executes)
└── tests/
    ├── quota_tests.rs            # Rolling window & countdown calculation tests
    └── vault_tests.rs            # Account switching & profile isolation tests
```

## 3. Core Features
1. **Multi-App Account Profiles**:
   - Google Antigravity (multi-Google accounts)
   - Claude Code (multi-Anthropic subscriptions via isolated config dirs)
   - Copilot / Codex (multi-GitHub accounts)
2. **True Rolling Quota Engine**:
   - Tracks 5-hour rolling sprint window
   - Tracks weekly marathon cap
   - Provides exact millisecond/second countdown until capacity is freed up
   - Calculates remaining percentage and status (Normal, Warning <= 20%, Critical <= 10%, Throttled)
3. **Seamless Profile Swapper & Runner**:
   - `switchx switch <provider> <account-name>`
   - `switchx run <provider> <account-name> -- <command...>` (e.g. `switchx run claude work -- claude`)
   - `switchx status` (matrix view of all providers, active accounts, remaining quota, reset timers)
   - `switchx auto-next <provider>` (auto-picks the account with highest available quota)

## 4. Test Strategy
- Unit tests for quota calculation: sliding 5h window, expired token pruning, reset duration formatting.
- Unit tests for profile vault: adding accounts, switching active profile, atomic file replacement.
- Integration tests: CLI execution (`switchx status`, `switchx list`, `switchx switch`).
