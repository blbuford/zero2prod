use crate::domain::SubscriptionToken;
use crate::utils::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use std::fmt::Formatter;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber"
    skip(parameters, pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SubscriptionConfirmationError> {
    let subscription_token = SubscriptionToken::parse(parameters.subscription_token.to_string())
        .map_err(SubscriptionConfirmationError::ValidationError)?;

    let id = get_subscriber_id_from_token(&pool, &subscription_token)
        .await
        .context("Failed to retrieve subscriber ID from subscription_tokens.")?
        .ok_or_else(|| {
            SubscriptionConfirmationError::UnauthorizedError(
                "Failed to find token in database.".into(),
            )
        })?;

    confirm_subscriber(&pool, id).await.context("")?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed"
    skip(pool, subscriber_id)
)]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token"
    skip(pool, subscription_token)
)]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &SubscriptionToken,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token.as_ref()
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|r| r.subscriber_id))
}

#[derive(thiserror::Error)]
pub enum SubscriptionConfirmationError {
    #[error("{0}")]
    ValidationError(String),

    #[error("{0}")]
    UnauthorizedError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscriptionConfirmationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscriptionConfirmationError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscriptionConfirmationError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscriptionConfirmationError::UnauthorizedError(_) => StatusCode::UNAUTHORIZED,
            SubscriptionConfirmationError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
