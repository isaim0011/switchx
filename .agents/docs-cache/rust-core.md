# Rust Standard & Core Dependencies Digest

## Versions
- Rust: `1.98.1` (installed via rustup/cargo)
- `tokio`: `1.43.0` (rt-multi-thread, macros, fs, sync, time)
- `serde`: `1.0.218` (derive)
- `serde_json`: `1.0.139`
- `chrono`: `0.4.39` (serde, clock)
- `clap`: `4.5.31` (derive)
- `directories`: `5.0.1` (ProjectDirs standard paths across Windows/macOS/Linux)
- `anyhow`: `1.0.96`
- `thiserror`: `2.0.12`

## Key Patterns
- Configuration directories on Windows resolve via `directories::ProjectDirs::from("com", "switchx", "switchx")` -> `AppData/Roaming/switchx`
- Claude Code profile directory routing uses standard `CLAUDE_CONFIG_DIR` environment variable isolation or hot-swapping `~/.claude.json`.
- Antigravity profile snapshotting preserves `state.vscdb` + auth records and restores atomically using atomic temp file replacement.
- Copilot profile switching routes `~/.config/github-copilot/hosts.json` tokens per account.
- Rolling quota windows: 5-hour rolling sliding expiration calculation using request timestamp queues and duration arithmetic (`chrono::Duration::hours(5)`).
