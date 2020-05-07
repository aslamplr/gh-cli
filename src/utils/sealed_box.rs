use anyhow::{anyhow, Result};
use base64;
use sodiumoxide::crypto::{box_::curve25519xsalsa20poly1305::PublicKey, sealedbox};

pub fn seal(message: &str, public_key_base64: &str) -> Result<String> {
    let public_key = base64::decode(public_key_base64)?;
    let public_key =
        PublicKey::from_slice(&public_key).ok_or(anyhow!("unable to create public key object"))?;

    let sealed_box = sealedbox::seal(message.as_bytes(), &public_key);
    let sealed_box_base64 = base64::encode(&sealed_box);
    Ok(sealed_box_base64)
}

#[cfg(test)]
mod tests {
    use super::seal;

    use anyhow::{anyhow, Result};
    use base64;
    use sodiumoxide::crypto::{
        box_::{curve25519xsalsa20poly1305::PublicKey, gen_keypair},
        sealedbox,
    };

    #[test]
    fn test_sealedbox_wrapper() -> Result<()> {
        let message = "Message inside the sealed box";
        let (pk, sk) = gen_keypair();
        let PublicKey(pk_bytes) = pk;
        let public_key_base64 = base64::encode(pk_bytes);
        let sealed_box_base64 = seal(&message, &public_key_base64)?;
        let c = base64::decode(&sealed_box_base64)?;
        let opened_box_content = sealedbox::open(c.as_slice(), &pk, &sk)
            .or_else(|()| Err(anyhow!("Something went wrong with opening the sealed box!")))?;
        let opened_box_content = std::str::from_utf8(&opened_box_content)?;
        assert_eq!(&message, &opened_box_content);
        Ok(())
    }
}
