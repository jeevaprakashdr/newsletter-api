use std::net::TcpListener;

use env_logger::Env;
use newsletter_api::configuration;
use newsletter_api::startup::run;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = configuration::get_configuration().expect("Failed to read the configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(address)?;
    let connection = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("failed to connect to database");

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    run(listener, connection)?.await
}
