use wiremock::{matchers::method, Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    mock_email_server_call().mount(&app.email_server).await;

    let req_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let resp = app.post_subscriptions(req_body.into()).await;

    assert_eq!(200, resp.status().as_u16());
}
#[tokio::test]
async fn subscribe_persists_a_new_subscriber() {
    let app = spawn_app().await;
    mock_email_server_call().mount(&app.email_server).await;

    let req_body = "name=hiju%20guin&email=hiju_guin%40gmail.com";

    let resp = app.post_subscriptions(req_body.into()).await;

    assert_eq!(200, resp.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Could not fetch subscriptions from db");

    assert_eq!(saved.email, "hiju_guin@gmail.com");
    assert_eq!(saved.name, "hiju guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=Bruno%20p", "a missing email"),
        ("email=bruno%40example.com", "a missing name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_msg) in test_cases {
        let resp = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            resp.status().as_u16(),
            "API call did not fail with a Bad Request when the payload was {}",
            error_msg
        )
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=julios%20james&email=julius%40email.com";

    mock_email_server_call().mount(&app.email_server).await;

    Mock::given(wiremock::matchers::path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_req = &app.email_server.received_requests().await.unwrap()[0];

    let email_body: serde_json::Value = serde_json::from_slice(&email_req.body).unwrap();
    // Extract email links from body
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(email_body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(email_body["TextBody"].as_str().unwrap());

    assert_eq!(html_link, text_link);

    // Wiremock will assert API calls on drop
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=&email=james%40email.com", "empty name"),
        ("name=James&email=", "empty email"),
        ("name=James&email=not-a-valid-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let resp = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            resp.status().as_u16(),
            "The API did not return a 200 status when the payload was {}",
            description
        )
    }
}

fn mock_email_server_call() -> Mock {
    Mock::given(wiremock::matchers::path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
}
