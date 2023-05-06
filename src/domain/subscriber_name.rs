use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

pub static FORBIDDEN_CHARS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let contains_forbidden_chars = s.chars().any(|c| FORBIDDEN_CHARS.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            Err(format!("{} is not a valid subscriber name", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use crate::domain::SubscriberName;

    use super::FORBIDDEN_CHARS;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ã".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "ã".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_with_invalid_chars_are_rejected() {
        for name in &FORBIDDEN_CHARS {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }
}
