// src/handlers.rs
use std::path::PathBuf;
use crate::{git, security, configs::AppConfig};

#[derive(serde::Deserialize)]
pub struct GenerateRequest {
    pub template: String,
    pub name: String,
    #[serde(default)]
    pub target_dir: Option<std::path::PathBuf>,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
    #[serde(default)]
    pub codeup_repo: Option<String>,
    #[serde(default = "default_branch")]
    pub branch: String,
    pub package_name: String,
    pub project_class: String,
    pub server_port: String,
}

fn default_branch() -> String {
    "main".to_string()
}

pub async fn handle_generate_project(
    req: GenerateRequest,
    app_config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // 验证模板白名单
    if !app_config.generate.allowed_templates.contains(&req.template) {
        return Err("Template not allowed".into());
    }

    // 解析目标路径
    let target_dir = req.target_dir
        .as_ref()
        .unwrap_or(&app_config.generate.default_target_dir)
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    // 路径安全检查
    security::validate_path(&target_dir, &app_config.security.allowed_directories)?;

    // 克隆模板仓库
    git::clone_template(&req.template, &target_dir, &req.branch).await?;

    // 如果提供了CodeUp仓库，则设置远程仓库
    if let Some(codeup_repo) = req.codeup_repo {
        git::setup_codeup_remote(&target_dir, &codeup_repo, &app_config.codeup).await?;
    }

    Ok(())
}
