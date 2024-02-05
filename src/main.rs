use std::net::TcpListener;

use newsletter_api::configuration;
use newsletter_api::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = configuration::get_configuration().expect("Failed to read the configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}