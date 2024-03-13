mod helper;

use crate::helper::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_200_ok_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=jk&email=newsletter-api%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let response = app
        .post_subscription(body.into())
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_passed_with_invalid_form_data() {
    let app = spawn_app().await;
    let invalid_data = vec![
        ("name=jk", "missing email address"),
        ("email=newsletter-api%40gmail.com", "missing name"),
        ("", "missing name and email address"),
    ];

    // Act
    for (body, error_message) in invalid_data {
        let response = app
            .post_subscription(body.into())
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API request did not failed with error message: {}",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_persist_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=jk&email=newsletter-api%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let _ = app
        .post_subscription(body.into())
        .await
        .expect("Failed to execute request");

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    // Assert
    assert_eq!(saved.name, "jk");
    assert_eq!(saved.email, "newsletter-api@gmail.com");
    assert_eq!(saved.status, "pending-confirmation")
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_request() {
    let app = spawn_app().await;
    let body = "name=jk&email=newsletter-api%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app
        .post_subscription(body.into())
        .await
        .expect("Failed to execute request");

    // Assert
    // Mock asserts sending of email on Drop
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_link() {
    let app = spawn_app().await;
    let body = "name=jk&email=newsletter-api%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app
        .post_subscription(body.into())
        .await
        .expect("Failed to execute request");

    // Assert
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let email_body: serde_json::Value = serde_json::from_slice(&email_request.body)
        .unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);

        links[0].as_str().to_owned()
    };

    let html_link = get_link(&email_body["HtmlContent"].as_str().unwrap());
    let text_link = get_link(&email_body["TextContent"].as_str().unwrap());

    assert_eq!(html_link, text_link)
}
