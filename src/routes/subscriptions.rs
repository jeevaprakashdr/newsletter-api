use std::ops::Deref;

use actix_web::web;
use actix_web::HttpResponse;
use sqlx::PgConnection;
use uuid::Uuid;
use chrono::Utc;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    form_data: web::Form<FormData>,
    connection: web::Data<PgConnection>,
) -> HttpResponse {
    sqlx::query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at) VALUES($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form_data.email,
        form_data.name,
        Utc::now()
    ).execute(connection.get_ref().deref())
    .await;

    HttpResponse::Ok().finish()
}
