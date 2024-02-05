use actix_web::HttpResponse;
use actix_web::web;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form_data: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}