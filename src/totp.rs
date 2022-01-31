use crate::otp_result::OTPResult;
use crate::util::{base32_decode, get_code, hash_generic, MacDigest};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A TOTP generator
///
/// Follows the specification listed in [RFC6238]. Needs a secret,
/// a digest algorithm, a number of digits and a period on initialization.
///
/// The TOTP can then be generated using [`TOTP::get_otp`] or
/// [`TOTP::get_otp_with_custom_time_start`].
///
/// # Example
/// See the top-level README for an example of TOTP usage
///
/// In addition to the example, all other initialization methods can be
/// utilized in a similar manner.
///
/// [RFC6238]: https://datatracker.ietf.org/doc/html/rfc6238
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Hash)]
pub struct TOTP {
    /// The secret key used in the HMAC process.
    ///
    /// Often given as a Base32 key, which can be conveniently initialized
    /// using the [`TOTP::default_from_base32`] constructor.
    secret: Vec<u8>,

    /// The digest to use in the HMAC process.
    ///
    /// This value defaults to [`MacDigest::SHA1`] if not specified in a
    /// constructor.
    mac_digest: MacDigest,

    /// The number of digits of the code generated.
    ///
    /// This value defaults to 6 if not specified in a constructor.
    digits: u32,

    /// The period in seconds between two different generated code.
    ///
    /// This value defaults to 30 if not specified in a constructor.
    period: u64,
}

/// All initializer implementations for the [`TOTP`] struct
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl TOTP {
    /// Generates a new TOTP instance from a byte array representation of the
    /// secret, a digest algorithm, a number of digits,
    /// and a period in seconds.
    pub fn new(secret: &[u8], mac_digest: MacDigest, digits: u32, period: u64) -> Self {
        TOTP {
            secret: secret.to_vec(),
            mac_digest,
            digits,
            period,
        }
    }

    /// Generates a new TOTP instance from an utf8 representation of the
    /// secret, a digest algorithm, a number of digits,
    /// and a period in seconds.
    pub fn new_from_utf8(secret: &str, mac_digest: MacDigest, digits: u32, period: u64) -> Self {
        TOTP::new(secret.as_bytes(), mac_digest, digits, period)
    }

    /// Generates a new TOTP instance from a base32-encoded representation of
    /// the secret, a digest algorithm, a number of digits,
    /// and a period in seconds.
    ///
    /// # Panics
    /// This method panics if the provided string is not correctly base32 encoded.
    pub fn new_from_base32(secret: &str, mac_digest: MacDigest, digits: u32, period: u64) -> Self {
        let decoded = base32_decode(secret).expect("Failed to decode base32 string");
        TOTP::new(&decoded, mac_digest, digits, period)
    }

    /// Creates a new TOTP instance with a byte-array representation of the
    /// secret.
    ///
    /// Defaults to using [`MacDigest::SHA1`] as the digest for HMAC
    /// operations, with a 6-digit OTP output and a 30-second period.
    pub fn default_from_secret(secret: &[u8]) -> Self {
        TOTP::default_from_secret_with_digest(secret, MacDigest::SHA1)
    }

    /// Creates a new TOTP instance with a byte-array representation of the
    /// secret and a digest algorithm.
    ///
    /// Defaults to a 6-digit OTP output and a 30-second period.
    pub fn default_from_secret_with_digest(secret: &[u8], mac_digest: MacDigest) -> Self {
        TOTP::new(secret, mac_digest, 6, 30)
    }

    /// Creates a new TOTP instance with an utf8 representation of the secret.
    ///
    /// Defaults to using [`MacDigest::SHA1`] as the digest for HMAC
    /// operations, with a 6-digit OTP output and a 30-second period.
    pub fn default_from_utf8(secret: &str) -> Self {
        TOTP::default_from_utf8_with_digest(secret, MacDigest::SHA1)
    }

    /// Creates a new TOTP instance with an utf8 representation of the secret and
    /// a digest algorithm.
    ///
    /// Defaults to a 6-digit OTP output and a 30-second period.
    pub fn default_from_utf8_with_digest(secret: &str, mac_digest: MacDigest) -> Self {
        TOTP::new_from_utf8(secret, mac_digest, 6, 30)
    }

    /// Creates a new TOTP instance with a base32 representation of the secret.
    ///
    /// Defaults to using [`MacDigest::SHA1`] as the digest for HMAC
    /// operations, with a 6-digit OTP output and a 30-second period.
    ///
    /// # Panics
    /// This method panics if the provided string is not correctly
    /// base32-encoded.
    pub fn default_from_base32(secret: &str) -> Self {
        TOTP::default_from_base32_with_digest(secret, MacDigest::SHA1)
    }

    /// Creates a new TOTP instance with a base32 representation of the secret
    /// and a digest algorithm.
    ///
    /// Defaults to a 6-digit OTP output and a 30-second period.
    ///
    /// # Panics
    /// This method panics if the provided string is not correctly
    /// base32-encoded.
    pub fn default_from_base32_with_digest(secret: &str, mac_digest: MacDigest) -> Self {
        TOTP::new_from_base32(secret, mac_digest, 6, 30)
    }
}

