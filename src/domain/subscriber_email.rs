use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<SubscriberEmail, String> {
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
    use super::SubscriberEmail;
    use claims::{assert_err_eq, assert_ok};
    use fake::{faker::internet::en::SafeEmail, Fake};

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn subscriber_email_is_parsed_successfully(valid_email: ValidEmailFixture) {
        dbg!(&valid_email);
        assert_ok!(SubscriberEmail::parse(valid_email.0));
    }

    #[quickcheck_macros::quickcheck]
    fn subscriber_email_is_parsing_failed(email: String) {
        assert_err_eq!(SubscriberEmail::parse(email), "Invalid email".to_string());
    }
}
