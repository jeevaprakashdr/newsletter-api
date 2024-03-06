use std::net::TcpListener;

use newsletter_api::email_client::EmailClient;
use newsletter_api::startup::run;
use newsletter_api::telemetry::init_tracing_subscriber;
use newsletter_api::{configuration, telemetry::get_tracing_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_tracing_subscriber(
        "newsletter".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_tracing_subscriber(subscriber);

    let settings = configuration::get_configuration().expect("Failed to read the configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(address)?;
    let connection = PgPool::connect(settings.database.connection_string().expose_secret())
        .await
        .expect("failed to connect to database");

    let sender_email = settings
        .email_client_settings
        .sender_email()
        .expect("Invalid subscription email sender address");
    let email_client = EmailClient::new(
        settings.email_client_settings.base_url,
        sender_email,
        settings.email_client_settings.auth_token,
    );
    run(listener, connection, email_client)?.await
}
