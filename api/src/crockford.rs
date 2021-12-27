use base32::Alphabet;

/// Generate a random 24 character Crockford Base 32 encoded ID.
pub fn random_id() -> String {
    let bytes: [u8; 15] = rand::random();
    base32::encode(Alphabet::Crockford, &bytes).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_id_len() {
        // Act.
        let id = random_id();

        // Assert.
        assert_eq!(id.len(), 24);
    }

    #[test]
    fn test_random_id_unique() {
        // Act.
        let (a, b) = (random_id(), random_id());

        // Assert.
        assert_ne!(a, b);
    }
}
