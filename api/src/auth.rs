use crate::repo::user_account::Creds;
use data_encoding::HEXLOWER_PERMISSIVE;
use ring::digest::SHA512_OUTPUT_LEN;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

fn create_creds(password: String) -> Creds {
    const CREDENTIAL_LEN: usize = SHA512_OUTPUT_LEN;
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let rng = SystemRandom::new();
    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt).unwrap();
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &mut pbkdf2_hash,
    );
    Creds {
        password_hash: HEXLOWER_PERMISSIVE.encode(&pbkdf2_hash),
        salt: HEXLOWER_PERMISSIVE.encode(&salt),
    }
}

fn verify_password(password: String, creds: Creds) -> Result<(), ()> {
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let password_hash = HEXLOWER_PERMISSIVE
        .decode(creds.password_hash.as_bytes())
        .map_err(|_| ())?;
    let salt = HEXLOWER_PERMISSIVE
        .decode(creds.salt.as_bytes())
        .map_err(|_| ())?;
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &password_hash,
    )
    .map_err(|_| ())
}