/// All getters for the [`TOTP`] struct
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl TOTP {
    /// Gets the algorithm used for code generation.
    pub fn get_digest(&self) -> MacDigest {
        self.mac_digest
    }

    /// Gets the number of digits of the code.
    pub fn get_digits(&self) -> u32 {
        self.digits
    }

    /// Gets the period between code changes.
    pub fn get_period(&self) -> u64 {
        self.period
    }
}

/// All helper methods for totp generation
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl TOTP {

    /// Returns the time in seconds until an OTP refresh is needed.
    ///
    /// Just like the corresponding [`TOTP::get_otp`] method, this method
    /// takes the current system time in seconds.
    pub fn time_until_refresh(&self, time: u64) -> u64 {
        self.time_until_refresh_with_start(time, 0)
    }

    /// Returns the time in seconds until an OTP refresh is needed.
    ///
    /// Just like the corresponding [`TOTP::get_otp_with_custom_time_start`]
    /// method, this method takes the current time in seconds along with a
    /// specified start time in case an offset is desired. Both values must be
    /// in seconds.
    pub fn time_until_refresh_with_start(&self, time: u64, time_start: u64) -> u64 {
        let time_until = (time - time_start) % self.period;
        if time_until == 0 { self.period } else { time_until }
    }
}

/// All otp generation methods for the [`TOTP`] struct.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl TOTP {
    /// Generates and returns the TOTP value for the specified time.
    ///
    /// The time must be specified in seconds to calculate the correct
    /// one-time password.
    ///
    /// # Panics
    /// This method panics if the [`TOTP::get_otp_with_custom_time_start`]
    /// method does, which happens if the hash's secret is incorrectly given.
    pub fn get_otp(&self, time: u64) -> OTPResult {
        self.get_otp_with_custom_time_start(time, 0)
    }

    /// Generates and returns the TOTP value for the specified time.
    ///
    /// The time must be specified in seconds to calculate the correct
    /// one-time password.
    ///
    /// This method allows a custom start time to be provided.
    ///
    /// # Panics
    /// This method panics if the hash's secret is incorrectly given.
    pub fn get_otp_with_custom_time_start(&self, time: u64, time_start: u64) -> OTPResult {
        let time_count = (time - time_start) / self.period;

        let hash = hash_generic(&time_count.to_be_bytes(), &self.secret, &self.mac_digest);
        let offset = (hash[hash.len() - 1] & 0xf) as usize;
        let bytes: [u8; 4] = hash[offset..offset + 4]
            .try_into()
            .expect("Failed byte get");


        let code = get_code(bytes, self.digits);
        OTPResult::new(self.digits, code)
    }
}
