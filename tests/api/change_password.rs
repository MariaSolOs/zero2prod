use uuid::Uuid;

use crate::helpers;

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    let app = helpers::spawn_app().await;

    let response = app.get_change_password().await;

    helpers::assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    let app = helpers::spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": new_password,
            "new_password_check": new_password
        }))
        .await;

    helpers::assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    let app = helpers::spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": new_password,
            "new_password_check": another_new_password
        }))
        .await;
    helpers::assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page
        .contains("<p><i>You entered 2 different new passwords, the values should match.</i></p>"));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    let app = helpers::spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": wrong_password,
            "new_password": new_password,
            "new_password_check": new_password
        }))
        .await;
    helpers::assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn changing_password_works() {
    let app = helpers::spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password
        }))
        .await;
    helpers::assert_is_redirect_to(&response, "/admin/dashboard");

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": new_password,
            "new_password_check": new_password
        }))
        .await;
    helpers::assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Your password has been changed.</i></p>"));

    let response = app.post_logout().await;
    helpers::assert_is_redirect_to(&response, "/login");

    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>You have successfully logged out.</i></p>"));

    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": new_password
        }))
        .await;
    helpers::assert_is_redirect_to(&response, "/admin/dashboard");
}
