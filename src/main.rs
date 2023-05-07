use mailbolt::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

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
        .connect_lazy_with(config.database.with_db());

    let sender_email = config.email_client.sender().expect("Invalid sender email");
    let email_client = EmailClient::new(config.email_client.base_url, sender_email);

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

    run(listener, conn_pool, email_client)?.await
}
