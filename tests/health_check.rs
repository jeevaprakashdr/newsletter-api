#[tokio::test]
async fn health_check_succeed() {
    // Arrange
    spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://localhost:8080/health_check")
        .send()
        .await
        .expect("Health check failed");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

fn spawn_app() {
    let server = newsletter_api::run().expect("Failed to spin the sever");

    let _ = tokio::spawn(server);
}
