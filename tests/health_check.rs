use std::net::TcpListener;

#[tokio::test]
async fn health_check_succeed() {
    // Arrange
    let url = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", url))
        .send()
        .await
        .expect("Health check failed");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to server");
    let port = listener.local_addr().unwrap().port();

    println!("port {port}");
    let server = newsletter_api::run(listener).expect("Failed to spin the sever");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
