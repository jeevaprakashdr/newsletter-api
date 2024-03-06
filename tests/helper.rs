use std::{io::sink, net::TcpListener};

use newsletter_api::email_client::EmailClient;
use newsletter_api::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to server");
    let port = listener.local_addr().unwrap().port();
    println!("port {port}");

    let mut settings = get_configuration().expect("Failed to get settings");
    settings.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&settings.database).await;
    let sender_email = settings
        .email_client_settings
        .sender_email()
        .expect("failed to parse the email id");
    let email_client = EmailClient::new(
        settings.email_client_settings.base_url,
        sender_email,
        settings.email_client_settings.auth_token,
    );
    let server =
        run(listener, connection_pool.clone(), email_client).expect("Failed to spin the sever");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

async fn configure_database(database: &DatabaseSettings) -> sqlx::Pool<sqlx::Postgres> {
    let mut connection =
        PgConnection::connect(&database.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to database");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database.database_name).as_str())
        .await
        .expect(&format!(
            "Failed to create database: {}",
            &database.database_name
        ));

    let connection_pool = PgPool::connect(&database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrate");

    connection_pool
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
