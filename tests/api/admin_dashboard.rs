use crate::helpers;

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    let app = helpers::spawn_app().await;

    let response = app.get_admin_dashboard().await;

    helpers::assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    let app = helpers::spawn_app().await;

    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password
        }))
        .await;
    helpers::assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

    let response = app.post_logout().await;
    helpers::assert_is_redirect_to(&response, "/login");

    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    let response = app.get_admin_dashboard().await;
    helpers::assert_is_redirect_to(&response, "/login");
}
