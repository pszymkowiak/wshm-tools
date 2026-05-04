//! AES-256-GCM seal/open over the master key.
//!
//! The master key is loaded once at startup (32 bytes from
//! `WSHM_MASTER_KEY`). Each plaintext is encrypted with a fresh 12-byte
//! random nonce. AAD bytes bind the ciphertext to a logical identifier
//! (`scope|slug|key`) so an attacker cannot copy a ciphertext from one
//! row into another and have the GCM tag still validate.
//!
//! On-disk record layout:
//!   nonce      BLOB(12)  random per-write
//!   ciphertext BLOB(N+16) plaintext + 16-byte GCM tag
//!   aad        BLOB      "scope|slug|key" UTF-8 (nullable for global keys)

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{anyhow, bail, Context, Result};
use rand::RngCore;

/// 32-byte AES-256 key. Construct via [`MasterKey::from_hex`].
pub struct MasterKey([u8; 32]);

impl MasterKey {
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str.trim()).context("master key is not valid hex")?;
        if bytes.len() != 32 {
            bail!(
                "master key must be 32 bytes (64 hex chars), got {} bytes",
                bytes.len()
            );
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Generate a fresh random 32-byte key (used by the bootstrap script).
    pub fn generate() -> Self {
        let mut arr = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut arr);
        Self(arr)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// GCM cipher wrapping a [`MasterKey`].
pub struct Cipher {
    inner: Aes256Gcm,
}

impl Cipher {
    pub fn new(key: &MasterKey) -> Self {
        let k = Key::<Aes256Gcm>::from_slice(&key.0);
        Self {
            inner: Aes256Gcm::new(k),
        }
    }

    /// Encrypt `plaintext` with a fresh random nonce. Returns
    /// `(nonce_12B, ciphertext_with_tag)`.
    pub fn seal(&self, plaintext: &[u8], aad: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .inner
            .encrypt(nonce, Payload { msg: plaintext, aad })
            .map_err(|e| anyhow!("AES-GCM encrypt failed: {e}"))?;
        Ok((nonce_bytes.to_vec(), ciphertext))
    }

    pub fn open(&self, nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        if nonce.len() != 12 {
            bail!("nonce must be 12 bytes, got {}", nonce.len());
        }
        let n = Nonce::from_slice(nonce);
        self.inner
            .decrypt(n, Payload { msg: ciphertext, aad })
            .map_err(|e| anyhow!("AES-GCM decrypt failed: {e}"))
    }
}

/// Build the AAD bytes for a given record from its logical identity.
pub fn aad_for(scope: &str, slug: Option<&str>, key: &str) -> Vec<u8> {
    let s = format!("{}|{}|{}", scope, slug.unwrap_or(""), key);
    s.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let k = MasterKey::generate();
        let c = Cipher::new(&k);
        let aad = aad_for("global", None, "github_token");
        let (nonce, ct) = c.seal(b"ghp_secret", &aad).unwrap();
        let pt = c.open(&nonce, &ct, &aad).unwrap();
        assert_eq!(pt, b"ghp_secret");
    }

    #[test]
    fn aad_mismatch_fails() {
        let k = MasterKey::generate();
        let c = Cipher::new(&k);
        let aad = aad_for("global", None, "github_token");
        let (nonce, ct) = c.seal(b"ghp_secret", &aad).unwrap();
        // Different AAD must fail (defends against row-substitution attack).
        let bad_aad = aad_for("global", None, "anthropic_key");
        assert!(c.open(&nonce, &ct, &bad_aad).is_err());
    }
}
