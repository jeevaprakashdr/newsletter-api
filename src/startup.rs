use std::net::TcpListener;

use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use sqlx::PgPool;

pub fn run(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(crate::routes::health_check))
            .route("/subscriptions", web::post().to(crate::routes::subscribe))
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
