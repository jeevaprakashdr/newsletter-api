use std::net::TcpListener;

use newsletter_api::{
    configuration::{self, get_configuration},
    startup::run,
};
use sqlx::{Connection, PgConnection, PgPool};

#[tokio::test]
async fn health_check_succeed() {
    // Arrange
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", app_address))
        .send()
        .await
        .expect("Health check failed");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscribe_returns_200_ok_for_valid_form_data() {
    let app_address = spawn_app().await;
    let configuration = configuration::get_configuration().expect("Failed to load configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to database");

    let client = reqwest::Client::new();
    let body = "name=jk&email=newsletter-api%40gmail.com";

    // Act
    let response = client
        .get(format!("{}/subscriptions", app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(saved.name, "jk");
    assert_eq!(saved.email, "newsletter-api@gmail.com")
}

#[tokio::test]
async fn subscribe_returns_400_when_passed_with_invalid_form_data() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let invalid_data = vec![
        ("name=jk", "missing email address"),
        ("email=newsletter-api%40gmail.com", "missing name"),
        ("", "missing name and email address"),
    ];

    // Act
    for (body, error_message) in invalid_data {
        let response = client
            .get(format!("{}/subscriptions", app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
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

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to server");
    let port = listener.local_addr().unwrap().port();
    println!("port {port}");
    let settings = get_configuration().expect("Failed to get settings");
    let connection = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Failed to connect to database");

    let server = run(listener, connection).expect("Failed to spin the sever");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
