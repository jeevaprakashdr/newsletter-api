use once_cell::sync::Lazy;
use std::{
    io::{sink, Sink, Stdout},
    net::TcpListener,
};

use newsletter_api::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

#[tokio::test]
async fn health_check_succeed() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Health check failed");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscribe_returns_200_ok_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=jk&email=newsletter-api%40gmail.com";

    // Act
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(saved.name, "jk");
    assert_eq!(saved.email, "newsletter-api@gmail.com")
}

#[tokio::test]
async fn subscribe_returns_400_when_passed_with_invalid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let invalid_data = vec![
        ("name=jk", "missing email address"),
        ("email=newsletter-api%40gmail.com", "missing name"),
        ("", "missing name and email address"),
    ];

    // Act
    for (body, error_message) in invalid_data {
        let response = client
            .post(format!("{}/subscriptions", app.address))
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

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_tracing_subscriber(subscriber_name, default_filter, std::io::stdout);
        init_tracing_subscriber(subscriber);
    } else {
        let subscriber = get_tracing_subscriber(subscriber_name, default_filter, sink);
        init_tracing_subscriber(subscriber);
    }
});

struct TestApp {
    address: String,
    db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to server");
    let port = listener.local_addr().unwrap().port();
    println!("port {port}");

    let mut settings = get_configuration().expect("Failed to get settings");
    settings.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&settings.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to spin the sever");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

async fn configure_database(database: &DatabaseSettings) -> sqlx::Pool<sqlx::Postgres> {
    let mut connection = PgConnection::connect(&database.connection_string_without_db())
        .await
        .expect("Failed to connect to database");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database.database_name).as_str())
        .await
        .expect(&format!(
            "Failed to create database: {}",
            &database.database_name
        ));

    let connection_pool = PgPool::connect(&database.connection_string())
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrate");

    connection_pool
}
