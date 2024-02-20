pub struct SubscriberName(String);

impl SubscriberName {
    pub(crate) fn parse(name: &str) -> Result<SubscriberName, String> {
        
        let is_not_empty = name.trim().is_empty();
        let is_too_long = name.len() > 256;
        let special_char = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_special_char = name.chars().any(|c|special_char.contains(&c));

        if is_not_empty || is_too_long || has_special_char {
            return Err(format!("Invalid name: {}", name));
        }

        Ok(SubscriberName(name.to_string()))
    }
}

impl AsRef<str> for SubscriberName{
    fn as_ref(&self) -> &str {
        &self.0
    }
}