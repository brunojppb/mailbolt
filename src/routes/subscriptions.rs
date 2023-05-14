use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[
    tracing::instrument(
        name = "Adding a new subscriber",
        skip(form, conn_pool, email_client),
        fields(
            subscriber_email = %form.email,
            subscriber_name = %form.name
        )
    )
]
pub async fn subscribe(
    form: web::Form<FormData>,
    conn_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&conn_pool, &new_subscriber).await {
        Ok(_) => {
            tracing::info!("New subscriber has been saved");
            let confirmation_link = "https://invalid.domain.com/subscriptions/confirm";
            match email_client
                .send_email(
                    new_subscriber.email,
                    "Welcome!",
                    &format!(
                        "Welcome to Mailbolt!<br/>\
                        Click <a href=\"{}\">here</a> to confirm your sub.",
                        confirmation_link
                    ),
                    &format!(
                        "Welcome to Mailbolt!\nVisit {} to confirm your sub",
                        confirmation_link
                    ),
                )
                .await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(e) => {
            // Using "{:?}" so we get the output of the Debug trait,
            // which gives us a better message in this case, including the query.
            tracing::error!("Could not save subscriber: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "Inserting new sub details to DB", skip(pool, subscriber))]
pub async fn insert_subscriber(
    pool: &PgPool,
    subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, $5)
    "#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        Utc::now(),
        "confirmed"
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Could not save subscriber: {:?}", e);
        e
    })?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}
