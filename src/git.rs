// src/git.rs
use crate::configs::CodeupConfig;
use tokio::process::Command;
use url::Url;
use std::path::Path;

pub async fn push_to_codeup(
    project_path: &Path,
    repo_url: &str,
    branch: &str,
    auth_token: &str,
    config: &CodeupConfig,
) -> Result<(), String> {
    // 验证仓库地址
    // validate_repo_url(repo_url, config)?;

    let parsed_url = Url::parse(repo_url)
        .map_err(|_| "Invalid repository URL".to_string())?;
    let repo_with_token = format!(
        "https://oauth2:{}@{}{}",
        auth_token,
        parsed_url.host_str().unwrap_or(""),
        parsed_url.path()
    );
    // let repo_with_token = repo_url
    //     .replace("https://", &format!("https://oauth2:{}@", auth_token));
    log::info!("repo_with_token {}", repo_with_token);

    let branch_string = branch.to_string();

    let commands = [
        ("init", vec![]),
        ("config", vec!["user.name", "flynn"]),
        ("config", vec!["user.email", "fei.wu@coraool.com"]),
        ("add", vec!["."]),
        ("commit", vec!["-m", "Initial commit"]),
        ("remote", vec!["add", "origin", &repo_with_token]),
        ("checkout", vec!["-b", "main"]),
        ("push", vec!["-u", "origin", "main"]),
    ];

    for (cmd, args) in commands {
        log::info!("Executing cargo command: git {}", args.join(" "));
        let status = Command::new("git")
            .current_dir(project_path)
            .arg(cmd)
            .args(args)
            .status()
            .await
            .map_err(|e| format!("Git command failed: {}", e))?;

        if !status.success() {
            return Err(format!("Git command failed: git {}", cmd));
        }
    }

    Ok(())
}

pub async fn clone_template(template_url: &str, target_dir: &Path, branch: &str) -> Result<(), String> {
    log::info!("Cloning template from {} to {:?}", template_url, target_dir);

    let status = Command::new("git")
        .arg("clone")
        .arg("--branch")
        .arg(branch)
        .arg(template_url)
        .arg(target_dir)
        .status()
        .await
        .map_err(|e| format!("Failed to clone template: {}", e))?;

    if !status.success() {
        return Err("Failed to clone template repository".into());
    }

    Ok(())
}

pub async fn setup_codeup_remote(target_dir: &Path, repo_url: &str, config: &CodeupConfig) -> Result<(), String> {
    // 验证仓库地址
    validate_repo_url(repo_url, config)?;

    let status = Command::new("git")
        .current_dir(target_dir)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(repo_url)
        .status()
        .await
        .map_err(|e| format!("Failed to setup remote: {}", e))?;

    if !status.success() {
        return Err("Failed to setup remote repository".into());
    }

    Ok(())
}

fn validate_repo_url(url: &str, config: &CodeupConfig) -> Result<(), String> {
    let parsed = Url::parse(url)
        .map_err(|_| "Invalid repository URL".to_string())?;

    // 验证域名白名单
    if !config.allowed_domains.contains(parsed.host_str().unwrap_or("")) {
        return Err("Domain not allowed".into());
    }

    // 验证路径前缀
    if !url.starts_with(&config.default_repo_prefix) {
        return Err("Repository path not allowed".into());
    }

    Ok(())
}
