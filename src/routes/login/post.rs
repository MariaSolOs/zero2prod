use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web::{Data, Form},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use anyhow::Error;
use secrecy::Secret;
use sqlx::PgPool;
use std::fmt;
use tracing::{field, Span};

use crate::{
    authentication::{self, AuthError, Credentials},
    routes,
    session_state::TypedSession,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed.")]
    AuthError(#[source] Error),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] Error),
}

impl fmt::Debug for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        routes::error_chain_fmt(self, f)
    }
}

#[tracing::instrument(
    skip(form, pool, session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    form: Form<FormData>,
    pool: Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    Span::current().record("username", field::display(&credentials.username));

    match authentication::validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            Span::current().record("user_id", field::display(&user_id));

            session.renew();

            session
                .insert_user_id(user_id)
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;

            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/admin/dashboard"))
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            Err(login_redirect(e))
        }
    }
}

fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();

    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();

    InternalError::from_response(e, response)
}
