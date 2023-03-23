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

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let req_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let resp = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(req_body)
        .send()
        .await
        .expect("Failed to send subscription request");

    assert_eq!(200, resp.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Bruno%20p", "a missing email"),
        ("email=bruno%40example.com", "a missing name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_msg) in test_cases {
        let resp = client
            .post(format!("{}/subscriptions", &app_address))
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

// Spawn our web server in the background so we can execute
// the web server and our tests concurrently.
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let server = mailbolt::startup::run(listener).expect("failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
