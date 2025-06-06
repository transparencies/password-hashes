//! Implementation of the `password-hash` crate API.

use crate::{Params, scrypt};
use core::cmp::Ordering;
use password_hash::{Decimal, Error, Ident, Output, PasswordHash, PasswordHasher, Result, Salt};

/// Algorithm identifier
pub const ALG_ID: Ident = Ident::new_unwrap("scrypt");

/// scrypt type for use with [`PasswordHasher`].
///
/// See the [crate docs](crate) for a usage example.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Scrypt;

impl PasswordHasher for Scrypt {
    type Params = Params;

    fn hash_password_customized<'a>(
        &self,
        password: &[u8],
        alg_id: Option<Ident<'a>>,
        version: Option<Decimal>,
        params: Params,
        salt: impl Into<Salt<'a>>,
    ) -> Result<PasswordHash<'a>> {
        if !matches!(alg_id, Some(ALG_ID) | None) {
            return Err(Error::Algorithm);
        }

        // Versions unsupported
        if version.is_some() {
            return Err(Error::Version);
        }

        let salt = salt.into();
        let mut salt_arr = [0u8; 64];
        let salt_bytes = salt.decode_b64(&mut salt_arr)?;
        let len = params.len.unwrap_or(Params::RECOMMENDED_LEN);

        let output = Output::init_with(len, |out| {
            scrypt(password, salt_bytes, &params, out).map_err(|_| {
                let provided = if out.is_empty() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };

                password_hash::Error::OutputSize {
                    provided,
                    expected: 0, // TODO(tarcieri): calculate for `Ordering::Greater` case
                }
            })
        })?;

        Ok(PasswordHash {
            algorithm: ALG_ID,
            version: None,
            params: params.try_into()?,
            salt: Some(salt),
            hash: Some(output),
        })
    }
}
