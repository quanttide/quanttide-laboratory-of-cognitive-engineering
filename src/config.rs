use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub base_url: String,
    pub model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "deepseek".to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let base = dirs::data_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".local").join("share"))
                .unwrap_or_else(|| PathBuf::from("/tmp/thinkcloud"))
        });
        Self {
            data_dir: base.join("thinkcloud"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub thought_window: usize,
    pub max_context_tokens: usize,
    pub thought_templates: Vec<String>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            thought_window: 10,
            max_context_tokens: 4096,
            thought_templates: vec![
                "复现步骤：".into(),
                "根因分析：".into(),
                "影响范围：".into(),
                "解决方案：".into(),
                "预期行为：".into(),
                "实际行为：".into(),
                "日志关键信息：".into(),
                "待确认问题：".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: AiConfig,
    pub storage: StorageConfig,
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AiConfig::default(),
            storage: StorageConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Config {
    pub fn api_key(&self) -> Option<String> {
        std::env::var("DEEPSEEK_API_KEY").ok()
    }

    pub fn load() -> crate::error::Result<Self> {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        let config_path = config_dir.join("thinkcloud").join("config.toml");

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content =
            std::fs::read_to_string(&config_path).map_err(|e| crate::error::ThinkCloudError::Other(format!("Failed to read config: {e}")))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::error::ThinkCloudError::Other(format!("Failed to parse config: {e}")))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ai.provider, "deepseek");
        assert_eq!(config.ai.model, "deepseek-chat");
        assert_eq!(config.ui.thought_window, 10);
        assert_eq!(config.ui.max_context_tokens, 4096);
        assert!(!config.ui.thought_templates.is_empty());
        assert_eq!(config.ui.thought_templates[0], "复现步骤：");
    }

    #[test]
    fn test_api_key_env_var() {
        let config = Config::default();

        let original = std::env::var("DEEPSEEK_API_KEY").ok();
        std::env::remove_var("DEEPSEEK_API_KEY");
        assert!(config.api_key().is_none());

        std::env::set_var("DEEPSEEK_API_KEY", "test-key");
        assert_eq!(config.api_key(), Some("test-key".to_string()));

        match original {
            Some(v) => std::env::set_var("DEEPSEEK_API_KEY", v),
            None => std::env::remove_var("DEEPSEEK_API_KEY"),
        }
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.ai.provider, config.ai.provider);
        assert_eq!(deserialized.ui.thought_window, config.ui.thought_window);
    }

    #[test]
    fn test_config_toml_format() {
        let toml_str = r#"
[ai]
provider = "ollama"
base_url = "http://localhost:11434"
model = "llama3"

[storage]
data_dir = "/tmp/thinkcloud"

[ui]
thought_window = 20
max_context_tokens = 2048
thought_templates = ["步骤：", "原因："]
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ai.provider, "ollama");
        assert_eq!(config.storage.data_dir.to_str().unwrap(), "/tmp/thinkcloud");
        assert_eq!(config.ui.thought_window, 20);
        assert_eq!(config.ui.max_context_tokens, 2048);
    }
}
