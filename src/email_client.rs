use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    auth_token: Secret<String>,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, auth_token: Secret<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            http_client: client,
            base_url,
            sender,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let send_email_request = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject.as_ref(),
            html_content: html_content.as_ref(),
            text_content: text_content.as_ref(),
        };

        self.http_client
            .post(&url)
            .json(&send_email_request)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_content: &'a str,
    text_content: &'a str,
}

#[cfg(test)]
mod email_client_tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claims::{assert_err, assert_err_eq, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailRequesstMatcher;

    impl Match for SendEmailRequesstMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlContent").is_some()
                    && body.get("TextContent").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn should_send_email() {
        // Arrange
        let server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake())
            .expect("failed to get the fake sender email");
        let email_client = EmailClient::new(server.uri(), sender, Secret::new(Faker.fake()));

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake())
            .expect("failed to get the fake sender email");
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..9).fake();

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequesstMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        // Act
        let response = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_ok!(response);
    }

    #[tokio::test]
    async fn should_return_http_status_code_500() {
        // Arrange
        let server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake())
            .expect("failed to get the fake sender email");
        let email_client = EmailClient::new(server.uri(), sender, Secret::new(Faker.fake()));

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake())
            .expect("failed to get the fake sender email");
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..9).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        // Act
        let response = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(response);
    }

    #[tokio::test]
    async fn should_time_out_if_server_takes_long_time_to_respond() {
        // Arrange
        let server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake())
            .expect("failed to get the fake sender email");
        let email_client = EmailClient::new(server.uri(), sender, Secret::new(Faker.fake()));

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake())
            .expect("failed to get the fake sender email");
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..9).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(20)))
            .expect(1)
            .mount(&server)
            .await;

        // Act
        let response = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(response);
    }
}
