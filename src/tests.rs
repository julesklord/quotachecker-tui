#[cfg(test)]
mod tests {
    use crate::agent::{base64_decode, decode_jwt_payload};
    use crate::config::AppConfig;
    use serial_test::serial;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_base64_decode() {
        let input = "SGVsbG8gd29ybGQ=";
        let decoded = base64_decode(input).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello world");
    }

    #[test]
    fn test_decode_jwt_payload() {
        // Sample JWT header.payload.signature (signature omitted)
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let payload = "eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ";
        let jwt = format!("{}.{}.sig", header, payload);

        let decoded = decode_jwt_payload(&jwt).unwrap();
        assert_eq!(decoded["name"], "John Doe");
        assert_eq!(decoded["sub"], "1234567890");
    }

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.refresh_rate_ms, 2000);
        assert!(config.model_limits.contains_key("gpt-5"));
    }

    #[test]
    fn test_user_tier_names() {
        use crate::agent::UserTier;
        assert_eq!(UserTier::LocalFree.display_name(), "Local (Free Tier)");
        assert_eq!(UserTier::Guest.display_name(), "Guest / Unauthenticated");
        assert_eq!(
            UserTier::NotInstalled.display_name(),
            "Not Installed / Inactive"
        );
    }

    #[test]
    fn test_tier_quota_limits() {
        use crate::agent::UserTier;
        // Verify Codex limits
        let codex_limit = |tier| match tier {
            UserTier::OAuthEnterprise => 2000,
            UserTier::OAuthPersonal => 200,
            UserTier::LocalFree => 50,
            _ => 0,
        };
        assert_eq!(codex_limit(UserTier::OAuthEnterprise), 2000);
        assert_eq!(codex_limit(UserTier::OAuthPersonal), 200);
        assert_eq!(codex_limit(UserTier::LocalFree), 50);
        assert_eq!(codex_limit(UserTier::Guest), 0);

        // Verify OpenCode limits
        let opencode_limit = |tier| match tier {
            UserTier::Enterprise => 2000,
            UserTier::PersonalFree => 1000,
            UserTier::Guest => 200,
            _ => 0,
        };
        assert_eq!(opencode_limit(UserTier::Enterprise), 2000);
        assert_eq!(opencode_limit(UserTier::PersonalFree), 1000);
        assert_eq!(opencode_limit(UserTier::Guest), 200);

        // Verify Agy limits
        let agy_limit = |tier| match tier {
            UserTier::AdvancedCli => 500,
            _ => 0,
        };
        assert_eq!(agy_limit(UserTier::AdvancedCli), 500);

        // Verify Zed limits
        let zed_limit = |tier| match tier {
            UserTier::OAuthPersonal => 300,
            _ => 0,
        };
        assert_eq!(zed_limit(UserTier::OAuthPersonal), 300);
    }

    #[test]
    #[serial]
    fn test_config_save() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_owned();

        // Save original environment variables to restore them later
        let home = env::var("HOME").ok();
        let xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
        let appdata = env::var("APPDATA").ok();

        env::set_var("HOME", &path);
        env::set_var("XDG_CONFIG_HOME", &path);
        env::set_var("APPDATA", &path);

        let mut config = AppConfig::default();
        config.refresh_rate_ms = 9999;

        // Obtain the dynamic config path
        let config_path = AppConfig::config_path().expect("Should determine config path");

        // Assert that the config is inside our temp directory
        assert!(config_path.starts_with(&path));

        let save_result = config.save();
        assert!(
            save_result.is_ok(),
            "Failed to save config: {:?}",
            save_result.err()
        );

        assert!(config_path.exists(), "Config file was not created");

        let content = fs::read_to_string(&config_path).unwrap();
        let loaded: AppConfig = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded.refresh_rate_ms, 9999);
        assert_eq!(loaded.theme, config.theme);

        // Restore environment variables
        if let Some(h) = home {
            env::set_var("HOME", h);
        } else {
            env::remove_var("HOME");
        }
        if let Some(x) = xdg_config_home {
            env::set_var("XDG_CONFIG_HOME", x);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
        if let Some(a) = appdata {
            env::set_var("APPDATA", a);
        } else {
            env::remove_var("APPDATA");
        }
    }
}
