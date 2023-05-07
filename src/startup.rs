use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};

/// Starts a Actix Web server and start listening for requests
/// on the given listener configuration.
pub fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // Wrap the DB connection in web::Data which wraps this instance
    // in an Arc reference that can be cloned.
    let conn_pool = web::Data::new(pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(conn_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub async fn build(config: Settings) -> Result<Server, std::io::Error> {
    let conn_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());

    let sender_email = config.email_client.sender().expect("Invalid sender email");
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.auth_token,
    );

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

    run(listener, conn_pool, email_client)
}
