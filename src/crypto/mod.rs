//! Cryptographic primitives for RoboChain

use borsh::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::{Signer, Verifier};
use rand_core::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::{Read, Write};

/// 32-byte hash type used throughout the system
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash(hash)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl BorshSerialize for Hash {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
    }
}

impl BorshDeserialize for Hash {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut bytes = [0u8; 32];
        reader.read_exact(&mut bytes)?;
        Ok(Hash(bytes))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

/// Ed25519 public key
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(pub [u8; 32]);

impl PublicKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hash(&self) -> Hash {
        Hash::from_bytes(&self.0)
    }
}

impl BorshSerialize for PublicKey {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
    }
}

impl BorshDeserialize for PublicKey {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut bytes = [0u8; 32];
        reader.read_exact(&mut bytes)?;
        Ok(PublicKey(bytes))
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", hex::encode(&self.0[..8]))
    }
}

/// Ed25519 signature
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_bytes::serialize(&self.0[..], serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = serde_bytes::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("signature must be 64 bytes"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(Signature(arr))
    }
}

impl Signature {
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl BorshSerialize for Signature {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
    }
}

impl BorshDeserialize for Signature {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut bytes = [0u8; 64];
        reader.read_exact(&mut bytes)?;
        Ok(Signature(bytes))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({}...)", hex::encode(&self.0[..8]))
    }
}

impl Default for Signature {
    fn default() -> Self {
        Signature([0u8; 64])
    }
}

/// Key pair for signing transactions
pub struct KeyPair {
    signing_key: ed25519_dalek::SigningKey,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        KeyPair { signing_key }
    }

    /// Create from seed bytes
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed);
        KeyPair { signing_key }
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        let vk = self.signing_key.verifying_key();
        PublicKey(vk.to_bytes())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        let sig = self.signing_key.sign(message);
        Signature(sig.to_bytes())
    }

    /// Verify a signature
    pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
        let Ok(vk) = ed25519_dalek::VerifyingKey::from_bytes(&public_key.0) else {
            return false;
        };
        let sig = ed25519_dalek::Signature::from_bytes(&signature.0);
        vk.verify(message, &sig).is_ok()
    }
}

/// Compute hash of serializable data
pub fn hash_data<T: Serialize>(data: &T) -> Hash {
    let bytes = serde_json::to_vec(data).unwrap_or_default();
    Hash::from_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify() {
        let keypair = KeyPair::generate();
        let message = b"Hello, RoboChain!";
        let signature = keypair.sign(message);

        assert!(KeyPair::verify(&keypair.public_key(), message, &signature));
        assert!(!KeyPair::verify(&keypair.public_key(), b"wrong message", &signature));
    }

    #[test]
    fn test_hash() {
        let hash1 = Hash::from_bytes(b"test data");
        let hash2 = Hash::from_bytes(b"test data");
        let hash3 = Hash::from_bytes(b"different data");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
