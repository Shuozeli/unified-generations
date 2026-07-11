use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AgentPlanError, Result};

pub const DEFAULT_PLAN_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/plan";
pub const DEFAULT_TTS_URL: &str = "https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional";
pub const DEFAULT_TTS_RESOURCE_ID: &str = "seed-tts-2.0";
pub const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkCliConfig {
    pub api_key: String,

    #[serde(default = "default_plan_base_url")]
    pub plan_base_url: String,

    #[serde(default = "default_tts_url")]
    pub tts_url: String,

    #[serde(default = "default_tts_resource_id")]
    pub tts_resource_id: String,

    #[serde(default = "default_anthropic_version")]
    pub anthropic_version: String,
}

impl ArkCliConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            plan_base_url: DEFAULT_PLAN_BASE_URL.to_string(),
            tts_url: DEFAULT_TTS_URL.to_string(),
            tts_resource_id: DEFAULT_TTS_RESOURCE_ID.to_string(),
            anthropic_version: DEFAULT_ANTHROPIC_VERSION.to_string(),
        }
    }
}

pub fn arkcli_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or(AgentPlanError::MissingHome)?;
    Ok(PathBuf::from(home).join(".arkcli"))
}

pub fn arkcli_config_path() -> Result<PathBuf> {
    Ok(arkcli_dir()?.join("config.toml"))
}

pub fn load_arkcli_config() -> Result<Option<ArkCliConfig>> {
    let path = arkcli_config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)?;
    Ok(Some(toml::from_str(&text)?))
}

pub fn write_arkcli_config(config: &ArkCliConfig) -> Result<PathBuf> {
    let dir = arkcli_dir()?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config.toml");
    std::fs::write(&path, toml::to_string_pretty(config)?)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(path)
}

pub fn masked_api_key(api_key: &str) -> String {
    let trimmed = api_key.trim();
    if trimmed.len() <= 10 {
        return "********".to_string();
    }

    let prefix = &trimmed[..trimmed.len().min(6)];
    let suffix_start = trimmed.len().saturating_sub(4);
    format!("{prefix}...{}", &trimmed[suffix_start..])
}

fn default_plan_base_url() -> String {
    DEFAULT_PLAN_BASE_URL.to_string()
}

fn default_tts_url() -> String {
    DEFAULT_TTS_URL.to_string()
}

fn default_tts_resource_id() -> String {
    DEFAULT_TTS_RESOURCE_ID.to_string()
}

fn default_anthropic_version() -> String {
    DEFAULT_ANTHROPIC_VERSION.to_string()
}
