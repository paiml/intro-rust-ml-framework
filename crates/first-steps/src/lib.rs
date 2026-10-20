//! Shared code for lessons 1.3.3 and 1.3.4.
//!
//! One small model, used two ways: the "trail reopen" linear regression from
//! the generated dataset in `data/reopen-hours.csv` (see the `regression`
//! crate), trained, saved as a SafeTensors file, then read back by the `apr`
//! CLI and by this library.
//!
//! Why SafeTensors and not `.apr`: in aprender 0.71.4 the library's
//! `format::save` writes the legacy APR v1 container, and the 0.71.4 `apr` CLI
//! refuses it ("Only APR v2 (APR\0) is supported"). `save_safetensors` /
//! `load_safetensors` are the format both sides of the same release read.

use aprender::model_selection::train_test_split;
use aprender::prelude::{Estimator, LinearRegression, Matrix};
use regression::{load_dataset, to_xy, SPLIT_SEED, TEST_SIZE};

/// Where the trained model is written, relative to the workspace root.
/// `models/` is gitignored: the file is build output, made by `save-model`.
pub const MODEL_PATH: &str = "models/reopen-hours.safetensors";

/// Train the reopen model on the training split. Deterministic: same data,
/// same split seed, same coefficients on every run.
pub fn train() -> Result<LinearRegression, String> {
    let rows = load_dataset()?;
    let (x, y) = to_xy(&rows);
    let (x_train, _x_test, y_train, _y_test) =
        train_test_split(&x, &y, TEST_SIZE, Some(SPLIT_SEED))?;
    let mut model = LinearRegression::new();
    model.fit(&x_train, &y_train).map_err(|e| e.to_string())?;
    Ok(model)
}

/// Save a model as SafeTensors: two tensors, `coefficients` and `intercept`.
pub fn save(model: &LinearRegression, path: &str) -> Result<(), String> {
    if let Some(dir) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    model.save_safetensors(path)
}

/// Load a saved reopen model. Refuses a file missing either tensor.
pub fn load(path: &str) -> Result<LinearRegression, String> {
    LinearRegression::load_safetensors(path).map_err(|e| format!("{path}: {e}"))
}

/// The model's estimate for one storm.
pub fn estimate(model: &LinearRegression, rainfall_mm: f32, sun_hours: f32) -> f32 {
    let x = Matrix::from_vec(1, 2, vec![rainfall_mm, sun_hours]).expect("1x2");
    model.predict(&x).as_slice()[0]
}

/// The same estimate computed by hand from the learned formula.
pub fn by_formula(model: &LinearRegression, rainfall_mm: f32, sun_hours: f32) -> f32 {
    let c = model.coefficients().as_slice();
    model.intercept() + c[0] * rainfall_mm + c[1] * sun_hours
}

/// Validate one input. The app refuses what the training data never covered.
pub fn check_inputs(rainfall_mm: f32, sun_hours: f32) -> Result<(), String> {
    if !rainfall_mm.is_finite() || !(0.0..=40.0).contains(&rainfall_mm) {
        return Err(format!("rain must be between 0 and 40 mm (the data's range), got {rainfall_mm}"));
    }
    if !sun_hours.is_finite() || !(0.0..=10.0).contains(&sun_hours) {
        return Err(format!("sun must be between 0 and 10 hours (the data's range), got {sun_hours}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace_root() {
        std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).unwrap();
    }

    #[test]
    fn training_is_deterministic() {
        workspace_root();
        let a = train().unwrap();
        let b = train().unwrap();
        assert_eq!(a.coefficients().as_slice(), b.coefficients().as_slice());
        assert_eq!(a.intercept(), b.intercept());
    }

    #[test]
    fn predict_matches_the_formula() {
        workspace_root();
        let m = train().unwrap();
        for (r, s) in [(0.0, 0.0), (20.0, 4.0), (40.0, 10.0)] {
            assert!((estimate(&m, r, s) - by_formula(&m, r, s)).abs() < 1e-3);
        }
    }

    #[test]
    fn save_then_load_round_trips() {
        workspace_root();
        let m = train().unwrap();
        let path = std::env::temp_dir().join(format!("reopen-roundtrip-{}.safetensors", std::process::id()));
        let path = path.to_str().unwrap();
        save(&m, path).unwrap();
        let back = load(path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(back.coefficients().as_slice(), m.coefficients().as_slice());
        assert_eq!(estimate(&back, 20.0, 4.0), estimate(&m, 20.0, 4.0));
    }

    #[test]
    fn load_refuses_a_missing_file() {
        assert!(load("/nonexistent/reopen.safetensors").is_err());
    }

    #[test]
    fn inputs_outside_the_data_are_refused() {
        assert!(check_inputs(20.0, 4.0).is_ok());
        assert!(check_inputs(-3.0, 4.0).is_err());
        assert!(check_inputs(41.0, 4.0).is_err());
        assert!(check_inputs(10.0, 11.0).is_err());
        assert!(check_inputs(f32::NAN, 1.0).is_err());
    }
}
