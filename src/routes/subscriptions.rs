use actix_web::web;
use actix_web::HttpResponse;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    form_data: web::Form<FormData>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    log::info!(
        "Saving new subscriber details {}, {}",
        form_data.name,
        form_data.email
    );

    match sqlx::query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at) VALUES($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form_data.email,
        form_data.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await
    {
        Ok(_) => {
            log::info!("New subscriber details are saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
