use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{Error, PasswordHasher, SaltString},
};
use password_hash::try_generate_salt;
use tokio::task::spawn_blocking;

use crate::error::{AuthAppError, AuthAppErrorTrait};

#[derive(Clone, Debug)]
pub struct PasswordChecker {
    pub argon2: Argon2<'static>,
}

impl PasswordChecker {
    pub fn new(argon2: Argon2<'static>) -> Self {
        Self { argon2 }
    }

    fn inner_hash_password_sync(
        argon2: &Argon2<'static>,
        password: &str,
    ) -> Result<String, AuthAppError> {
        let raw_salt =
            try_generate_salt().map_err(|e| AuthAppError::password_hashing(Some(e.into())))?;
        let salt = SaltString::encode_b64(&raw_salt)
            .map_err(|e| AuthAppError::password_hashing(Some(format!("{}", e).into())))?;
        let hashed = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthAppError::password_hashing(Some(format!("{}", e).into())))?
            .to_string();
        Ok(hashed)
    }

    pub fn hash_password_sync(&self, password: impl AsRef<str>) -> Result<String, AuthAppError> {
        Self::inner_hash_password_sync(&self.argon2, password.as_ref())
    }

    pub async fn take_hash_password(
        self,
        password: impl AsRef<str>,
    ) -> Result<String, AuthAppError> {
        let password = password.as_ref().to_string();
        spawn_blocking(move || Self::inner_hash_password_sync(&self.argon2, &password))
            .await
            .map_err(|e| AuthAppError::password_hashing(Some(e.into())))?
    }

    pub async fn hash_password(&self, password: impl AsRef<str>) -> Result<String, AuthAppError> {
        self.clone().take_hash_password(password).await
    }

    fn inner_verify_password(
        argon2: &Argon2<'static>,
        password: &str,
        hashed: &str,
    ) -> Result<bool, AuthAppError> {
        let hashed = PasswordHash::new(hashed)
            .map_err(|e| AuthAppError::password_hashing(Some(format!("{}", e).into())))?;
        match argon2.verify_password(password.as_bytes(), &hashed) {
            Ok(()) => Ok(true),
            Err(error) => match error {
                Error::Password => Ok(false),
                _ => Err(AuthAppError::password_hashing(Some(
                    format!("{}", error).into(),
                ))),
            },
        }
    }

    pub fn verify_password_sync(
        &self,
        password: impl AsRef<str>,
        hashed: impl AsRef<str>,
    ) -> Result<bool, AuthAppError> {
        Self::inner_verify_password(&self.argon2, password.as_ref(), hashed.as_ref())
    }

    pub async fn take_verify_password(
        self,
        password: &str,
        hashed: &str,
    ) -> Result<bool, AuthAppError> {
        let password = password.to_string();
        let hashed = hashed.to_string();
        spawn_blocking(move || Self::inner_verify_password(&self.argon2, &password, &hashed))
            .await
            .map_err(|e| AuthAppError::password_hashing(Some(e.into())))?
    }

    pub async fn verify_password(
        &self,
        password: &str,
        hashed: &str,
    ) -> Result<bool, AuthAppError> {
        self.clone().take_verify_password(password, hashed).await
    }
}

impl Default for PasswordChecker {
    fn default() -> Self {
        Self::new(Argon2::default())
    }
}
