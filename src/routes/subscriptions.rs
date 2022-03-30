use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName, SubscriptionToken};
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use askama_actix::Template;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use std::fmt::Formatter;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[derive(Template)]
#[template(path = "confirmation.html")]
pub struct ConfirmationTemplate<'a> {
    confirmation_link: &'a str,
}

#[tracing::instrument(
    name = "Adding as a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = % form.email,
        subscriber_name = % form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let subscriber_id = match get_past_subscription(&mut transaction, &new_subscriber)
        .await
        .context("Failed to check if the subscriber already exists in database.")?
    {
        Some(id) => id,
        None => insert_subscriber(&mut transaction, &new_subscriber)
            .await
            .context("Failed to insert new subscriber in the database.")?,
    };
    let subscription_token = match get_past_subscription_token(&mut transaction, subscriber_id)
        .await
        .context("Failed to check for existing subscription token in database.")?
    {
        Some(token) => token,
        None => {
            let subscription_token = SubscriptionToken::generate();
            store_token(&mut transaction, subscriber_id, &subscription_token)
                .await
                .context("Failed to store subscription token in the database.")?;
            subscription_token
        }
    };

    transaction
        .commit()
        .await
        .context("Failed to commit the SQL query to the database.")?;
    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email.")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Storing subscription token in the database",
    skip(transaction, subscription_token)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &SubscriptionToken,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        subscription_token.as_ref(),
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| StoreTokenError(e))?;
    Ok(())
}
#[tracing::instrument(
    name = "Sending a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &ApplicationBaseUrl,
    subscription_token: &SubscriptionToken,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url.0,
        subscription_token.as_ref()
    );

    let template = ConfirmationTemplate {
        confirmation_link: confirmation_link.as_str(),
    };

    let rendered_html = template.render().unwrap();
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &rendered_html,
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription",
                confirmation_link
            ),
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        ON CONFLICT DO NOTHING
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Checking for past subscription in the database",
    skip(new_subscriber, transaction)
)]
pub async fn get_past_subscription(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT id FROM subscriptions WHERE email = $1
        "#,
        new_subscriber.email.as_ref(),
    )
    .fetch_optional(transaction)
    .await?;
    Ok(result.map(|r| r.id))
}

#[tracing::instrument(
    name = "Checking for past subscription token in the database",
    skip(subscriber_id, transaction)
)]
pub async fn get_past_subscription_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
) -> Result<Option<SubscriptionToken>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscription_token FROM subscription_tokens WHERE subscriber_id = $1
        "#,
        subscriber_id,
    )
    .fetch_optional(transaction)
    .await?;
    Ok(result.map(|r| SubscriptionToken::parse(r.subscription_token).unwrap()))
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token."
        )
    }
}
impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

pub fn error_chain_fmt(
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
