use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::OsRng;

#[derive(Debug, Clone)]
pub struct Identity {
    pub signing: SigningKey,
    pub verifying: VerifyingKey,
}

impl Identity {
    pub fn load_or_generate(path: &std::path::Path) -> anyhow::Result<Self> {
        if path.exists() {
            let data = std::fs::read(path)?;
            let bytes: [u8; 32] = data
                .as_slice()
                .try_into()
                .expect("identity file must be 32 bytes");
            let signing = SigningKey::from_bytes(&bytes);
            let verifying = VerifyingKey::from(&signing);
            Ok(Self { signing, verifying })
        } else {
            let mut csprng = OsRng;
            let signing = SigningKey::generate(&mut csprng);
            let verifying = VerifyingKey::from(&signing);
            std::fs::write(path, signing.to_bytes().as_slice())?;
            Ok(Self { signing, verifying })
        }
    }
}
