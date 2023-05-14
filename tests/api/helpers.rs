use once_cell::sync::Lazy;
use uuid::Uuid;

use mailbolt::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_db_conn_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use wiremock::MockServer;

// Spawn our web server in the background so we can execute
// the web server and our tests concurrently.
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked, the code in `TRACING` is executed.
    // All other invocations will instead skip execution. It memoises this call.
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let config = {
        let mut c = get_configuration().expect("Failed to read config");
        c.database.database_name = Uuid::new_v4().to_string();
        // zero signals to the OS to use a random, available port.
        c.application.port = 0;
        // Overwrite email server endpoint so we can intercept and mock responses
        c.email_client.base_url = email_server.uri();
        c
    };

    //Create and migrate the test DB
    configure_db(&config.database).await;

    let app = Application::build(config.clone())
        .await
        .expect("Could not build application");

    let app_port = app.port();
    let address = format!("http://127.0.0.1:{}", app_port);
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address,
        port: app_port,
        db_pool: get_db_conn_pool(&config.database),
        email_server,
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
    /// Address where our app will be listening to HTTP requests.
    /// Commonly using 127.0.0.1 during local tests.
    pub address: String,
    /// application port assigned during test app bootstrap
    pub port: u16,
    /// Postgres connection pool for tests to perform queries against.
    pub db_pool: PgPool,
    /// Intercept and mock email provider APIs
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}
