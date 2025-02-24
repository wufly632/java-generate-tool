// src/main.rs
use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_governor::Governor;

mod handlers;
mod configs;
mod git;
mod security;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let app_config = configs::load_config().expect("Failed to load config");
    
    HttpServer::new(move || {
        let governor_conf = actix_governor::GovernorConfigBuilder::default()
            .per_second(app_config.rate_limit.requests_per_second)
            .burst_size(app_config.rate_limit.burst_capacity)
            .finish()
            .unwrap();

        App::new()
            .app_data(web::Data::new(app_config.clone()))
            .wrap(Logger::default())
            .wrap(Governor::new(&governor_conf))
            .service(web::resource("/api/generate").to(handlers::generate_project))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
