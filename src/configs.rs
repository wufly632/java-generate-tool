// src/configs.rs
use serde::Deserialize;
use std::{path::PathBuf, collections::HashSet};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub generate: GenerateConfig,
    pub codeup: CodeupConfig,
    pub security: SecurityConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateConfig {
    pub allowed_templates: HashSet<String>,
    pub max_project_size: String,
    pub timeout_seconds: u64,
    pub default_target_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeupConfig {
    pub default_repo_prefix: String,
    pub allowed_domains: HashSet<String>,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub allowed_directories: HashSet<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u64,
    pub burst_capacity: u32,
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config/default"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()?
        .try_deserialize()
}
