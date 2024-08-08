use actix_web::{error, http::header::LOCATION, Error, HttpResponse};
use std::fmt;

pub fn e400<T>(e: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    error::ErrorBadRequest(e)
}

pub fn e500<T>(e: T) -> Error
where
    T: fmt::Debug + fmt::Display + 'static,
{
    error::ErrorInternalServerError(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
