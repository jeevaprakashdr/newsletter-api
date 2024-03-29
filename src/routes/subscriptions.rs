use actix_web::{web, HttpResponse};
use chrono::Utc;
use reqwest::Error;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::SubscriberEmail;
use crate::domain::{NewSubscriber, SubscriberName};
use crate::email_client::EmailClient;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
name = "Adding new subscriber",
skip(form_data, connection, email_client),
fields(
subscriber_email = % form_data.email,
subscriber_name = % form_data.name
)
)]
pub async fn subscribe(
    form_data: web::Form<FormData>,
    connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form_data.0.try_into() {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if create_subscriber(&new_subscriber, &connection)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, new_subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "sending confirmation email ",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
) -> Result<(), Error> {
    let confirmation_link = "https://newletter-api.com/confirmation/confirm";
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome",
            &format!(
                "Welcome to newsletter subscription! click on <a href= \"{}\">HERE </a>",
                confirmation_link
            ),
            &format!(
                "Welcome to newsletter subscription! click on <a href= \"{}\">HERE </a>",
                confirmation_link
            ),
        )
        .await
}

#[tracing::instrument(name = "create subscriber", skip(new_subscriber, connection_pool))]
async fn create_subscriber(
    new_subscriber: &NewSubscriber,
    connection_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at, status)
        VALUES($1, $2, $3, $4,'pending-confirmation')"#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { name, email })
    }
}
