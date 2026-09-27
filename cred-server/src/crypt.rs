use argon2::Argon2;
use base64::prelude::*;
use sha2::{Digest, Sha512};

pub fn transform(key: String, salt: String) -> String {
    let salt = Sha512::digest(salt.as_bytes());
    let mut result = [0u8; 256];

    Argon2::default()
        .hash_password_into(key.as_bytes(), &salt, &mut result)
        .unwrap();

    let normalised: String = Sha512::digest(&result)
        .iter()
        .map(|c| c.to_string())
        .collect();
    let normalised = BASE64_STANDARD.encode(normalised);

    normalised
}
