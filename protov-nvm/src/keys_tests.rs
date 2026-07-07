#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    use crate::{PUBLIC_KEY_CIX, PUBLIC_KEY_CIX_SIG, PUBLIC_KEY_ROOT};

    #[test]
    fn public_key_cix_sig_verifies_with_root() {
        let root = VerifyingKey::from_bytes(PUBLIC_KEY_ROOT)
            .expect("PUBLIC_KEY_ROOT must be valid Ed25519");
        let signature = Signature::from_bytes(PUBLIC_KEY_CIX_SIG);
        root.verify(PUBLIC_KEY_CIX, &signature)
            .expect("PUBLIC_KEY_CIX_SIG must be a valid root signature over PUBLIC_KEY_CIX");
    }
}
