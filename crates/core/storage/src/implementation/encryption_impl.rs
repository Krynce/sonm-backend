use std::sync::OnceLock;

use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, AeadCore, AeadInPlace, OsRng},
};
use base64::{Engine, prelude::BASE64_STANDARD};

pub struct EncryptionKey {
    key: String,
    /// Built on first use: the base64 decode and AES key schedule are the same every time
    cipher: OnceLock<Aes256Gcm>,
}

impl EncryptionKey {
    pub async fn from_config() -> EncryptionKey {
        EncryptionKey::new(sonm_config::config().await.files.encryption_key)
    }

    pub fn new(key: String) -> EncryptionKey {
        EncryptionKey {
            key,
            cipher: OnceLock::new(),
        }
    }

    fn create_cipher(&self) -> &Aes256Gcm {
        self.cipher.get_or_init(|| {
            let key = &BASE64_STANDARD
                .decode(&self.key)
                .expect("valid base64 string")[..];
            let key: &Key<Aes256Gcm> = key.into();
            Aes256Gcm::new(key)
        })
    }
}

impl EncryptionKey {
    pub fn decrypt_buffer(&self, mut buf: Vec<u8>, iv: &str) -> anyhow::Result<Vec<u8>> {
        // A corrupt IV in the database must not take the process down.
        let iv = BASE64_STANDARD
            .decode(iv)
            .map_err(|error| anyhow::anyhow!("EncryptionRepository: invalid IV: {error}"))?;

        if iv.len() != 12 {
            return Err(anyhow::anyhow!(
                "EncryptionRepository: IV must be 12 bytes, got {}",
                iv.len()
            ));
        }

        let iv = Nonce::<typenum::consts::U12>::from_slice(&iv);

        self.create_cipher()
            .decrypt_in_place(iv, b"", &mut buf)
            .map_err(|error| {
                tracing::error!("{}", error);
                anyhow::anyhow!("EncryptionRepository: decryption failed")
            })?;

        Ok(buf)
    }

    pub fn encrypt_buffer(&self, buf: &[u8]) -> anyhow::Result<(Vec<u8>, String)> {
        let iv = Aes256Gcm::generate_nonce(&mut OsRng);

        let buf = self.create_cipher().encrypt(&iv, buf).map_err(|error| {
            tracing::error!("{}", error);
            anyhow::anyhow!("EncryptionRepository: encryption failed")
        })?;

        Ok((buf, BASE64_STANDARD.encode(iv)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_and_decrypt() {
        let encryption =
            EncryptionKey::new("XkbJ8gBzrouQ+15Ri23xCC81+aZE26Z6+gXzglFxOD4=".to_string());

        let buf: Vec<u8> = vec![67];
        let (ciphertext, iv) = encryption.encrypt_buffer(&buf[..]).unwrap();
        assert_eq!(ciphertext.len(), 17);

        let plaintext = encryption.decrypt_buffer(ciphertext, &iv).unwrap();
        assert_eq!(plaintext.len(), 1);
        assert_eq!(plaintext[0], 67);
    }

    #[test]
    fn broken_iv_errors_instead_of_panicking() {
        let encryption =
            EncryptionKey::new("XkbJ8gBzrouQ+15Ri23xCC81+aZE26Z6+gXzglFxOD4=".to_string());

        for iv in ["not base64!", "", &BASE64_STANDARD.encode([1, 2, 3])] {
            assert!(encryption.decrypt_buffer(vec![67; 17], iv).is_err());
        }
    }
}
