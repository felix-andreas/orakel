use sha2::{Digest, Sha256};

/// Lowercase hex sha256 of `bytes` — the content address under `blobs/<sha256>`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn known_vector_abc() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn known_vector_longer() {
        // sha256 of "orakel" computed with coreutils sha256sum.
        assert_eq!(
            sha256_hex(b"orakel"),
            "73a74b697ac520e9cded3db30318bee8287294ca1e71809d4e0fbfd55a49f145"
        );
    }
}
