#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod secret_store;

use std::sync::Mutex;

use fluxenv_core::{build_effective_session_env, set_provider_enabled};
use fluxenv_models::{EnvVariable, Profile, ProviderConfig};

struct AppState {
    providers: Mutex<Vec<ProviderConfig>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            providers: Mutex::new(vec![
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
            ]),
        }
    }
}

/// Clone providers and fill secret values from the OS store for merging (never persist plaintext in state).
fn providers_with_resolved_secrets(providers: &[ProviderConfig]) -> Vec<ProviderConfig> {
    providers
        .iter()
        .map(|p| {
            let mut p = p.clone();
            for v in &mut p.variables {
                if v.is_secret {
                    if let Some(s) = secret_store::get_secret(&p.name, &v.key) {
                        v.value = s;
                    } else {
                        v.value.clear();
                    }
                }
            }
            p
        })
        .collect()
}

fn delete_provider_secrets(p: &ProviderConfig) {
    for v in &p.variables {
        if v.is_secret {
            secret_store::delete_secret(&p.name, &v.key);
        }
    }
}

fn migrate_secret_if_renamed(
    old_name: &str,
    old_key: &str,
    new_name: &str,
    new_key: &str,
) {
    if old_name == new_name && old_key == new_key {
        return;
    }
    if let Some(val) = secret_store::get_secret(old_name, old_key) {
        let _ = secret_store::set_secret(new_name, new_key, &val);
    }
    secret_store::delete_secret(old_name, old_key);
}

fn looks_like_secret(key: &str) -> bool {
    let k = key.to_ascii_uppercase();
    k.contains("KEY") || k.contains("TOKEN") || k.contains("SECRET") || k.contains("PASSWORD")
}

fn mask_value(value: &str) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }
    if value.len() <= 4 {
        return "*".repeat(value.len());
    }
    format!("{}***{}", &value[..2], &value[value.len() - 2..])
}

#[tauri::command]
fn run_with_selected_profile(profile: String, state: tauri::State<AppState>) -> String {
    let system_env: Vec<EnvVariable> = std::env::vars()
        .map(|(key, value)| EnvVariable {
            is_secret: looks_like_secret(&key),
            key,
            value,
        })
        .collect();
    let profile = Profile {
        name: profile,
        variables: vec![],
    };
    let providers = match state.providers.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => return "Provider state is unavailable.".to_string(),
    };
    let resolved = providers_with_resolved_secrets(&providers);
    let effective = build_effective_session_env(&system_env, &profile, &resolved);

    let mut preview: Vec<String> = effective
        .into_iter()
        .filter(|v| v.key.contains("API_KEY") || v.key.starts_with("OPEN"))
        .take(8)
        .map(|v| {
            if v.is_secret {
                format!("{}={}", v.key, mask_value(&v.value))
            } else {
                format!("{}={}", v.key, v.value)
            }
        })
        .collect();

    if preview.is_empty() {
        preview.push("No API-related variables found in effective session env.".to_string());
    }
    format!("Effective env preview:\n{}", preview.join("\n"))
}

#[tauri::command]
fn toggle_provider(
    provider_name: String,
    enabled: bool,
    state: tauri::State<AppState>,
) -> String {
    let mut providers = match state.providers.lock() {
        Ok(guard) => guard,
        Err(_) => return "Provider state is unavailable.".to_string(),
    };
    if set_provider_enabled(&mut providers, &provider_name, enabled) {
        return format!(
            "Provider '{}' is now {}.",
            provider_name,
            if enabled { "ON" } else { "OFF" }
        );
    }
    format!("Provider '{}' was not found.", provider_name)
}

#[tauri::command]
fn get_secret_status() -> String {
    "Session mode active. API keys are stored in the OS keychain (Keychain / Credential Manager / Secret Service), not in app memory or plain config files."
        .to_string()
}

#[tauri::command]
fn clear_provider_secret(provider_name: String, env_key: String) -> Result<String, String> {
    secret_store::delete_secret(&provider_name, &env_key);
    Ok(format!("Cleared stored secret for {} / {}.", provider_name, env_key))
}

#[tauri::command]
fn secret_is_stored(provider_name: String, env_key: String) -> bool {
    secret_store::has_secret(&provider_name, &env_key)
}

#[tauri::command]
fn upsert_provider(
    original_name: Option<String>,
    name: String,
    env_key: String,
    env_value: String,
    enabled: bool,
    write_secret: bool,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let mut providers = state
        .providers
        .lock()
        .map_err(|_| "Provider state is unavailable.".to_string())?;

    let lookup_name = original_name.as_deref().unwrap_or(&name);
    let old_provider = providers.iter().find(|p| p.name == lookup_name).cloned();

    if let Some(ref old) = original_name {
        if old != &name {
            providers.retain(|p| p.name != *old);
        }
    }

    let old_snapshot = old_provider
        .as_ref()
        .and_then(|p| p.variables.first().map(|v| (p.name.clone(), v.key.clone())));

    if write_secret {
        if env_value.is_empty() {
            secret_store::delete_secret(&name, &env_key);
        } else {
            secret_store::set_secret(&name, &env_key, &env_value)?;
        }
        if let Some((ref on, ref ok)) = old_snapshot {
            if on != &name || ok != &env_key {
                secret_store::delete_secret(on, ok);
            }
        }
    } else if let Some((ref on, ref ok)) = old_snapshot {
        if on != &name || ok != &env_key {
            migrate_secret_if_renamed(on, ok, &name, &env_key);
        }
    }

    let cfg = ProviderConfig {
        name: name.clone(),
        enabled,
        variables: vec![EnvVariable {
            key: env_key,
            value: String::new(),
            is_secret: true,
        }],
    };

    if let Some(idx) = providers.iter().position(|p| p.name == name) {
        providers[idx] = cfg;
    } else {
        providers.push(cfg);
    }

    Ok(format!("Synced provider '{}'.", name))
}

#[tauri::command]
fn remove_provider(name: String, state: tauri::State<AppState>) -> Result<String, String> {
    let mut providers = state
        .providers
        .lock()
        .map_err(|_| "Provider state is unavailable.".to_string())?;
    let removed = providers.iter().find(|p| p.name == name).cloned();
    let before = providers.len();
    providers.retain(|p| p.name != name);
    if providers.len() < before {
        if let Some(p) = removed {
            delete_provider_secrets(&p);
        }
        Ok(format!("Removed '{}'.", name))
    } else {
        Err(format!("Provider '{}' was not found.", name))
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            run_with_selected_profile,
            toggle_provider,
            upsert_provider,
            remove_provider,
            get_secret_status,
            clear_provider_secret,
            secret_is_stored
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
