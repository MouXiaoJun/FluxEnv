//! OS-backed secret storage (Keychain / Credential Manager / Secret Service).
//! Values are never persisted in plaintext in app state — only in the platform store.

use keyring::Entry;

/// Must match `identifier` in tauri.conf.json for stable service name.
const SERVICE: &str = "com.fluxenv.desktop";

fn entry_key(provider_name: &str, env_key: &str) -> String {
    format!("v1|{provider_name}|{env_key}")
}

pub fn set_secret(provider_name: &str, env_key: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, &entry_key(provider_name, env_key)).map_err(|e| e.to_string())?;
    entry
        .set_password(value)
        .map_err(|e| format!("keyring set failed: {e}"))
}

pub fn get_secret(provider_name: &str, env_key: &str) -> Option<String> {
    let entry = Entry::new(SERVICE, &entry_key(provider_name, env_key)).ok()?;
    entry.get_password().ok()
}

pub fn delete_secret(provider_name: &str, env_key: &str) {
    if let Ok(entry) = Entry::new(SERVICE, &entry_key(provider_name, env_key)) {
        let _ = entry.delete_credential();
    }
}

pub fn has_secret(provider_name: &str, env_key: &str) -> bool {
    get_secret(provider_name, env_key)
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}
