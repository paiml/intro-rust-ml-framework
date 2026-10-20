//! The trail-status classifier: is Blankets Creek open, and is Rope Mill open,
//! given a free-text status update from the SORBA Woodstock trail page?
//!
//! Trained on the hand-labelled data in `data/trail-status/` (see its
//! PROVENANCE.md). Lessons 3.1.x, 3.2.x and 3.3.x.
//!
//! - [`data`]: load the JSONL rows and decode the 4-class label into two booleans.
//! - [`bundle`]: TF-IDF features + two `LogisticRegression` models, trained,
//!   saved and loaded as ONE `.apr` file.
//! - [`model`]: that `.apr` embedded into the binary with `include_bytes!`.
//! - [`http`]: a tiny HTTP/1.1 client the contract checkers use (no curl needed).

pub mod bundle;
pub mod data;
pub mod http;
pub mod model;

/// Where `train` writes the bundle, and where `model` embeds it from.
/// Relative to the workspace root.
pub const MODEL_PATH: &str = "crates/trail-status/models/trail_status_model.apr";

/// The SHA-256 of some bytes, as lowercase hex (the same string `sha256sum` prints).
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn sha256_of_empty_input_is_the_known_value() {
        assert_eq!(
            super::sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
