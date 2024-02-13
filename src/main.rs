use std::net::TcpListener;

use newsletter_api::configuration;
use newsletter_api::startup::run;
use sqlx::PgPool;
use tracing::dispatcher::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    LogTracer::init().expect("Failed to initialize logger");
    
    let settings = configuration::get_configuration().expect("Failed to read the configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(address)?;
    let connection = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("failed to connect to database");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let format_layer = BunyanFormattingLayer::new("newsletter".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(format_layer)
        .into();

    set_global_default(subscriber).expect("Failed to set tracing subscriber");

    run(listener, connection)?.await
}
