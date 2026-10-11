# Changelog

All notable changes to **SwitchX Pro** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-10-12

### Added
- **Universal Multi-App Subscription Support**:
  - Full OAuth session persistence and DPAPI/keyring profile management for **Claude Code CLI** (`claude`), **Google Antigravity & Cursor** (`state.vscdb`), and **GitHub Copilot / Codex** (`hosts.json`).
- **High-Performance Rust Core Engine**:
  - Sub-millisecond `<10ms` account switching.
  - Zero-touch CLI command runner: `switchx run <provider> <account> -- <command...>`.
- **Dual Rolling Quota Engine**:
  - Sliding 5-Hour sprint window calculator with live token/unit usage tracking.
  - 7-Day marathon limit window calculator.
  - Real-time countdown calculation formatted to exact days, hours, minutes, and seconds.
  - Health categorizer: `OPTIMAL`, `MODERATE`, `WARNING` ($\le 20\%$), `CRITICAL` ($\le 10\%$), and `THROTTLED`.
- **Intelligent Auto-Failover (`auto-next`)**:
  - Evaluates remaining quotas across account pools and automatically hot-swaps to the account with the highest available capacity upon rate limits.
- **Sleek VS Code & Open-VSX Extension (`switchx-pro`)**:
  - Unified package naming as `switchx-pro` under the `isaim0011` publisher namespace.
  - Activity Bar view container with radial dark glassmorphism dashboard.
  - Dynamic visual progress bars, real-time health badges, and one-click failover.
  - Interactive multi-provider filtering tabs (`All Apps`, `Claude`, `Antigravity`, `Copilot`).
- **Cloudflare Pages Static Site**:
  - Responsive landing page with dark design system, installation guides, and competitor comparison matrix.
  - Configured automated deployment in GitHub Actions passing `CLOUDFLARE_ACCOUNT_ID` directly to Wrangler.
- **Ecosystem Packaging**:
  - `.vsix` extension package (`switchx-pro-1.0.0.vsix`).
  - Published to crates.io as `switchx`.
  - GitHub Actions automated multi-OS testing and deployment pipeline.
