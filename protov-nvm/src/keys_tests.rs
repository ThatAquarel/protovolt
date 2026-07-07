#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    use crate::{
        CI_KEY_COUNT, CI_KEY_LEN, HW_KEY_COUNT, HW_KEY_LEN, PUBLIC_KEY_CIX, PUBLIC_KEY_CIX_SIG,
        PUBLIC_KEY_HWX, PUBLIC_KEY_HWX_SIG, PUBLIC_KEY_ROOT, ci_public_keys, hw_public_keys,
        verify_cix_manifest, verify_hwx_manifest,
    };

    #[test]
    fn public_key_cix_sig_verifies_with_root() {
        verify_cix_manifest()
            .expect("PUBLIC_KEY_CIX_SIG must verify PUBLIC_KEY_CIX with PUBLIC_KEY_ROOT");
    }

    #[test]
    fn verify_cix_manifest_matches_root_key() {
        let root = VerifyingKey::from_bytes(PUBLIC_KEY_ROOT)
            .expect("PUBLIC_KEY_ROOT must be valid Ed25519");
        let signature = Signature::from_bytes(PUBLIC_KEY_CIX_SIG);
        root.verify(PUBLIC_KEY_CIX, &signature)
            .expect("PUBLIC_KEY_CIX_SIG must be a valid root signature over PUBLIC_KEY_CIX");
    }

    #[test]
    fn ci_public_keys_splits_manifest_into_three_valid_keys() {
        let keys = ci_public_keys();
        assert_eq!(keys.len(), CI_KEY_COUNT);

        for (i, key) in keys.iter().enumerate() {
            VerifyingKey::from_bytes(key)
                .unwrap_or_else(|_| panic!("CI key {i} must be valid Ed25519"));
            let start = i * CI_KEY_LEN;
            assert_eq!(&key[..], &PUBLIC_KEY_CIX[start..start + CI_KEY_LEN]);
        }
    }

    #[test]
    fn public_key_hwx_sig_verifies_with_root() {
        verify_hwx_manifest()
            .expect("PUBLIC_KEY_HWX_SIG must verify PUBLIC_KEY_HWX with PUBLIC_KEY_ROOT");
    }

    #[test]
    fn verify_hwx_manifest_matches_root_key() {
        let root = VerifyingKey::from_bytes(PUBLIC_KEY_ROOT)
            .expect("PUBLIC_KEY_ROOT must be valid Ed25519");
        let signature = Signature::from_bytes(PUBLIC_KEY_HWX_SIG);
        root.verify(PUBLIC_KEY_HWX, &signature)
            .expect("PUBLIC_KEY_HWX_SIG must be a valid root signature over PUBLIC_KEY_HWX");
    }

    #[test]
    fn hw_public_keys_splits_manifest_into_three_valid_keys() {
        let keys = hw_public_keys();
        assert_eq!(keys.len(), HW_KEY_COUNT);

        for (i, key) in keys.iter().enumerate() {
            VerifyingKey::from_bytes(key)
                .unwrap_or_else(|_| panic!("HW key {i} must be valid Ed25519"));
            let start = i * HW_KEY_LEN;
            assert_eq!(&key[..], &PUBLIC_KEY_HWX[start..start + HW_KEY_LEN]);
        }
    }

    #[test]
    fn verify_serial_attestation_rejects_invalid_signature() {
        use crate::verify_serial_attestation;

        assert!(verify_serial_attestation("550e8400", &[0u8; 64]).is_err());
    }
}
