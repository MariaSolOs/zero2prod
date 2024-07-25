use actix_web::{Error, HttpResponse};
use actix_web_flash_messages::FlashMessage;

use crate::{session_state::TypedSession, utils};

pub async fn log_out(session: TypedSession) -> Result<HttpResponse, Error> {
    session.log_out();
    FlashMessage::info("You have successfully logged out.").send();
    Ok(utils::see_other("/login"))
}
