mod helper;
use crate::helper::spawn_app;

#[tokio::test]
async fn health_check_succeed() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Health check failed");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}
