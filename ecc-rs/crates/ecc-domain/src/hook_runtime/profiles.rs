use std::collections::HashSet;

/// Hook execution profile — controls which hooks run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookProfile {
    Minimal,
    Standard,
    Strict,
}

impl HookProfile {
    /// Parse a profile name string into a HookProfile variant.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "minimal" => Some(Self::Minimal),
            "standard" => Some(Self::Standard),
            "strict" => Some(Self::Strict),
            _ => None,
        }
    }

    /// Return the string name of this profile.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Strict => "strict",
        }
    }
}

/// Normalize a hook ID to lowercase trimmed string.
pub fn normalize_id(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Get the active hook profile from an env value, defaulting to Standard.
pub fn get_hook_profile(env_value: Option<&str>) -> HookProfile {
    let raw = env_value.unwrap_or("standard");
    HookProfile::from_str_opt(raw).unwrap_or(HookProfile::Standard)
}

/// Parse a disabled hooks env value into a set of normalized hook IDs.
pub fn get_disabled_hook_ids(env_value: Option<&str>) -> HashSet<String> {
    let raw = match env_value {
        Some(v) if !v.trim().is_empty() => v,
        _ => return HashSet::new(),
    };

    raw.split(',')
        .map(normalize_id)
        .filter(|v| !v.is_empty())
        .collect()
}

/// Default fallback profiles when none are specified.
const DEFAULT_FALLBACK: &[HookProfile] = &[HookProfile::Standard, HookProfile::Strict];

/// Parse a comma-separated string of profile names into validated profiles.
/// Returns fallback if input is empty or contains no valid profiles.
pub fn parse_profiles(raw: Option<&str>, fallback: Option<&[HookProfile]>) -> Vec<HookProfile> {
    let fallback = fallback.unwrap_or(DEFAULT_FALLBACK);

    let raw = match raw {
        Some(v) if !v.trim().is_empty() => v,
        _ => return fallback.to_vec(),
    };

    let parsed: Vec<HookProfile> = raw
        .split(',')
        .filter_map(HookProfile::from_str_opt)
        .collect();

    if parsed.is_empty() {
        fallback.to_vec()
    } else {
        parsed
    }
}

/// Parse a list of profile name strings into validated profiles.
/// Returns fallback if input is empty or contains no valid profiles.
pub fn parse_profiles_list(raw: &[&str], fallback: Option<&[HookProfile]>) -> Vec<HookProfile> {
    let fallback = fallback.unwrap_or(DEFAULT_FALLBACK);

    let parsed: Vec<HookProfile> = raw
        .iter()
        .filter_map(|v| HookProfile::from_str_opt(v))
        .collect();

    if parsed.is_empty() {
        fallback.to_vec()
    } else {
        parsed
    }
}

/// Options for checking if a hook should execute.
#[derive(Default)]
pub struct HookEnabledOptions<'a> {
    /// Comma-separated profiles string, or None for default.
    pub profiles: Option<&'a str>,
}

