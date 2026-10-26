//! Lesson 1.3.3, step 1: train the reopen model and save it as a SafeTensors file.
//!
//! Provable contract: lossless save. The file written to disk loads back as a
//! model with exactly the same coefficients and the same predictions.
//!
//! Run from the workspace root:
//!   cargo run -p first-steps --bin save-model

use first_steps::{estimate, load, save, train, MODEL_PATH};

fn main() {
    let model = train()
        .expect("train (run `cargo run -p regression --bin gen-data` if the CSV is missing)");
    let c = model.coefficients().as_slice();
    println!(
        "trained: hours = {:.2} {:+.2} * rainfall_mm {:+.2} * sun_hours",
        model.intercept(),
        c[0],
        c[1]
    );

    save(&model, MODEL_PATH).expect("save");
    let bytes = std::fs::metadata(MODEL_PATH).expect("stat").len();
    println!("saved:   {MODEL_PATH} ({bytes} bytes)");

    let back = load(MODEL_PATH).expect("load back");
    assert_eq!(
        back.coefficients().as_slice(),
        c,
        "coefficients survive the save"
    );
    assert_eq!(
        back.intercept(),
        model.intercept(),
        "intercept survives the save"
    );
    assert_eq!(
        estimate(&back, 20.0, 4.0),
        estimate(&model, 20.0, 4.0),
        "predictions survive the save"
    );
    println!(
        "contract: lossless save (reloaded model has identical coefficients and predictions) OK"
    );
}
