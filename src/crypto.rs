use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand_core::{CryptoRng, RngCore};

pub struct Crypto {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Crypto {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        Crypto {
            signing_key,
            verifying_key,
        }
    }

    pub fn sign(&self, message: impl AsRef<[u8]>) -> [u8; 64] {
        self.signing_key.sign(message.as_ref()).to_bytes()
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }
}
