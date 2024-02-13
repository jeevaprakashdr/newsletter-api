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
    let request_id = Uuid::new_v4();
    tracing::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        request_id,
        form_data.email,
        form_data.name
    );

    tracing::info!(
        "request_id {} - Saving new subscriber details in the database",
        request_id
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
            tracing::info!("New subscriber details are saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
