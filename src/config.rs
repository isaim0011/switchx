use directories::ProjectDirs;
use std::path::PathBuf;

pub struct AppConfig;

impl AppConfig {
    pub fn root_dir() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "switchx", "switchx") {
            dirs.data_dir().to_path_buf()
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".switchx")
        }
    }

    pub fn profiles_file() -> PathBuf {
        Self::root_dir().join("profiles.json")
    }

    pub fn quotas_file() -> PathBuf {
        Self::root_dir().join("quotas.json")
    }

    pub fn provider_storage_dir(provider: &str) -> PathBuf {
        Self::root_dir().join("providers").join(provider)
    }

    pub fn session_vault_dir(provider: &str, session_id: &str) -> PathBuf {
        Self::provider_storage_dir(provider).join(session_id)
    }
}
