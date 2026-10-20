//! Lesson 2.1.2: train and test a regression model with a held-out split.
//!
//! Predicts how many hours a trail stays closed after rain, from rainfall and
//! sun hours (the generated dataset in `data/reopen-hours.csv`).
//!
//! Provable contract: honest evaluation. The split is a partition (every row
//! lands in exactly one of train or test, 160 + 40 = 200), the model is
//! scored only on rows it never saw, and on those rows it explains at least
//! 90 % of the variance (R² >= 0.90).
//!
//! Run from the workspace root:
//!   cargo run -p regression --bin train-test

use aprender::metrics::{mae, r_squared};
use aprender::model_selection::train_test_split;
use aprender::prelude::{Estimator, LinearRegression, Matrix, Vector};
use regression::{load_dataset, to_xy, SPLIT_SEED, TEST_SIZE};

/// Every (rainfall, sun, target) triple as sortable integer tenths.
fn rows_of(x: &Matrix<f32>, y: &Vector<f32>) -> Vec<[i64; 3]> {
    let tenths = |v: f32| (v * 10.0).round() as i64;
    (0..x.shape().0)
        .map(|i| [tenths(x.get(i, 0)), tenths(x.get(i, 1)), tenths(y.as_slice()[i])])
        .collect()
}

fn main() {
    let rows = load_dataset().expect("run `cargo run -p regression --bin gen-data` first");
    let (x, y) = to_xy(&rows);
    println!("dataset: {} rows, features [rainfall_mm, sun_hours]", rows.len());

    // Step 1: Split BEFORE fitting. The test rows are locked away from here on.
    let (x_train, x_test, y_train, y_test) =
        train_test_split(&x, &y, TEST_SIZE, Some(SPLIT_SEED)).expect("valid split");
    println!("split:   {} train / {} test (seed {SPLIT_SEED})", y_train.len(), y_test.len());

    // Step 2: Fit on the training rows only.
    let mut model = LinearRegression::new();
    model.fit(&x_train, &y_train).expect("fit");
    let c = model.coefficients().as_slice();
    println!(
        "learned: hours = {:.2} {:+.2} * rainfall_mm {:+.2} * sun_hours",
        model.intercept(),
        c[0],
        c[1]
    );

    // Step 3: Score on rows the model has never seen.
    let train_r2 = r_squared(&model.predict(&x_train), &y_train);
    let pred_test = model.predict(&x_test);
    let test_r2 = r_squared(&pred_test, &y_test);
    let test_mae = mae(&pred_test, &y_test);
    println!("train R²: {train_r2:.3}");
    println!("test  R²: {test_r2:.3}   test MAE: {test_mae:.2} hours");

    // Step 4: One prediction a person can check.
    let storm = Matrix::from_vec(1, 2, vec![20.0, 4.0]).expect("1x2");
    let hours = model.predict(&storm).as_slice()[0];
    println!("20 mm of rain, 4 sun hours -> {hours:.1} hours until reopen");

    // Contract: the split is a partition of the dataset...
    let mut all = rows_of(&x, &y);
    let mut parts = rows_of(&x_train, &y_train);
    parts.extend(rows_of(&x_test, &y_test));
    all.sort_unstable();
    parts.sort_unstable();
    assert_eq!(y_train.len() + y_test.len(), rows.len(), "sizes must add up");
    assert_eq!(all, parts, "every row must land in exactly one side of the split");
    // ...and the held-out score clears the bar.
    assert!(test_r2 >= 0.90, "test R² {test_r2:.3} is below 0.90");
    println!("contract: honest evaluation (train ∪ test = all rows, disjoint; test R² {test_r2:.3} >= 0.90) OK");
}
