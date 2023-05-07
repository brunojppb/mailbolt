use once_cell::sync::Lazy;
use std::net::TcpListener;

use mailbolt::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};

// Spawn our web server in the background so we can execute
// the web server and our tests concurrently.
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked, the code in `TRACING` is executed.
    // All other invocations will instead skip execution. It memoises this call.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();

    let mut config = get_configuration().expect("Could not read configuration file");
    // Generate a random DB name for this test case
    // So we can run every test case in an isolated DB instance1
    config.database.database_name = uuid::Uuid::new_v4().to_string();
    let conn_pool = configure_db(&config.database).await;

    let sender_email = config.email_client.sender().expect("invalid sender email");
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.auth_token,
    );

    let server = mailbolt::startup::run(listener, conn_pool.clone(), email_client)
        .expect("failed to bind address");

    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);

    TestApp {
        address,
        db_pool: conn_pool,
    }
}

// Create a new database whenever we spawn a new app
// So tests can be executed in isolation
pub async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut conn = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Could not connect to DB");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Could not create database for tests.");

    // Given a connection pool that can be passed around
    // between requests, we can run async queries using
    // a minimal amount of connections.
    let conn_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Could not connect to database.");

    sqlx::migrate!("./migrations")
        .run(&conn_pool)
        .await
        .expect("Could not migrate database");

    conn_pool
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".into();
    let subscriber_name = "test".into();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
