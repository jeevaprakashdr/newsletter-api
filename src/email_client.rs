use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;
use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    auth_token: Secret<String>
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, auth_token: Secret<String>) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            auth_token,
        }
    }

    pub async fn send_email(&self, recipient: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let send_email_request = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_content: html_content.to_owned(),
            text_content: text_content.to_owned(),
        };

        self.http_client
            .post(&url)
            .json(&send_email_request)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .send().await?;

        Ok(())
    }
}

#[derive(Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_content: String,
    text_content: String,
}

#[cfg(test)]
mod email_client_tests {
    use fake::{Fake, Faker};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use secrecy::Secret;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{ header, header_exists, method, path};
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn should_send_email() {
        // Arrange
        let server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).expect("failed to get the fake sender email");
        let email_client = EmailClient::new(server.uri(), sender, Secret::new(Faker.fake()));

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).expect("failed to get the fake sender email");
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..9).fake();

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        // Act
        let _ = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        // Assert
    }
}