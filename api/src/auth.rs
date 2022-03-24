use crate::repo::user_account::{ArcUserAccountRepo, Creds};
use data_encoding::HEXLOWER_PERMISSIVE;
use ring::digest::SHA512_OUTPUT_LEN;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

const N_ITER: u32 = 100_000;
const CREDENTIAL_LEN: usize = SHA512_OUTPUT_LEN;

#[derive(Clone)]
pub struct AuthProvider {
    pub user_account_repo: ArcUserAccountRepo,
}

impl AuthProvider {
    pub async fn set_password(&self, user_account_id: &str, password: &str) -> anyhow::Result<()> {
        let creds = create_creds(password);
        self.user_account_repo
            .set_creds(user_account_id, creds)
            .await?;
        Ok(())
    }

    pub async fn verify_password(
        &self,
        user_account_id: &str,
        password: &str,
    ) -> anyhow::Result<Result<(), ()>> {
        let creds = self.user_account_repo.creds(user_account_id).await?;
        Ok(match creds {
            None => {
                if password.is_empty() {
                    Ok(())
                } else {
                    Err(())
                }
            }
            Some(creds) => verify_password(password, &creds),
        })
    }
}

fn create_creds(password: &str) -> Creds {
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

fn verify_password(password: &str, creds: &Creds) -> Result<(), ()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_correct() {
        // Arrange.
        let password = "rightpass";
        let creds = create_creds(password);

        // Act.
        let res = verify_password(&password, &creds);

        // Assert.
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify_incorrect() {
        // Arrange.
        let password = "rightpass";
        let creds = create_creds(password);

        // Act.
        let res = verify_password("wrongpass", &creds);

        // Assert.
        assert!(res.is_err());
    }
}
