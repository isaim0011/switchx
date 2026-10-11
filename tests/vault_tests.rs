use switchx::model::ProviderKind;
use switchx::vault::ProfileVault;
use tempfile::TempDir;

#[test]
fn test_vault_add_and_switch_accounts() {
    let temp_dir = TempDir::new().unwrap();
    let vault = ProfileVault::with_root(temp_dir.path().to_path_buf());

    // 1. Add account 1
    let acc1 = vault
        .add_account(
            ProviderKind::ClaudeCode,
            "work",
            Some("work@company.com".to_string()),
            Some("Work Pro".to_string()),
            false,
        )
        .unwrap();

    assert!(acc1.is_active);

    // 2. Add account 2
    let acc2 = vault
        .add_account(
            ProviderKind::ClaudeCode,
            "personal",
            Some("me@gmail.com".to_string()),
            Some("Personal Claude".to_string()),
            false,
        )
        .unwrap();

    assert!(!acc2.is_active);

    // 3. Switch to personal
    vault.switch_account(ProviderKind::ClaudeCode, "personal").unwrap();

    let profiles = vault.load_profiles().unwrap();
    let work = profiles.iter().find(|p| p.name == "work").unwrap();
    let personal = profiles.iter().find(|p| p.name == "personal").unwrap();

    assert!(!work.is_active);
    assert!(personal.is_active);
}

#[test]
fn test_vault_auto_pick_highest_quota() {
    let temp_dir = TempDir::new().unwrap();
    let vault = ProfileVault::with_root(temp_dir.path().to_path_buf());

    vault
        .add_account(ProviderKind::Antigravity, "acc1", None, None, false)
        .unwrap();
    vault
        .add_account(ProviderKind::Antigravity, "acc2", None, None, false)
        .unwrap();

    // Consume heavy quota on acc1
    vault.record_usage(ProviderKind::Antigravity, "acc1", 45).unwrap();
    // Consume minor quota on acc2
    vault.record_usage(ProviderKind::Antigravity, "acc2", 5).unwrap();

    let best = vault.pick_best_account(ProviderKind::Antigravity).unwrap();
    assert_eq!(best, Some("acc2".to_string()));
}
