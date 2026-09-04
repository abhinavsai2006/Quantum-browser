//! Ed25519 Digital Signature Verification for Release Manifests and IPC Authentication

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct QualiumPublicKey {
    pub bytes: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QualiumSecretKey {
    pub bytes: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QualiumSignature {
    pub bytes: Vec<u8>,
}

pub struct QualiumSigner;

impl QualiumSigner {
    /// Generate a fresh Ed25519 keypair using OS CSPRNG
    pub fn generate_keypair() -> (QualiumSecretKey, QualiumPublicKey) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        (
            QualiumSecretKey {
                bytes: signing_key.to_bytes(),
            },
            QualiumPublicKey {
                bytes: verifying_key.to_bytes(),
            },
        )
    }

    /// Sign a message using the Ed25519 private key
    pub fn sign(sk: &QualiumSecretKey, message: &[u8]) -> Result<QualiumSignature, String> {
        let signing_key = SigningKey::from_bytes(&sk.bytes);
        let sig: Signature = signing_key.sign(message);
        Ok(QualiumSignature {
            bytes: sig.to_bytes().to_vec(),
        })
    }

    /// Verify an Ed25519 signature over a message
    pub fn verify(pk: &QualiumPublicKey, message: &[u8], signature: &QualiumSignature) -> bool {
        if signature.bytes.len() != 64 {
            return false;
        }
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&signature.bytes[..64]);

        if let Ok(verifying_key) = VerifyingKey::from_bytes(&pk.bytes) {
            let sig = Signature::from_bytes(&sig_arr);
            verifying_key.verify(message, &sig).is_ok()
        } else {
            false
        }
    }
}
