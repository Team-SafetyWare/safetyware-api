use base32::Alphabet;

/// Generate a random 24 character Crockford Base 32 encoded ID.
pub fn random_id() -> String {
    let bytes: [u8; 15] = rand::random();
    base32::encode(Alphabet::Crockford, &bytes).to_lowercase()
}
