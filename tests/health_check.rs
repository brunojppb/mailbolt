#[tokio::test]
async fn health_check() {
    spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to get /health_check endpoint");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Spawn our web server in the background so we can execute
// the web server and our tests concurrently.
fn spawn_app() {
    let server = mailbolt::run().expect("failed to bind address");
    let _ = tokio::spawn(server);
}
