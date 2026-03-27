use fluxenv_models::{EnvVariable, Profile, ProviderConfig};

pub fn merge_profile_variables(base: &[EnvVariable], profile: &Profile) -> Vec<EnvVariable> {
    let mut merged = base.to_vec();

    for var in &profile.variables {
        if let Some(existing) = merged.iter_mut().find(|v| v.key == var.key) {
            *existing = var.clone();
        } else {
            merged.push(var.clone());
        }
    }

    merged
}

pub fn build_effective_session_env(
    system_env: &[EnvVariable],
    profile: &Profile,
    providers: &[ProviderConfig],
) -> Vec<EnvVariable> {
    let mut merged = merge_profile_variables(system_env, profile);

    for provider in providers.iter().filter(|p| p.enabled) {
        for var in &provider.variables {
            if let Some(existing) = merged.iter_mut().find(|v| v.key == var.key) {
                *existing = var.clone();
            } else {
                merged.push(var.clone());
            }
        }
    }

    merged
}

pub fn set_provider_enabled(providers: &mut [ProviderConfig], provider_name: &str, enabled: bool) -> bool {
    if let Some(provider) = providers.iter_mut().find(|p| p.name == provider_name) {
        provider.enabled = enabled;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{build_effective_session_env, merge_profile_variables, set_provider_enabled};
    use fluxenv_models::{EnvVariable, Profile, ProviderConfig};

    #[test]
    fn profile_values_override_base_values() {
        let base = vec![EnvVariable {
            key: "API_URL".to_string(),
            value: "http://localhost".to_string(),
            is_secret: false,
        }];

        let profile = Profile {
            name: "staging".to_string(),
            variables: vec![EnvVariable {
                key: "API_URL".to_string(),
                value: "https://staging.example.com".to_string(),
                is_secret: false,
            }],
        };

        let merged = merge_profile_variables(&base, &profile);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].value, "https://staging.example.com");
    }

    #[test]
    fn enabled_provider_overrides_profile_and_system_values() {
        let system_env = vec![EnvVariable {
            key: "OPENAI_API_KEY".to_string(),
            value: "system-key".to_string(),
            is_secret: true,
        }];
        let profile = Profile {
            name: "dev".to_string(),
            variables: vec![EnvVariable {
                key: "OPENAI_API_KEY".to_string(),
                value: "profile-key".to_string(),
                is_secret: true,
            }],
        };
        let providers = vec![ProviderConfig {
            name: "openrouter".to_string(),
            enabled: true,
            variables: vec![EnvVariable {
                key: "OPENAI_API_KEY".to_string(),
                value: "provider-key".to_string(),
                is_secret: true,
            }],
        }];

        let effective = build_effective_session_env(&system_env, &profile, &providers);
        assert_eq!(effective.len(), 1);
        assert_eq!(effective[0].value, "provider-key");
    }

    #[test]
    fn disabled_provider_does_not_take_effect() {
        let system_env = vec![EnvVariable {
            key: "ANTHROPIC_API_KEY".to_string(),
            value: "system-anthropic".to_string(),
            is_secret: true,
        }];
        let profile = Profile {
            name: "dev".to_string(),
            variables: vec![],
        };
        let providers = vec![ProviderConfig {
            name: "anthropic".to_string(),
            enabled: false,
            variables: vec![EnvVariable {
                key: "ANTHROPIC_API_KEY".to_string(),
                value: "provider-anthropic".to_string(),
                is_secret: true,
            }],
        }];

        let effective = build_effective_session_env(&system_env, &profile, &providers);
        assert_eq!(effective[0].value, "system-anthropic");
    }

    #[test]
    fn toggling_provider_changes_effective_output() {
        let system_env = vec![EnvVariable {
            key: "DEEPSEEK_API_KEY".to_string(),
            value: "system-deepseek".to_string(),
            is_secret: true,
        }];
        let profile = Profile {
            name: "dev".to_string(),
            variables: vec![],
        };
        let mut providers = vec![ProviderConfig {
            name: "deepseek".to_string(),
            enabled: false,
            variables: vec![EnvVariable {
                key: "DEEPSEEK_API_KEY".to_string(),
                value: "provider-deepseek".to_string(),
                is_secret: true,
            }],
        }];

        let before = build_effective_session_env(&system_env, &profile, &providers);
        assert_eq!(before[0].value, "system-deepseek");

        assert!(set_provider_enabled(&mut providers, "deepseek", true));
        let after = build_effective_session_env(&system_env, &profile, &providers);
        assert_eq!(after[0].value, "provider-deepseek");
    }
}
