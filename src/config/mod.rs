use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_MAX_LOG_LINES: usize = 200;
const DEFAULT_THEME: &str = "dark";
const DEFAULT_API_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_PROVIDER: &str = "anthropic";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_key: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_max_log_lines")]
    pub max_log_lines: usize,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub telemetry: bool,
}

fn default_model() -> String {
    DEFAULT_MODEL.to_string()
}

fn default_max_log_lines() -> usize {
    DEFAULT_MAX_LOG_LINES
}

fn default_theme() -> String {
    DEFAULT_THEME.to_string()
}

fn default_api_base_url() -> String {
    DEFAULT_API_BASE_URL.to_string()
}

fn default_provider() -> String {
    DEFAULT_PROVIDER.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            model: default_model(),
            max_log_lines: default_max_log_lines(),
            theme: default_theme(),
            api_base_url: default_api_base_url(),
            provider: default_provider(),
            telemetry: false,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .context("Could not determine home directory")?
            .join(".sherlog");
        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

    /// Get the API key, preferring the environment variable over the config file.
    pub fn api_key(&self) -> Option<String> {
        std::env::var("SHERLOG_API_KEY")
            .ok()
            .or_else(|| self.api_key.clone())
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "api_key" => self.api_key = Some(value.to_string()),
            "model" => self.model = value.to_string(),
            "max_log_lines" => {
                self.max_log_lines = value
                    .parse()
                    .context("max_log_lines must be a positive integer")?;
            }
            "theme" => self.theme = value.to_string(),
            "api_base_url" => self.api_base_url = value.to_string(),
            "provider" => match value {
                "anthropic" | "ollama" => {
                    self.provider = value.to_string();
                    // Auto-switch api_base_url when changing providers
                    if value == "ollama" && self.api_base_url == DEFAULT_API_BASE_URL {
                        self.api_base_url = "http://localhost:11434".to_string();
                    } else if value == "anthropic"
                        && self.api_base_url == "http://localhost:11434"
                    {
                        self.api_base_url = DEFAULT_API_BASE_URL.to_string();
                    }
                }
                _ => anyhow::bail!(
                    "Unknown provider: '{}'. Valid providers: anthropic, ollama",
                    value
                ),
            },
            "telemetry" => {
                self.telemetry = value
                    .parse()
                    .context("telemetry must be 'true' or 'false'")?;
            }
            _ => anyhow::bail!(
                "Unknown config key: '{}'. Valid keys: api_key, model, max_log_lines, theme, api_base_url, provider, telemetry",
                key
            ),
        }
        Ok(())
    }
}
