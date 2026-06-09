use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiTheme {
    #[default]
    Cyan,
    Purple,
    Emerald,
    Amber,
    Monochrome,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentQuotaSettings {
    pub limit: u32,
}

impl Default for AgentQuotaSettings {
    fn default() -> Self {
        Self { limit: 100 }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub refresh_rate_ms: u64,
    pub soft_limit_percent: f64,
    pub hard_limit_percent: f64,
    pub theme: TuiTheme,
    pub codex_quota: AgentQuotaSettings,
    pub opencode_quota: AgentQuotaSettings,
    pub gemini_quota: AgentQuotaSettings,
    pub agy_quota: AgentQuotaSettings,
    pub zed_quota: AgentQuotaSettings,
    pub model_limits: HashMap<String, u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut model_limits = HashMap::new();
        model_limits.insert("gpt-5".to_string(), 50);
        model_limits.insert("gpt-4.1".to_string(), 100);
        model_limits.insert("gpt-4o".to_string(), 50);
        model_limits.insert("gpt-4o-mini".to_string(), 200);
        model_limits.insert("o3".to_string(), 30);
        model_limits.insert("o4-mini".to_string(), 100);
        model_limits.insert("deepseek-chat".to_string(), 150);
        model_limits.insert("deepseek-reasoner".to_string(), 50);
        model_limits.insert("gpt-oss".to_string(), 100);
        model_limits.insert("gemini-3.5-flash".to_string(), 1500);
        model_limits.insert("gemini-3.1-pro".to_string(), 50);
        model_limits.insert("Gemini 3.5 Flash".to_string(), 1500);
        model_limits.insert("Gemini 3.1 Pro".to_string(), 50);
        model_limits.insert("claude-4.7".to_string(), 150);

        Self {
            refresh_rate_ms: 2000,
            soft_limit_percent: 80.0,
            hard_limit_percent: 100.0,
            theme: TuiTheme::Cyan,
            codex_quota: AgentQuotaSettings { limit: 200 },
            opencode_quota: AgentQuotaSettings { limit: 1000 },
            gemini_quota: AgentQuotaSettings { limit: 2000 },
            agy_quota: AgentQuotaSettings { limit: 500 },
            zed_quota: AgentQuotaSettings { limit: 300 },
            model_limits,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "quotachecker-tui", "dashboard")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                        // Ensure newly added fields are initialized if they were absent in legacy JSON
                        if config.model_limits.is_empty() {
                            config.model_limits = Self::default().model_limits;
                            let _ = config.save();
                        }
                        return config;
                    }
                }
            }
        }
        
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }
}
