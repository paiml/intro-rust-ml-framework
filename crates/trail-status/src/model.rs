//! The trained bundle, embedded in the binary at compile time (lesson 3.1.4).
//!
//! `include_bytes!` copies the `.apr` file into the executable when it is
//! built, and `TrailStatusBundle::from_bytes` (aprender's
//! `format::load_from_bytes`) reads it back from memory. The binaries that use
//! this need no `.apr` file on disk at run time.
//!
//! Trade-off: the model is fixed at build time. A new model means
//! `cargo build`, not copying a file next to the binary.

use crate::bundle::{Prediction, TrailStatusBundle};

/// The bytes of `crates/trail-status/models/trail_status_model.apr`, as they
/// were when this binary was built.
// Step 1: copy the model file into the program when it is compiled.
pub static MODEL_BYTES: &[u8] = include_bytes!("../models/trail_status_model.apr");

/// The model this binary carries.
pub struct TrailStatusModel {
    bundle: TrailStatusBundle,
}

impl TrailStatusModel {
    /// Load the embedded model. Cannot fail on a missing file: there is none.
    pub fn load() -> Self {
        // Step 2: rebuild the model from those bytes, in memory.
        let bundle = TrailStatusBundle::from_bytes(MODEL_BYTES)
            .expect("the embedded model is corrupt: a build problem, not a runtime one");
        Self { bundle }
    }

    pub fn predict(&self, text: &str) -> Prediction {
        self.bundle.predict(text)
    }
}
