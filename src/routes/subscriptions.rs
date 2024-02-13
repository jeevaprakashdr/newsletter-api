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

#[tracing::instrument(
    name = "Adding new subscriber",
    skip(form_data, connection),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form_data.email,
        subscriber_name = %form_data.name
    )
)]
pub async fn subscribe(
    form_data: web::Form<FormData>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    match create_subscriber(&form_data, &connection).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn create_subscriber(
    form_data: &FormData,
    connection_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at) VALUES($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form_data.email,
        form_data.name,
        Utc::now()
    )
    .execute(connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
