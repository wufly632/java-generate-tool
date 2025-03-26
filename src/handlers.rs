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
    pub auth_token: String,
}

fn default_branch() -> String {
    "main".to_string()
}

pub async fn handle_generate_project(
    req: GenerateRequest,
    app_config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // 验证模板白名单
    // if !app_config.generate.allowed_templates.contains(&req.template) {
    //     return Err("Template not allowed".into());
    // }

    // 解析目标路径
    let target_dir = req.target_dir
        .as_ref()
        .unwrap_or(&app_config.generate.default_target_dir)
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    // 路径安全检查
    // security::validate_path(&target_dir, &app_config.security.allowed_directories)?;

    // 以.拆分package_name
    let mut package_parts: Vec<&str> = req.package_name.split('.').collect();
    let _base_package = package_parts.first()
        .ok_or_else(|| format!("Invalid package name format"))?;

    // 执行cargo generate
    let mut cmd_args = vec![
        "generate".to_string(),
        "-g".to_string(), req.template.clone(),
        "-b".to_string(), format!("package{}", package_parts.len().to_string()),
        "--name".to_string(), req.name.clone(),
        "--destination".to_string(), target_dir.display().to_string(),
        "--define".to_string(), format!("project_name={}", req.name),
        "--define".to_string(), format!("package_name={}", req.package_name),
        "--define".to_string(), format!("project_class={}", req.project_class),
        "--define".to_string(), format!("server_port={}", req.server_port),
        "--define".to_string(), format!("package_dir={}", req.package_name.replace(".", "/"))
    ];
    for (i, part) in package_parts.iter_mut().enumerate() {
        cmd_args.push("--define".to_string());
        cmd_args.push(format!("package_dir{}={}", (i+1).to_string(), part.to_string()));
    }

    log::info!("Executing cargo command: cargo {}", cmd_args.join(" "));
    
    let output = tokio::process::Command::new("cargo")
        .args(&cmd_args)
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|e| format!("Failed to execute cargo command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Cargo command failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    // 提交到Codeup
    if let Some(repo_url) = &req.codeup_repo {
        git::push_to_codeup(
            &target_dir.join(&req.name),
            repo_url,
            &req.branch,
            &req.auth_token,
            &app_config.codeup
        ).await.map_err(|e| format!("Codeup push failed: {}", e))?;
    }
    Ok(())
}
