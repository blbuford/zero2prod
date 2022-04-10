use secrecy::{ExposeSecret, Secret};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct AdminPassword(Secret<String>);

impl AdminPassword {
    pub fn parse(s: Secret<String>) -> Result<Self, String> {
        let is_too_short = s.expose_secret().graphemes(true).count() <= 12;

        if is_too_short {
            Err("Passwords must be longer than 12 characters.".into())
        } else {
            Ok(Self(s))
        }
    }

    pub fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::AdminPassword;
    use claim::{assert_err, assert_ok};
    use secrecy::Secret;

    #[test]
    fn a_13_grapheme_long_password_is_valid() {
        let password = Secret::new("g̈".repeat(13));
        assert_ok!(AdminPassword::parse(password));
    }

    #[test]
    fn a_13_character_long_password_is_valid() {
        let password = Secret::new("a".repeat(13));
        assert_ok!(AdminPassword::parse(password));
    }

    #[test]
    fn a_12_grapheme_long_password_is_invalid() {
        let password = Secret::new("g̈".repeat(12));
        assert_err!(AdminPassword::parse(password));
    }

    #[test]
    fn a_12_character_long_password_is_invalid() {
        let password = Secret::new("a".repeat(12));
        assert_err!(AdminPassword::parse(password));
    }
}
