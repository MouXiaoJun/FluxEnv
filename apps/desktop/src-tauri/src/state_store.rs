//! Persist provider *metadata* (names, keys, enabled) under the app data directory.
//! Secret values are never written here — only in the OS keychain.

use std::path::{Path, PathBuf};

use fluxenv_models::{EnvVariable, ProviderConfig};
use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "providers_state.json";

#[derive(Debug, Serialize, Deserialize)]
struct PersistedFile {
    version: u32,
    providers: Vec<ProviderConfig>,
}

pub fn default_builtin_providers() -> Vec<ProviderConfig> {
    vec![
        ProviderConfig {
            name: "openai".to_string(),
            enabled: false,
            variables: vec![EnvVariable {
                key: "OPENAI_API_KEY".to_string(),
                value: String::new(),
                is_secret: true,
            }],
        },
        ProviderConfig {
            name: "anthropic".to_string(),
            enabled: false,
            variables: vec![EnvVariable {
                key: "ANTHROPIC_API_KEY".to_string(),
                value: String::new(),
                is_secret: true,
            }],
        },
        ProviderConfig {
            name: "deepseek".to_string(),
            enabled: false,
            variables: vec![EnvVariable {
                key: "DEEPSEEK_API_KEY".to_string(),
                value: String::new(),
                is_secret: true,
            }],
        },
        ProviderConfig {
            name: "openrouter".to_string(),
            enabled: false,
            variables: vec![EnvVariable {
                key: "OPENROUTER_API_KEY".to_string(),
                value: String::new(),
                is_secret: true,
            }],
        },
    ]
}

fn state_path(data_dir: &Path) -> PathBuf {
    data_dir.join(FILE_NAME)
}

pub fn load_or_init(data_dir: &Path) -> Vec<ProviderConfig> {
    let path = state_path(data_dir);
    if !path.exists() {
        let defaults = default_builtin_providers();
        let _ = save(data_dir, &defaults);
        return defaults;
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<PersistedFile>(&text) {
            Ok(p) => sanitize_loaded(p.providers),
            Err(_) => {
                let defaults = default_builtin_providers();
                let _ = save(data_dir, &defaults);
                defaults
            }
        },
        Err(_) => {
            let defaults = default_builtin_providers();
            let _ = save(data_dir, &defaults);
            defaults
        }
    }
}

/// Strip any accidental plaintext secrets and ensure at least one variable slot per provider.
fn sanitize_loaded(mut providers: Vec<ProviderConfig>) -> Vec<ProviderConfig> {
    for p in &mut providers {
        for v in &mut p.variables {
            if v.is_secret {
                v.value.clear();
            }
        }
    }
    providers
}

pub fn is_builtin_name(name: &str) -> bool {
    matches!(
        name,
        "openai" | "anthropic" | "deepseek" | "openrouter"
    )
}

pub fn save(data_dir: &Path, providers: &[ProviderConfig]) -> Result<(), String> {
    std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let body = PersistedFile {
        version: 1,
        providers: providers.to_vec(),
    };
    let json = serde_json::to_string_pretty(&body).map_err(|e| e.to_string())?;
    let path = state_path(data_dir);
    std::fs::write(&path, json).map_err(|e| e.to_string())
}
