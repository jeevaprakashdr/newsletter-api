use unicode_segmentation::UnicodeSegmentation;
#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub(crate) fn parse(name: String) -> Result<SubscriberName, String> {
        let is_not_empty = name.trim().is_empty();
        let is_too_long = name.graphemes(true).count() > 256;
        let special_char = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_special_char = name.chars().any(|c| special_char.contains(&c));

        if is_not_empty || is_too_long || has_special_char {
            return Err(format!("Invalid name: {}", name));
        }

        Ok(SubscriberName(name.to_string()))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;

    use claims::assert_err;
    use claims::assert_ok;

    #[test]
    fn a_grapheme_name_is_valid() {
        let name = "ë".repeat(255);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_long_name_more_than_256_char_is_invalid() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_containing_invalid_char_is_invalid() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parse() {
        let name = "a valid name".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
