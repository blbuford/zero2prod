use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct SubscriptionToken(String);

impl SubscriptionToken {
    pub fn parse(s: String) -> Result<SubscriptionToken, String> {
        let is_25_characters = s.len() == 25;
        let contains_only_alphanumerics = s.chars().all(|x| x.is_alphanumeric());

        if is_25_characters && contains_only_alphanumerics {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscription token!", s))
        }
    }

    pub fn generate() -> SubscriptionToken {
        let mut rng = thread_rng();
        Self(
            std::iter::repeat_with(|| rng.sample(Alphanumeric))
                .map(char::from)
                .take(25)
                .collect(),
        )
    }
}
impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriptionToken;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_25_char_long_token_is_valid() {
        let token = "a".repeat(25);
        assert_ok!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_token_shorter_than_25_char_is_rejected() {
        let token = "a".repeat(24);
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_token_longer_than_25_char_is_rejected() {
        let token = "a".repeat(26);
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_token_that_contains_a_non_alphanumeric_is_rejected() {
        let token = "aZaZaZaZaZaZaZaZaZaZaZaZ/";
        assert_err!(SubscriptionToken::parse(token.into()));
    }
}
