use std::net::TcpListener;

#[tokio::test]
async fn health_check() {
    let base_url = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &base_url))
        .send()
        .await
        .expect("Failed to get /health_check endpoint");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Spawn our web server in the background so we can execute
// the web server and our tests concurrently.
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let server = mailbolt::run(listener).expect("failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
