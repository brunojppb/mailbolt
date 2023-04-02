use env_logger::Env;
use mailbolt::{configuration::get_configuration, startup::run};
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Init our global logger.
    // Let env_logger call `set_logger` internally so it's all setup from here.
    // To bump log levels, change the RUST_LOG env variable.
    // e.g. RUST_LOG=debug cargo run
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = get_configuration().expect("Could not read configuration file");
    let conn_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.app_port))
        .unwrap_or_else(|_| panic!("Could not bind server to port '{}'", config.app_port));

    log::info!("Server started on port {}", config.app_port);

    run(listener, conn_pool)?.await
}
