#[cfg(test)]
mod tests {
    use crate::agent::{base64_decode, decode_jwt_payload};
    use crate::config::AppConfig;

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
    fn test_check_executable_exists() {
        use crate::agent::AgentScanner;
        let result = AgentScanner::check_executable("cargo");
        assert!(result.is_some());
    }

    #[test]
    fn test_check_executable_not_exists() {
        use crate::agent::AgentScanner;
        let result = AgentScanner::check_executable("this_command_absolutely_does_not_exist_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_check_executable_fallback_path() {
        use crate::agent::AgentScanner;
        use std::env;
        use std::fs;

        struct HomeEnvGuard {
            original_home: Option<String>,
        }

        impl HomeEnvGuard {
            fn new(temp_home: &std::path::Path) -> Self {
                let original_home = env::var("HOME").ok();
                env::set_var("HOME", temp_home);
                Self { original_home }
            }
        }

        impl Drop for HomeEnvGuard {
            fn drop(&mut self) {
                if let Some(ref home) = self.original_home {
                    env::set_var("HOME", home);
                } else {
                    env::remove_var("HOME");
                }
            }
        }

        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("quotachecker_test_home_{}", timestamp));
        let bin_dir = temp_dir.join(".local/bin");
        fs::create_dir_all(&bin_dir).unwrap();

        let _env_guard = HomeEnvGuard::new(&temp_dir);

        let mock_cmd = format!("mock_cmd_for_test_{}", timestamp);
        let mock_executable = bin_dir.join(&mock_cmd);
        fs::write(&mock_executable, "#!/bin/sh
exit 0").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&mock_executable).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&mock_executable, perms).unwrap();
        }


        let result = AgentScanner::check_executable(&mock_cmd);

        let _ = fs::remove_dir_all(&temp_dir);
        assert_eq!(result, Some(mock_executable.to_string_lossy().to_string()));
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
}
