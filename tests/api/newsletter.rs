use reqwest::Client;
use serde_json::json;
use uuid::Uuid;
use wiremock::{matchers, Mock, ResponseTemplate};

use crate::helpers::{self, ConfirmationLinks, TestApp};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = helpers::spawn_app().await;

    create_unconfirmed_subscriber(&app).await;

    Mock::given(matchers::any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(200, response.status());
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = helpers::spawn_app().await;

    create_confirmed_subscriber(&app).await;

    Mock::given(matchers::path("/email"))
        .and(matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(200, response.status());
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    let app = helpers::spawn_app().await;

    let test_cases = vec![
        (
            json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }),
            "missing title",
        ),
        (json!({ "title": "Newsletter!" }), "missing content"),
    ];

    for (body, message) in test_cases {
        let response = app.post_newsletters(body).await;

        assert_eq!(
            400,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            message
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    let app = helpers::spawn_app().await;

    let response = Client::new()
        .post(format!("{}/newsletters", &app.address))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    let app = helpers::spawn_app().await;
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let response = Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    let app = helpers::spawn_app().await;
    let username = &app.test_user.username;
    let password = Uuid::new_v4().to_string();

    assert_ne!(app.test_user.password, password);

    let response = Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(matchers::path("/email"))
        .and(matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
