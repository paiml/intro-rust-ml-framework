//! Lesson 2.1.3: the Estimator pattern, `fit` then `predict`.
//!
//! Three different regression models, one function. `evaluate` is written
//! once against the `Estimator` trait and never names a concrete model.
//!
//! Provable contract: one interface, one behaviour. For every estimator,
//! `predict` returns exactly one value per input row, and `score` is the
//! same number as computing R² from `predict` by hand.
//!
//! Run from the workspace root:
//!   cargo run -p regression --bin estimators

use aprender::metrics::r_squared;
use aprender::model_selection::train_test_split;
use aprender::prelude::{Estimator, Lasso, LinearRegression, Matrix, Ridge, Vector};
use regression::{load_dataset, to_xy, SPLIT_SEED, TEST_SIZE};

/// Fit any estimator and report on held-out data. Returns test R².
fn evaluate<E: Estimator>(
    name: &str,
    mut model: E,
    train: (&Matrix<f32>, &Vector<f32>),
    test: (&Matrix<f32>, &Vector<f32>),
) -> f32 {
    // Step 1: fit learns from the training rows.
    model.fit(train.0, train.1).expect("fit");
    // Step 2: predict gives one number per test row.
    let predictions = model.predict(test.0);
    // Step 3: score reports R² on the test rows.
    let score = model.score(test.0, test.1);
    let by_hand = r_squared(&predictions, test.1);
    println!(
        "{name:<18} test R² {score:.4}   first prediction {:.1} h",
        predictions.as_slice()[0]
    );

    assert_eq!(
        predictions.len(),
        test.0.shape().0,
        "{name}: one prediction per row"
    );
    assert!(
        (score - by_hand).abs() < 1e-5,
        "{name}: score() must equal R² of predict()"
    );
    score
}

fn main() {
    let rows = load_dataset().expect("run `cargo run -p regression --bin gen-data` first");
    let (x, y) = to_xy(&rows);
    let (x_train, x_test, y_train, y_test) =
        train_test_split(&x, &y, TEST_SIZE, Some(SPLIT_SEED)).expect("valid split");
    let train = (&x_train, &y_train);
    let test = (&x_test, &y_test);

    let scores = [
        evaluate("LinearRegression", LinearRegression::new(), train, test),
        evaluate("Ridge(alpha=1.0)", Ridge::new(1.0), train, test),
        evaluate("Lasso(alpha=0.1)", Lasso::new(0.1), train, test),
    ];

    assert!(
        scores.iter().all(|s| s.is_finite()),
        "every estimator produced a finite score"
    );
    println!("contract: one interface, one behaviour (3 estimators: len(predict) == rows, score == R²(predict)) OK");
}
