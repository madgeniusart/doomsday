//! AES-256-GCM encryption service with OS keyring key storage.
//! The encryption key is stored in the OS keyring (macOS Keychain, Windows DPAPI, Linux Secret Service).

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

const SERVICE_NAME: &str = "universal-browser-sync";
const KEY_NAME: &str = "backup-key";
const KEY_SIZE: usize = 32; // 256 bits

pub struct EncryptionService {
    key: [u8; KEY_SIZE],
}

impl EncryptionService {
    pub fn new() -> Result<Self> {
        let key = Self::load_or_create_key()?;
        Ok(Self { key })
    }

    fn load_or_create_key() -> Result<[u8; KEY_SIZE]> {
        // Try OS keyring first
        match Self::load_key_from_keyring() {
            Ok(key) => {
                log::info!("Loaded encryption key from OS keyring");
                Ok(key)
            }
            Err(e) => {
                log::warn!("Could not load key from keyring: {e}. Generating new key.");
                let key = Self::generate_key();
                Self::save_key_to_keyring(&key)?;
                Ok(key)
            }
        }
    }

    fn load_key_from_keyring() -> Result<[u8; KEY_SIZE]> {
        let entry = keyring::Entry::new(SERVICE_NAME, KEY_NAME)
            .context("Failed to create keyring entry")?;
        let secret = entry
            .get_password()
            .context("Key not found in keyring")?;
        let bytes = BASE64.decode(&secret).context("Invalid key encoding")?;
        if bytes.len() != KEY_SIZE {
            anyhow::bail!("Key size mismatch");
        }
        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&bytes);
        Ok(key)
    }

    fn save_key_to_keyring(key: &[u8; KEY_SIZE]) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, KEY_NAME)
            .context("Failed to create keyring entry")?;
        let encoded = BASE64.encode(key);
        entry
            .set_password(&encoded)
            .context("Failed to save key to keyring")?;
        log::info!("Encryption key saved to OS keyring");
        Ok(())
    }

    fn generate_key() -> [u8; KEY_SIZE] {
        use rand::RngCore;
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt data with AES-256-GCM.
    /// Output format: [12-byte nonce][ciphertext+tag]
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .context("Failed to initialize cipher")?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut output = Vec::with_capacity(nonce.len() + ciphertext.len());
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Decrypt data encrypted with `encrypt()`.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            anyhow::bail!("Ciphertext too short");
        }

        let (nonce_bytes, ciphertext) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .context("Failed to initialize cipher")?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
    }

    /// Derive a key from a password using PBKDF2-HMAC-SHA256
    #[allow(dead_code)]
    pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_SIZE] {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        let mut key = [0u8; KEY_SIZE];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // Use a test key instead of keyring for unit tests
        let key = [0u8; KEY_SIZE];
        let service = EncryptionService { key };

        let plaintext = b"Hello, Universal Browser Sync!";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let key = [42u8; KEY_SIZE];
        let service = EncryptionService { key };
        let plaintext = b"test data";

        let enc1 = service.encrypt(plaintext).unwrap();
        let enc2 = service.encrypt(plaintext).unwrap();

        // Each encryption should produce a different ciphertext (different nonce)
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_tamper_detection() {
        let key = [1u8; KEY_SIZE];
        let service = EncryptionService { key };
        let plaintext = b"sensitive data";

        let mut encrypted = service.encrypt(plaintext).unwrap();
        // Tamper with the ciphertext
        if let Some(byte) = encrypted.last_mut() {
            *byte ^= 0xff;
        }

        assert!(service.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_key_derivation() {
        let key1 = EncryptionService::derive_key("password", b"salt1234");
        let key2 = EncryptionService::derive_key("password", b"salt1234");
        let key3 = EncryptionService::derive_key("password", b"other567");

        assert_eq!(key1, key2); // same inputs → same key
        assert_ne!(key1, key3); // different salt → different key
    }
}
