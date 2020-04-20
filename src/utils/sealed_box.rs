use crate::Result;
use base64;
use sodiumoxide::crypto::{box_::curve25519xsalsa20poly1305::PublicKey, sealedbox};

pub fn seal(message: &str, public_key_base64: &str) -> Result<String> {
    let public_key = base64::decode(public_key_base64)?;
    let public_key =
        PublicKey::from_slice(&public_key).ok_or("unable to create public key object")?;

    let sealed_box = sealedbox::seal(message.as_bytes(), &public_key);
    let sealed_box_base64 = base64::encode(&sealed_box);
    Ok(sealed_box_base64)
}
