use std::{fmt, ops};

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::InternalError,
    Error, FromRequest, HttpMessage,
};
use actix_web_lab::middleware::Next;
use anyhow::anyhow;
use uuid::Uuid;

use crate::{session_state::TypedSession, utils};

#[derive(Copy, Clone, Debug)]
pub struct UserId(Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ops::Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn reject_anonymous_users(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let session = {
        let (http_request, payload) = req.parts_mut();
        TypedSession::from_request(http_request, payload).await
    }?;

    match session.get_user_id().map_err(utils::e500)? {
        Some(user_id) => {
            req.extensions_mut().insert(UserId(user_id));
            next.call(req).await
        }
        None => {
            let response = utils::see_other("/login");
            let e = anyhow!("The user has not logged in");
            Err(InternalError::from_response(e, response).into())
        }
    }
}