/// Check if a hook is enabled based on its ID, disabled list, and active profile.
/// `profile_env` is the ECC_HOOK_PROFILE env value.
/// `disabled_env` is the ECC_DISABLED_HOOKS env value.
pub fn is_hook_enabled(
    hook_id: &str,
    profile_env: Option<&str>,
    disabled_env: Option<&str>,
    options: &HookEnabledOptions<'_>,
) -> bool {
    let id = normalize_id(hook_id);
    if id.is_empty() {
        return true;
    }

    let disabled = get_disabled_hook_ids(disabled_env);
    if disabled.contains(&id) {
        return false;
    }

    let profile = get_hook_profile(profile_env);
    let allowed = parse_profiles(options.profiles, None);
    allowed.contains(&profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- normalize_id ---

    #[test]
    fn normalize_id_lowercases() {
        assert_eq!(normalize_id("MyHook"), "myhook");
    }

    #[test]
    fn normalize_id_trims() {
        assert_eq!(normalize_id("  spaced  "), "spaced");
    }

    #[test]
    fn normalize_id_empty() {
        assert_eq!(normalize_id(""), "");
    }

    #[test]
    fn normalize_id_whitespace_only() {
        assert_eq!(normalize_id("   "), "");
    }

    #[test]
    fn normalize_id_already_lowercase() {
        assert_eq!(normalize_id("already"), "already");
    }

    // --- HookProfile ---

    #[test]
    fn profile_from_str_minimal() {
        assert_eq!(HookProfile::from_str_opt("minimal"), Some(HookProfile::Minimal));
    }

    #[test]
    fn profile_from_str_standard() {
        assert_eq!(HookProfile::from_str_opt("standard"), Some(HookProfile::Standard));
    }

    #[test]
    fn profile_from_str_strict() {
        assert_eq!(HookProfile::from_str_opt("strict"), Some(HookProfile::Strict));
    }

    #[test]
    fn profile_from_str_case_insensitive() {
        assert_eq!(HookProfile::from_str_opt("MINIMAL"), Some(HookProfile::Minimal));
        assert_eq!(HookProfile::from_str_opt("Standard"), Some(HookProfile::Standard));
    }

    #[test]
    fn profile_from_str_with_whitespace() {
        assert_eq!(HookProfile::from_str_opt("  strict  "), Some(HookProfile::Strict));
    }

    #[test]
    fn profile_from_str_invalid() {
        assert_eq!(HookProfile::from_str_opt("invalid"), None);
        assert_eq!(HookProfile::from_str_opt(""), None);
    }

    #[test]
    fn profile_as_str() {
        assert_eq!(HookProfile::Minimal.as_str(), "minimal");
        assert_eq!(HookProfile::Standard.as_str(), "standard");
        assert_eq!(HookProfile::Strict.as_str(), "strict");
    }

    // --- get_hook_profile ---

    #[test]
    fn get_hook_profile_none_defaults_to_standard() {
        assert_eq!(get_hook_profile(None), HookProfile::Standard);
    }

    #[test]
    fn get_hook_profile_valid() {
        assert_eq!(get_hook_profile(Some("minimal")), HookProfile::Minimal);
        assert_eq!(get_hook_profile(Some("strict")), HookProfile::Strict);
    }

    #[test]
    fn get_hook_profile_invalid_defaults_to_standard() {
        assert_eq!(get_hook_profile(Some("bogus")), HookProfile::Standard);
    }

    #[test]
    fn get_hook_profile_empty_defaults_to_standard() {
        assert_eq!(get_hook_profile(Some("")), HookProfile::Standard);
    }

    #[test]
    fn get_hook_profile_case_insensitive() {
        assert_eq!(get_hook_profile(Some("STRICT")), HookProfile::Strict);
    }

    // --- get_disabled_hook_ids ---

    #[test]
    fn disabled_hooks_none() {
        assert!(get_disabled_hook_ids(None).is_empty());
    }

    #[test]
    fn disabled_hooks_empty() {
        assert!(get_disabled_hook_ids(Some("")).is_empty());
    }

    #[test]
    fn disabled_hooks_whitespace() {
        assert!(get_disabled_hook_ids(Some("   ")).is_empty());
    }

    #[test]
    fn disabled_hooks_single() {
        let ids = get_disabled_hook_ids(Some("my-hook"));
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("my-hook"));
    }

    #[test]
    fn disabled_hooks_multiple() {
        let ids = get_disabled_hook_ids(Some("hook-a,hook-b,hook-c"));
        assert_eq!(ids.len(), 3);
        assert!(ids.contains("hook-a"));
        assert!(ids.contains("hook-b"));
        assert!(ids.contains("hook-c"));
    }

    #[test]
    fn disabled_hooks_normalizes() {
        let ids = get_disabled_hook_ids(Some("HOOK-A, Hook-B "));
        assert!(ids.contains("hook-a"));
        assert!(ids.contains("hook-b"));
    }

    #[test]
    fn disabled_hooks_filters_empty() {
        let ids = get_disabled_hook_ids(Some("a,,b,"));
        assert_eq!(ids.len(), 2);
    }

    // --- parse_profiles ---

    #[test]
    fn parse_profiles_none_returns_fallback() {
        let result = parse_profiles(None, None);
        assert_eq!(result, vec![HookProfile::Standard, HookProfile::Strict]);
    }

    #[test]
    fn parse_profiles_empty_returns_fallback() {
        let result = parse_profiles(Some(""), None);
        assert_eq!(result, vec![HookProfile::Standard, HookProfile::Strict]);
    }

    #[test]
    fn parse_profiles_single_valid() {
        let result = parse_profiles(Some("minimal"), None);
        assert_eq!(result, vec![HookProfile::Minimal]);
    }

    #[test]
    fn parse_profiles_multiple_valid() {
        let result = parse_profiles(Some("minimal,strict"), None);
        assert_eq!(result, vec![HookProfile::Minimal, HookProfile::Strict]);
    }

    #[test]
    fn parse_profiles_filters_invalid() {
        let result = parse_profiles(Some("minimal,bogus,strict"), None);
        assert_eq!(result, vec![HookProfile::Minimal, HookProfile::Strict]);
    }

    #[test]
    fn parse_profiles_all_invalid_returns_fallback() {
        let result = parse_profiles(Some("bogus,nope"), None);
        assert_eq!(result, vec![HookProfile::Standard, HookProfile::Strict]);
    }

    #[test]
    fn parse_profiles_custom_fallback() {
        let fallback = [HookProfile::Minimal];
        let result = parse_profiles(Some("bogus"), Some(&fallback));
        assert_eq!(result, vec![HookProfile::Minimal]);
    }

    #[test]
    fn parse_profiles_case_insensitive() {
        let result = parse_profiles(Some("MINIMAL,Standard"), None);
        assert_eq!(result, vec![HookProfile::Minimal, HookProfile::Standard]);
    }

    // --- parse_profiles_list ---

    #[test]
    fn parse_profiles_list_valid() {
        let result = parse_profiles_list(&["minimal", "strict"], None);
        assert_eq!(result, vec![HookProfile::Minimal, HookProfile::Strict]);
    }

    #[test]
    fn parse_profiles_list_empty_returns_fallback() {
        let result = parse_profiles_list(&[], None);
        assert_eq!(result, vec![HookProfile::Standard, HookProfile::Strict]);
    }

    #[test]
    fn parse_profiles_list_filters_invalid() {
        let result = parse_profiles_list(&["minimal", "bogus"], None);
        assert_eq!(result, vec![HookProfile::Minimal]);
    }

    // --- is_hook_enabled ---

    #[test]
    fn enabled_default_standard_profile() {
        // Default profile is standard, default allowed is [standard, strict]
        assert!(is_hook_enabled("my-hook", None, None, &HookEnabledOptions::default()));
    }

    #[test]
    fn enabled_empty_id_always_true() {
        assert!(is_hook_enabled("", None, None, &HookEnabledOptions::default()));
    }

    #[test]
    fn enabled_whitespace_id_always_true() {
        assert!(is_hook_enabled("   ", None, None, &HookEnabledOptions::default()));
    }

    #[test]
    fn disabled_by_env() {
        assert!(!is_hook_enabled("my-hook", None, Some("my-hook"), &HookEnabledOptions::default()));
    }

    #[test]
    fn disabled_by_env_case_insensitive() {
        assert!(!is_hook_enabled("MY-HOOK", None, Some("my-hook"), &HookEnabledOptions::default()));
    }

    #[test]
    fn disabled_by_profile_mismatch() {
        // Profile is minimal, but allowed is only [strict]
        let opts = HookEnabledOptions {
            profiles: Some("strict"),
        };
        assert!(!is_hook_enabled("my-hook", Some("minimal"), None, &opts));
    }

    #[test]
    fn enabled_by_profile_match() {
        let opts = HookEnabledOptions {
            profiles: Some("minimal,standard"),
        };
        assert!(is_hook_enabled("my-hook", Some("minimal"), None, &opts));
    }

    #[test]
    fn disabled_takes_priority_over_profile() {
        let opts = HookEnabledOptions {
            profiles: Some("standard"),
        };
        assert!(!is_hook_enabled("my-hook", Some("standard"), Some("my-hook"), &opts));
    }
}
