use clap::{Parser, Args};
use std::path::PathBuf;
use serde_json::Value;

#[derive(Parser)]
#[command(name = "cargo-generate-service")]
#[command(about = "A tool for generating project from template")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Generate a new project from template
    Generate(GenerateArgs),
}

#[derive(Args)]
pub struct GenerateArgs {
    /// Template URL
    #[arg(short = 't', long)]
    pub template: String,

    /// Project name
    #[arg(short = 'n', long)]
    pub name: String,

    /// Target directory
    #[arg(short = 'd', long)]
    pub target_dir: Option<PathBuf>,

    /// Additional parameters in JSON format
    #[arg(short = 'p', long)]
    pub parameters: Option<String>,

    /// CodeUp repository
    #[arg(long)]
    pub codeup_repo: Option<String>,

    /// Branch name
    #[arg(short, long, default_value = "main")]
    pub branch: String,

    /// Package name
    #[arg(long)]
    pub package_name: String,

    /// Project class
    #[arg(long)]
    pub project_class: String,

    /// Server port
    #[arg(long)]
    pub server_port: String,
}

impl GenerateArgs {
    pub fn into_request(self) -> crate::handlers::GenerateRequest {
        crate::handlers::GenerateRequest {
            template: self.template,
            name: self.name,
            target_dir: self.target_dir,
            parameters: self.parameters.and_then(|p| serde_json::from_str(&p).ok()),
            codeup_repo: self.codeup_repo,
            branch: self.branch,
            package_name: self.package_name,
            project_class: self.project_class,
            server_port: self.server_port,
        }
    }
}