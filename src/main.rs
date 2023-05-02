use mailbolt::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::{net::TcpListener, time::Duration};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Init telemetry subscriber to process tracing spans and logs
    let subscriber = get_subscriber("mailbolt".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Could not read configuration file");

    let conn_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(config.database.connection_string().expose_secret())
        .expect("Failed to create a Postgres connection pool");

    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))
    .unwrap_or_else(|_| {
        panic!(
            "Could not bind server to port '{}'",
            config.application.port
        )
    });

    tracing::info!("Server started on port {}", config.application.port);

    run(listener, conn_pool)?.await
}
