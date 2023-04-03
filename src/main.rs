use mailbolt::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Init telemetry subscriber to process tracing spans and logs
    let subscriber = get_subscriber("mailbolt".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Could not read configuration file");
    let conn_pool = PgPool::connect(config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.app_port))
        .unwrap_or_else(|_| panic!("Could not bind server to port '{}'", config.app_port));

    tracing::info!("Server started on port {}", config.app_port);

    run(listener, conn_pool)?.await
}
