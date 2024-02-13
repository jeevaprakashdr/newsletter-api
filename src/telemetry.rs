use tracing::{dispatcher::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_tracing_subscriber(app_name: String, log_level: String) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    let format_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(format_layer)    
}

pub fn init_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to initialize logger");
    set_global_default(subscriber.into()).expect("Failed to set tracing subscriber");
}