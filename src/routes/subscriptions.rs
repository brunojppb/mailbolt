use std::{fmt::Debug, write, writeln};

use actix_web::{web, HttpResponse, ResponseError};
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();

    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }

    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error has occurred while storing the subscription token"
        )
    }
}

impl Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for StoreTokenError {}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[
    tracing::instrument(
        name = "Adding a new subscriber",
        skip(form, conn_pool, email_client, base_url),
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
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, actix_web::Error> {
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };

    let mut transaction = match conn_pool.begin().await {
        Ok(t) => t,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => {
            let token = generate_subscription_token();
            store_token(&mut transaction, subscriber_id, &token).await?;

            if transaction.commit().await.is_err() {
                return Ok(HttpResponse::InternalServerError().finish());
            }

            match send_confirmation_email(&email_client, new_subscriber, &base_url.0, &token).await
            {
                Ok(_) => Ok(HttpResponse::Ok().finish()),
                Err(_) => Ok(HttpResponse::InternalServerError().finish()),
            }
        }
        Err(e) => {
            // Using "{:?}" so we get the output of the Debug trait,
            // which gives us a better message in this case, including the query.
            tracing::error!("Could not save subscriber: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(client, new_subscriber, base_url)
)]
pub async fn send_confirmation_email(
    client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let html_body = &format!(
        "Welcome to Mailbolt!<br/>\
                Click <a href=\"{}\">here</a> to confirm your sub.",
        confirmation_link
    );
    let plain_body = &format!(
        "Welcome to Mailbolt!\nVisit {} to confirm your sub",
        confirmation_link
    );
    client
        .send_email(new_subscriber.email, "Welcome!", html_body, plain_body)
        .await
}

#[tracing::instrument(
    name = "Inserting new sub details to DB",
    skip(transaction, subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, $5)
    "#,
        subscriber_id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        Utc::now(),
        "pending_confirmation"
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Could not save subscriber: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Store subscription token",
    skip(transaction, subscriber_id, subscription_token)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id,
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
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
