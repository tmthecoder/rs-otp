// Implementation of the HOTP standard according to RFC4226 by Tejas Mehta

use crate::otp_result::OTPResult;
use crate::util::{base32_decode, get_code, hash_generic, MacDigest};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A HOTP Generator
///
/// Follows the specification listed in [RFC4226]. Needs a secret and
/// digit count on initialization.
///
/// The HOTP can then be generated using [`HOTP::get_otp`].
///
/// # Example
/// See the top-level README for an example of HOTP usage
///
/// In addition to the example, all other initialization methods can be
/// utilized in a similar manner.
///
/// [RFC4226]: https://datatracker.ietf.org/doc/html/rfc4226

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Hash)]
pub struct HOTP {
    /// The secret key used in the HMAC process.
    ///
    /// Often given as a Base32 key, which can be conveniently initialized
    /// using the [`HOTP::default_from_base32`] constructor.
    secret: Vec<u8>,

    /// The number of digits of the code generated.
    ///
    /// This value defaults to 6 if not specified in a constructor.
    digits: u32,
}

/// All initializer implementations for the [`HOTP`] struct.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HOTP {
    /// Creates a new HOTP instance with a byte-array representation
    /// of the secret and specified digit count.
    ///
    /// Since only SHA1 was specified in the reference implementation and
    /// RFC specification, there's no need to initialize with a digest object.
    pub fn new(secret: &[u8], digits: u32) -> Self {
        HOTP {
            secret: secret.to_vec(),
            digits,
        }
    }

    /// Creates a new HOTP instance from a utf8-encoded string secret
    /// and specified digit count.
    pub fn new_from_utf8(secret: &str, digits: u32) -> Self {
        HOTP::new(secret.as_bytes(), digits)
    }

    /// Creates a new HOTP instance from a base32-encoded string secret
    /// and specified digit count.
    ///
    /// # Panics
    /// This method panics if the provided string is not correctly
    /// base32-encoded.
    pub fn new_from_base32(secret: &str, digits: u32) -> Self {
        let decoded = base32_decode(secret).expect("Failed to decode base32 string");
        HOTP::new(&decoded, digits)
    }

    /// Creates a new HOTP instance from a byte-array representation of
    /// the secret and a default digit count of 6.
    pub fn default_from_secret(secret: &[u8]) -> Self {
        HOTP::new(secret, 6)
    }

    /// Creates a new HOTP instance from an utf8-encoded string secret
    /// and a default digit count of 6..
    pub fn default_from_utf8(secret: &str) -> Self {
        HOTP::new_from_utf8(secret, 6)
    }

    /// Creates a new HOTP instance from a base32-encoded string secret
    /// and a default digit count of 6..
    ///
    /// # Panics
    /// This method panics if the provided string is not correctly
    /// base32-encoded.
    pub fn default_from_base32(secret: &str) -> Self {
        HOTP::new_from_base32(secret, 6)
    }
}

/// All getters for the ['HOTP'] struct
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HOTP {
    /// Gets the number of digits of the code.
    pub fn get_digits(&self) -> u32 {
        self.digits
    }
}

/// All otp generation methods for the [`HOTP`] struct.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HOTP {
    /// Generates and returns the HOTP value.
    ///
    /// Uses the given counter value.
    ///
    /// # Panics
    /// This method panics if the hash's secret is incorrectly given.
    pub fn get_otp(&self, counter: u64) -> OTPResult {
        let hash = hash_generic(&counter.to_be_bytes(), &self.secret, &MacDigest::SHA1);
        let offset = (hash[hash.len() - 1] & 0xf) as usize;
        let bytes: [u8; 4] = hash[offset..offset + 4]
            .try_into()
            .expect("Failed byte get");

        let code = get_code(bytes, self.digits);
        OTPResult::new(self.digits, code)
    }
}
