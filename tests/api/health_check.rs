use reqwest::Client;

use crate::helpers;

#[tokio::test]
async fn health_check_works() {
    let app = helpers::spawn_app().await;
    let client = Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
