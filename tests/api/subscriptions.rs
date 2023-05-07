use crate::helpers::spawn_app;

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
