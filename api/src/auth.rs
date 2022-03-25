use crate::repo::user_account::{Access, ArcUserAccountRepo, Creds, UserAccount};
use data_encoding::HEXLOWER_PERMISSIVE;
use jsonwebtoken::{EncodingKey, Header};
use ring::digest::SHA512_OUTPUT_LEN;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

const N_ITER: u32 = 100_000;
const CREDENTIAL_LEN: usize = SHA512_OUTPUT_LEN;

#[derive(thiserror::Error, Debug)]
pub enum VerifyError {
    #[error("incorrect password")]
    IncorrectPassword,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

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
    ) -> anyhow::Result<Result<(), VerifyError>> {
        let creds = self.user_account_repo.creds(user_account_id).await?;
        Ok(match creds {
            None => {
                if password.is_empty() {
                    Ok(())
                } else {
                    Err(VerifyError::IncorrectPassword)
                }
            }
            Some(creds) => verify_password(password, &creds),
        })
    }
}

pub fn create_creds(password: &str) -> Creds {
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

pub fn verify_password(password: &str, creds: &Creds) -> Result<(), VerifyError> {
    let password_hash = HEXLOWER_PERMISSIVE
        .decode(creds.password_hash.as_bytes())
        .map_err(anyhow::Error::from)?;
    let salt = HEXLOWER_PERMISSIVE
        .decode(creds.salt.as_bytes())
        .map_err(anyhow::Error::from)?;
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(N_ITER).unwrap(),
        &salt,
        password.as_bytes(),
        &password_hash,
    )
    .map_err(|_| VerifyError::IncorrectPassword)
}

/// A user account bearer token. The token is missing recommended fields like exp and iat for simplicity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject is user account ID.
    sub: String,
    access: Access,
}

#[derive(Debug, Clone)]
pub struct TokenProvider {
    pub private_key: String,
}

impl TokenProvider {
    pub fn create_token(&self, user_account: &UserAccount) -> anyhow::Result<String> {
        Ok(jsonwebtoken::encode(
            &Header::default(),
            &Claims {
                sub: user_account.id.to_string(),
                access: user_account.access,
            },
            &EncodingKey::from_secret(self.private_key.as_bytes()),
        )?)
    }
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
        let res = verify_password(password, &creds);

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
        assert!(matches!(res, Err(VerifyError::IncorrectPassword)));
    }
}
