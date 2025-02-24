// src/handlers.rs
use actix_web::{web, HttpResponse, HttpRequest};
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

pub async fn generate_project(
    req: web::Json<GenerateRequest>,
    app_config: web::Data<AppConfig>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {

    // 验证模板白名单
    if !app_config.generate.allowed_templates.contains(&req.template) {
        return Ok(HttpResponse::BadRequest().json(
            serde_json::json!({"error": "Template not allowed"})
        ));
    }

    // 解析目标路径
    let target_dir = req.target_dir
        .as_ref()
        .unwrap_or(&app_config.generate.default_target_dir)
        .canonicalize()
        .map_err(|e| {
            actix_web::error::ErrorBadRequest(format!("Invalid path: {}", e))
        })?;

    // 路径安全检查
    security::validate_path(&target_dir, &app_config.security.allowed_directories)
        .map_err(actix_web::error::ErrorForbidden)?;

    // 以.拆分package_name
    let mut package_parts: Vec<&str> = req.package_name.split('.').collect();
    let _base_package = package_parts.first().ok_or_else(|| 
        actix_web::error::ErrorBadRequest("Invalid package name format"))?.to_string();

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
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !output.status.success() {
        return Ok(HttpResponse::BadRequest().json(
            serde_json::json!({"error": String::from_utf8_lossy(&output.stderr)})
        ));
    }

    // 提交到Codeup
    if let Some(repo_url) = &req.codeup_repo {
        let auth_token = http_req.headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .map(|token| token.trim_start_matches("Bearer ").to_string())
            .ok_or_else(|| 
                actix_web::error::ErrorBadRequest("Missing authorization header"))?;

        git::push_to_codeup(
            &target_dir.join(&req.name),
            repo_url,
            &req.branch,
            &auth_token,
            &app_config.codeup
        ).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Codeup push failed: {}", e))
        })?;
    }

    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "path": target_dir.join(&req.name).display().to_string(),
        "codeup_url": req.codeup_repo.as_ref().map(|url| format!("{}/-/tree/{}", url, req.branch))
    })))
}
