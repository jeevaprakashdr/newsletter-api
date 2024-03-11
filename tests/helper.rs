use newsletter_api::configuration::DatabaseSettings;
use newsletter_api::startup::{get_connection_pool, Application};
use newsletter_api::{
    configuration::get_configuration,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::io::sink;
use uuid::Uuid;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let settings = {
        let mut s = get_configuration().expect("Failed to get settings");
        s.database.database_name = Uuid::new_v4().to_string();
        s.application_port = 0;
        s
    };

    configure_database(&settings.database).await;

    let application = Application::build(settings.clone())
        .await
        .expect("Failed to spin the server");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stoped());
    TestApp {
        address,
        db_pool: get_connection_pool(&settings),
        email_server: MockServer::start().await,
    }
}

async fn configure_database(database: &DatabaseSettings) {
    let mut connection = PgConnection::connect_with(&database.without_db())
        .await
        .expect("Failed to connect to database");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database.database_name).as_str())
        .await
        .expect(&format!(
            "Failed to create database: {}",
            &database.database_name
        ));

    let connection_pool = PgPool::connect_with(database.with_db())
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrate");
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
