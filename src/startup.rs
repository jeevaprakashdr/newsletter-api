use crate::configuration::Settings;
use crate::email_client::EmailClient;
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io::Error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
) -> Result<Server, Error> {
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(crate::routes::health_check))
            .route("/subscriptions", web::post().to(crate::routes::subscribe))
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn get_connection_pool(settings: &Settings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(settings.database.with_db())
}

impl Application {
    pub async fn build(settings: Settings) -> Result<Self, Error> {
        let address = format!(
            "{}:{}",
            settings.application_host_address, settings.application_port
        );

        let listener = TcpListener::bind(address)?;
        let connection_pool = get_connection_pool(&settings);
        let sender_email = settings
            .email_client_settings
            .sender_email()
            .expect("Invalid subscription email sender address");
        let email_client = EmailClient::new(
            settings.email_client_settings.base_url,
            sender_email,
            settings.email_client_settings.auth_token,
        );

        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stoped(self) -> Result<(), Error> {
        self.server.await
    }
}
