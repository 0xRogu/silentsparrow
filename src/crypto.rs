use base64ct::{Base64, Encoding};
use directories::ProjectDirs;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::fs;
use std::path::PathBuf;

const KEY_FILE_NAME: &str = "sparrow.key";

pub struct Crypto {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Crypto {
    pub fn load_or_create() -> Self {
        let key_path = Self::key_path();
        if key_path.exists() {
            let b64 = fs::read_to_string(&key_path).expect("Failed to read key file");
            let bytes = Base64::decode_vec(b64.trim()).expect("Invalid base64 in key file");
            let bytes: [u8; 32] = bytes.as_slice().try_into().expect("Key must be 32 bytes");
            let signing_key = SigningKey::from_bytes(&bytes);
            let verifying_key = signing_key.verifying_key();
            Crypto {
                signing_key,
                verifying_key,
            }
        } else {
            let crypto = Crypto::new();
            crypto.save_key(&key_path);
            crypto
        }
    }
    fn new() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Crypto {
            signing_key,
            verifying_key,
        }
    }

    fn key_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("org", "silent-sparrow", "Silent Sparrow")
            .expect("Unable to determine config directory");
        let config_dir = proj_dirs.config_dir();

        fs::create_dir_all(config_dir).expect("Failed to create config directory");
        config_dir.join(KEY_FILE_NAME)
    }

    fn save_key(&self, path: &PathBuf) {
        let b64 = Base64::encode_string(&self.signing_key.to_bytes());
        fs::write(path, b64).expect("Failed to write key file");
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
