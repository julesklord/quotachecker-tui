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
    fn test_calculate_seconds_monthly_reset_mid_month() {
        use crate::agent::calculate_seconds_until_monthly_reset;
        use chrono::{Local, TimeZone};

        let naive = chrono::NaiveDate::from_ymd_opt(2023, 5, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let now = Local.from_local_datetime(&naive).single().unwrap();

        let naive_next = chrono::NaiveDate::from_ymd_opt(2023, 6, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let expected_reset = Local.from_local_datetime(&naive_next).single().unwrap();

        let expected_seconds = expected_reset.signed_duration_since(now).num_seconds();
        assert_eq!(calculate_seconds_until_monthly_reset(now), expected_seconds);
    }

    #[test]
    fn test_calculate_seconds_monthly_reset_year_boundary() {
        use crate::agent::calculate_seconds_until_monthly_reset;
        use chrono::{Local, TimeZone};

        let naive = chrono::NaiveDate::from_ymd_opt(2023, 12, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let now = Local.from_local_datetime(&naive).single().unwrap();

        let naive_next = chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let expected_reset = Local.from_local_datetime(&naive_next).single().unwrap();

        let expected_seconds = expected_reset.signed_duration_since(now).num_seconds();
        assert_eq!(calculate_seconds_until_monthly_reset(now), expected_seconds);
    }
}
