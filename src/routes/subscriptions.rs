use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[
    tracing::instrument(
        name = "Adding a new subscriber",
        skip(form, conn_pool),
        fields(
            subscriber_email = %form.email,
            subscriber_name = %form.name
        )
    )
]
pub async fn subscribe(form: web::Form<FormData>, conn_pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(&conn_pool, &form).await {
        Ok(_) => {
            tracing::info!("New subscriber has been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            // Using "{:?}" so we get the output of the Debug trait,
            // which gives us a better message in this case, including the query.
            tracing::error!("Could not save subscriber: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "Inserting new sub details to DB", skip(pool, form))]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Could not save subscriber: {:?}", e);
        e
    })?;

    Ok(())
}
