//! Cryptographic primitives for Stark-Link.
//!
//! This module provides:
//!
//! * X25519 key-pair generation and ECDH shared-secret derivation.
//! * AES-256-GCM authenticated encryption / decryption.
//! * Secure random nonce generation.
//! * Device fingerprint derivation from a public key.

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret, StaticSecret};

use crate::error::{Result, StarkLinkError};

/// Length of an AES-256-GCM nonce in bytes.
pub const NONCE_LEN: usize = 12;

/// A long-lived X25519 key pair used for device identity and key exchange.
pub struct KeyPair {
    /// The secret (private) key — never leaves this device.
    secret: StaticSecret,
    /// The public key — shared freely with peers.
    public: PublicKey,
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public", &hex_encode(self.public.as_bytes()))
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

impl KeyPair {
    /// Generate a fresh random key pair.
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        tracing::debug!("generated new X25519 key pair");
        Self { secret, public }
    }

    /// Return the public key bytes (32 bytes).
    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public.as_bytes()
    }

    /// Return a reference to the [`PublicKey`].
    pub fn public_key(&self) -> &PublicKey {
        &self.public
    }

    /// Perform ECDH with the peer's public key to derive a shared secret.
    ///
    /// The returned [`SharedSecret`] should be used (after hashing) as the
    /// AES-256-GCM key.
    pub fn diffie_hellman(&self, peer_public: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(peer_public)
    }

    /// Derive a 32-byte AES key from the ECDH shared secret using SHA-256.
    pub fn derive_aes_key(&self, peer_public: &PublicKey) -> [u8; 32] {
        let shared = self.diffie_hellman(peer_public);
        let mut hasher = Sha256::new();
        hasher.update(shared.as_bytes());
        hasher.finalize().into()
    }

    /// Compute a human-readable fingerprint for the public key.
    ///
    /// The fingerprint is the first 16 hex characters of the SHA-256 hash of
    /// the public key bytes, formatted as `XXXX-XXXX-XXXX-XXXX`.
    pub fn fingerprint(&self) -> String {
        fingerprint_of(&self.public_key_bytes())
    }
}

/// Generate an ephemeral X25519 key pair (for single-use key exchanges).
///
/// Returns `(secret, public)`.
pub fn ephemeral_keypair() -> (EphemeralSecret, PublicKey) {
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (secret, public)
}

/// Compute the device fingerprint from raw public-key bytes.
///
/// Format: `XXXX-XXXX-XXXX-XXXX` (16 hex chars from SHA-256).
pub fn fingerprint_of(public_key_bytes: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key_bytes);
    let hash = hasher.finalize();
    let hex = hex_encode(&hash[..8]);
    format!(
        "{}-{}-{}-{}",
        &hex[0..4],
        &hex[4..8],
        &hex[8..12],
        &hex[12..16]
    )
}

/// Generate a cryptographically secure random nonce for AES-256-GCM.
pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Encrypt `plaintext` with AES-256-GCM.
///
/// Returns `nonce || ciphertext` (the first [`NONCE_LEN`] bytes of the output
/// are the nonce).
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| StarkLinkError::Crypto(format!("invalid AES key: {e}")))?;

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| StarkLinkError::Crypto(format!("encryption failed: {e}")))?;

    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt data previously encrypted with [`encrypt`].
///
/// Expects the input to be `nonce || ciphertext` (as produced by [`encrypt`]).
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < NONCE_LEN {
        return Err(StarkLinkError::Crypto(
            "ciphertext too short to contain a nonce".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| StarkLinkError::Crypto(format!("invalid AES key: {e}")))?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| StarkLinkError::Crypto(format!("decryption failed: {e}")))
}

// ── helpers ────────────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encryption() {
        let key = [42u8; 32];
        let plaintext = b"hello stark-link";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails() {
        let key = [42u8; 32];
        let wrong = [99u8; 32];
        let encrypted = encrypt(&key, b"secret").unwrap();
        assert!(decrypt(&wrong, &encrypted).is_err());
    }

    #[test]
    fn fingerprint_format() {
        let kp = KeyPair::generate();
        let fp = kp.fingerprint();
        // XXXX-XXXX-XXXX-XXXX
        assert_eq!(fp.len(), 19);
        assert_eq!(fp.chars().filter(|c| *c == '-').count(), 3);
    }

    #[test]
    fn ecdh_shared_secret() {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();
        let key_a = alice.derive_aes_key(bob.public_key());
        let key_b = bob.derive_aes_key(alice.public_key());
        assert_eq!(key_a, key_b);
    }
}
