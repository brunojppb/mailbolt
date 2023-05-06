use once_cell::sync::Lazy;
use std::net::TcpListener;

use mailbolt::{
    configuration::{get_configuration, DatabaseSettings},
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};

#[tokio::test]
async fn health_check() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to get /health_check endpoint");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let req_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let resp = client
        .post(format!("{}/subscriptions", &test_app.address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(req_body)
        .send()
        .await
        .expect("Failed to send subscription request");

    assert_eq!(200, resp.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Could not fetch subscriptions from db");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Bruno%20p", "a missing email"),
        ("email=bruno%40example.com", "a missing name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_msg) in test_cases {
        let resp = client
            .post(format!("{}/subscriptions", &test_app.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send subscription request");

        assert_eq!(
            400,
            resp.status().as_u16(),
            "API call did not fail with a Bad Request when the payload was {}",
            error_msg
        )
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=james%40email.com", "empty name"),
        ("name=James&email=", "empty email"),
        ("name=James&email=not-a-valid-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 status when the payload was {}",
            description
        )
    }
}

// Spawn our web server in the background so we can execute
// the web server and our tests concurrently.
async fn spawn_app() -> TestApp {
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

    let server =
        mailbolt::startup::run(listener, conn_pool.clone()).expect("failed to bind address");

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
