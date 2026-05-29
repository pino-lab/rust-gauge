use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::metrics::MetricKind;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AppConfig {
    pub app_name: String,
    pub update_interval_ms: u64,
    pub enabled_metrics: Vec<MetricKind>,
    pub icon: IconConfig,
    pub thresholds: Thresholds,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct IconConfig {
    pub size: u32,
    pub low_color: String,
    pub medium_color: String,
    pub high_color: String,
    pub track_color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Thresholds {
    pub medium: f32,
    pub high: f32,
}

pub struct LoadedConfig {
    pub config: AppConfig,
    pub path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "RustGauge".to_string(),
            update_interval_ms: 1_000,
            enabled_metrics: vec![MetricKind::Cpu, MetricKind::Memory],
            icon: IconConfig::default(),
            thresholds: Thresholds::default(),
        }
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            size: 32,
            low_color: "#48c774".to_string(),
            medium_color: "#ffdd57".to_string(),
            high_color: "#f14668".to_string(),
            track_color: "#2b2f36".to_string(),
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            medium: 60.0,
            high: 85.0,
        }
    }
}

pub fn load_or_create_config() -> Result<LoadedConfig> {
    let path = config_path()?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory: {}", parent.display())
            })?;
        }

        let default_config = AppConfig::default();
        let text = toml::to_string_pretty(&default_config)
            .context("failed to serialize default config")?;
        fs::write(&path, text)
            .with_context(|| format!("failed to write default config: {}", path.display()))?;
    }

    Ok(LoadedConfig {
        config: load_config_from_path(&path)?,
        path,
    })
}

pub fn load_config_from_path(path: &Path) -> Result<AppConfig> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("failed to parse config: {}", path.display()))
}

pub fn save_config_to_path(path: &Path, config: &AppConfig) -> Result<()> {
    let text = toml::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(path, text).with_context(|| format!("failed to write config: {}", path.display()))
}

fn config_path() -> Result<PathBuf> {
    let local_path = std::env::current_dir()
        .context("failed to resolve current directory")?
        .join("rust-gauge.toml");

    if local_path.exists() {
        return Ok(local_path);
    }

    let project_dirs = ProjectDirs::from("dev", "RustGauge", "RustGauge")
        .context("failed to resolve user config directory")?;

    Ok(project_dirs.config_dir().join("config.toml"))
}
