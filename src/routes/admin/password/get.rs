use actix_web::{http::header::ContentType, Error, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, Error> {
    let mut msg_html = String::new();
    for msg in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", msg.content()).unwrap();
    }

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        format!(r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Change password</title>
    </head>
    <body>
        {msg_html}
        <form action="/admin/dashboard" method="post">
            <label>
                Current password
                <input type="password" placeholder="Enter your current password" name="current_password">
            </label>
            <br>
            <label>
                New password
                <input type="password" placeholder="Enter new password" name="new_password">
            </label>
            <br>
            <label>
                Confirm new password
                <input type="password" placeholder="Enter new password again" name="new_password_check">
            </label>
            <br>
            <button type="submit">Change password</button>
        </form>
        <p><a href="/admin/dashboard">&lt;- Back</a></p>
    </body>
</html>
    "#),
    ))
}
