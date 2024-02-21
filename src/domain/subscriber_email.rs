use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<SubscriberEmail, String>{
        if validate_email(&email) {
            return Ok(Self(email));
        }
        
        Err("Invalid email".to_string())
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err_eq, assert_ok};

    use super::SubscriberEmail;

    #[test]
    fn subscriber_email_is_parsed_successfully() {
        let email = SubscriberEmail::parse("test@example.com".to_string());

        assert_ok!(email);
    }

    #[test]
    fn subscriber_email_is_parsing_failed() {
        let email = SubscriberEmail::parse("not an email at all".to_string());

        assert_err_eq!(email, "Invalid email".to_string());
    }
}