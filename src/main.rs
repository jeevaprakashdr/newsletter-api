use newsletter_api::startup::Application;
use newsletter_api::telemetry::init_tracing_subscriber;
use newsletter_api::{configuration, telemetry::get_tracing_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_tracing_subscriber(
        "newsletter".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_tracing_subscriber(subscriber);

    let settings = configuration::get_configuration().expect("Failed to read the configuration");
    let application = Application::build(settings).await?;
    application.run_until_stoped().await?;
    Ok(())
}
