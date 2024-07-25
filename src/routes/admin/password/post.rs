use actix_web::{
    web::{Data, Form, ReqData},
    Error, HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    authentication::{self, AuthError, Credentials, UserId},
    routes::admin::dashboard,
    utils,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: Form<FormData>,
    pool: Data<PgPool>,
    user_id: ReqData<UserId>,
) -> Result<HttpResponse, Error> {
    let user_id = user_id.into_inner();

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error("You entered 2 different new passwords, the values should match.")
            .send();
        return Ok(utils::see_other("/admin/password"));
    }

    let username = dashboard::get_username(*user_id, &pool)
        .await
        .map_err(utils::e500)?;

    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };

    if let Err(e) = authentication::validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(utils::see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(utils::e500(e).into()),
        };
    }

    authentication::change_password(*user_id, form.0.new_password, &pool)
        .await
        .map_err(utils::e500)?;

    FlashMessage::error("Your password has been changed.").send();

    Ok(utils::see_other("/admin/password"))
}
