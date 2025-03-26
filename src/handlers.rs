// src/handlers.rs
use std::{path::PathBuf, fs}; // Added fs for file operations
use walkdir::WalkDir; // Added walkdir
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
    pub nacos_data_id: String,
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
        "-b".to_string(), format!("master"),
        "--name".to_string(), req.name.clone(),
        "--destination".to_string(), target_dir.display().to_string(),
        "--define".to_string(), format!("project_name={}", req.name),
        "--define".to_string(), format!("package_name={}", req.package_name),
        "--define".to_string(), format!("project_class={}", req.project_class),
        "--define".to_string(), format!("server_port={}", req.server_port),
        "--define".to_string(), format!("nacos_data_id={}", req.nacos_data_id),
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

    // Recursively find and rename all 'tmp' directories based on package_name before committing to Codeup
    let project_base_path = target_dir.join(&req.name);
    let package_dir_relative_path = req.package_name.replace(".", "/");

    log::info!("Starting recursive search and rename for 'tmp' directories in {:?}", project_base_path);

    // fmt.Println("project_base_path: {:?}", project_base_path);
    let mut renamed_count = 0;
    let mut error_encountered = false;

    for entry in WalkDir::new(&project_base_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        // Check if it's a directory and the name is exactly "tmp"
        if path.is_dir() && path.file_name().map_or(false, |name| name == "coraool_project_tmp_dir") {
            let old_dir_path = path;
            // Construct the new path relative to the parent of the found 'tmp' directory
            if let Some(parent_path) = old_dir_path.parent() {
                let new_dir_path = parent_path.join(&package_dir_relative_path);

                log::info!("Found 'tmp' directory at {:?}. Attempting to rename to {:?}", old_dir_path, new_dir_path);

                // Ensure parent directories for the new path exist (relative to the tmp's parent)
                 if let Some(new_parent) = new_dir_path.parent() {
                     if !new_parent.exists() {
                         log::info!("Creating parent directories for target: {:?}", new_parent);
                         match fs::create_dir_all(new_parent) {
                             Ok(_) => log::info!("Successfully created parent directories {:?}", new_parent),
                             Err(e) => {
                                 log::error!("Failed to create parent directories {:?}: {}", new_parent, e);
                                 error_encountered = true; // Mark error and continue search if possible
                                 continue; // Skip renaming this instance
                             }
                         }
                     }
                 }

                 // Perform the rename operation
                 match fs::rename(&old_dir_path, &new_dir_path) {
                     Ok(_) => {
                         log::info!("Successfully renamed directory {:?} to {:?}", old_dir_path, new_dir_path);
                         renamed_count += 1;
                     }
                     Err(e) => {
                         log::error!("Failed to rename directory from {:?} to {:?}: {}", old_dir_path, new_dir_path, e);
                         error_encountered = true; // Mark error
                     }
                 }
            } else {
                log::warn!("Could not get parent path for {:?}, skipping rename.", old_dir_path);
            }
        }
    }

    if error_encountered {
         return Err("One or more errors occurred during directory renaming. Check logs for details.".into());
    } else {
      log::info!("Finished recursive rename. Renamed {} 'tmp' directories.", renamed_count);
    } // End of if/else for error_encountered

    // 提交到Codeup
    if let Some(repo_url) = &req.codeup_repo {
        log::info!("Pushing generated project to Codeup repository: {}", repo_url);
        git::push_to_codeup(
            &project_base_path, // Root of the generated project
            repo_url,
            &req.branch,
            &req.auth_token,
            &app_config.codeup
        ).await.map_err(|e| format!("Codeup push failed: {}", e))?;
        log::info!("Successfully pushed to Codeup.");
    } else {
        log::info!("No Codeup repository specified, skipping push.");
    }
    
    Ok(())
} // End of handle_generate_project function
