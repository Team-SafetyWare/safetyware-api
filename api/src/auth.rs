use crate::repo::user_account::Creds;
use data_encoding::HEXLOWER_PERMISSIVE;
use ring::digest::SHA512_OUTPUT_LEN;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

const N_ITER: u32 = 100_000;
const CREDENTIAL_LEN: usize = SHA512_OUTPUT_LEN;

fn create_creds(password: String) -> Creds {
    let rng = SystemRandom::new();
    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt).unwrap();
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(N_ITER).unwrap(),
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
    let password_hash = HEXLOWER_PERMISSIVE
        .decode(creds.password_hash.as_bytes())
        .map_err(|_| ())?;
    let salt = HEXLOWER_PERMISSIVE
        .decode(creds.salt.as_bytes())
        .map_err(|_| ())?;
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(N_ITER).unwrap(),
        &salt,
        password.as_bytes(),
        &password_hash,
    )
    .map_err(|_| ())
}
