use actix_web::{
    web::{Data, Form, ReqData},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use anyhow::{anyhow, Context};
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    email_client::EmailClient,
    idempotency::{self, IdempotencyKey, NextAction},
    utils,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, pool, email_client, user_id),
    fields(user_id=%*user_id)
)]
pub async fn publish_newsletter(
    form: Form<FormData>,
    user_id: ReqData<UserId>,
    pool: Data<PgPool>,
    email_client: Data<EmailClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(utils::e400)?;

    let transaction = match idempotency::try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(utils::e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(res) => {
            success_message().send();
            return Ok(res);
        }
    };

    let subscribers = get_confirmed_subscribers(&pool)
        .await
        .map_err(utils::e500)?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(&subscriber.email, &title, &html_content, &text_content)
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(utils::e500)?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    error.message = %error,
                    "Skipping a confirmed subscriber. Their stored contact details are invalid."
                );
            }
        }
    }

    success_message().send();

    let response = utils::see_other("/admin/newsletters");
    let response = idempotency::save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(utils::e500)?;

    Ok(response)
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers =
        sqlx::query!(r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|r| match SubscriberEmail::parse(r.email) {
                Ok(email) => Ok(ConfirmedSubscriber { email }),
                Err(error) => Err(anyhow!(error)),
            })
            .collect();

    Ok(confirmed_subscribers)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter issue has been published!")
}
