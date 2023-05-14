use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

/// using the web::Query<T> extractor, query parameters that are not optional
/// are automatically populated in the struct T and any request that does not
/// provide these parameters are faced with a 400 response automatically.
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(parameters: web::Query<Parameters>) -> HttpResponse {
    tracing::info!("Token confirming: {}", parameters.subscription_token);
    HttpResponse::Ok().finish()
}
