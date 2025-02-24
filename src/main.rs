// src/main.rs
use clap::Parser;

mod handlers;
mod configs;
mod git;
mod security;
mod cli;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let app_config = configs::load_config().expect("Failed to load config");
    
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Generate(args) => {
            let request = args.into_request();
            match handlers::handle_generate_project(request, &app_config).await {
                Ok(_) => {
                    println!("Project generated successfully");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error generating project: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
