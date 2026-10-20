//! Shared code for the regression lessons (2.1.2, 2.1.3).
//!
//! The dataset is GENERATED, and says so: `hours_until_reopen` for a trail
//! after a storm, from `rainfall_mm` and `sun_hours` since the rain stopped.
//! The true relationship is a stated formula plus bounded noise, so a lesson
//! can check what the model learned against what is actually there:
//!
//! ```text
//! hours_until_reopen = 14 + 1.5 * rainfall_mm - 1.0 * sun_hours + noise,  noise in [-3, 3)
//! ```
//!
//! The noise comes from a fixed-seed linear congruential generator, so the
//! CSV is byte-identical on every machine and every run.

use aprender::prelude::{Matrix, Vector};
use std::fmt::Write as _;

/// The formula the data is generated from. The model never sees these.
pub const TRUE_INTERCEPT: f32 = 14.0;
pub const TRUE_RAIN_COEF: f32 = 1.5;
pub const TRUE_SUN_COEF: f32 = -1.0;
/// Half-width of the uniform noise added to every target.
pub const NOISE_HALF_WIDTH: f32 = 3.0;
/// Rows in the committed dataset.
pub const N_ROWS: usize = 200;
/// Seed for the data generator.
pub const DATA_SEED: u64 = 2026;
/// Seed for the train/test shuffle. Same seed, same split, every run.
pub const SPLIT_SEED: u64 = 42;
/// Fraction of rows held out for testing.
pub const TEST_SIZE: f32 = 0.2;

pub const CSV_HEADER: &str = "rainfall_mm,sun_hours,hours_until_reopen";

/// A 64-bit linear congruential generator (Knuth's MMIX constants).
/// Small, dependency-free and identical on every platform.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Next value in `[0, 1)`.
    pub fn next_unit(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // Top 24 bits -> exactly representable in f32.
        ((self.0 >> 40) as f32) / ((1u64 << 24) as f32)
    }
}

/// One generated observation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    pub rainfall_mm: f32,
    pub sun_hours: f32,
    pub hours_until_reopen: f32,
}

/// Generate `n` rows from the stated formula.
pub fn generate(n: usize, seed: u64) -> Vec<Row> {
    let mut rng = Lcg::new(seed);
    (0..n)
        .map(|_| {
            // Round inputs to one decimal so the CSV reads like real measurements.
            let rainfall_mm = (rng.next_unit() * 400.0).round() / 10.0; // 0.0..=40.0
            let sun_hours = (rng.next_unit() * 100.0).round() / 10.0; // 0.0..=10.0
            let noise = (rng.next_unit() * 2.0 - 1.0) * NOISE_HALF_WIDTH;
            let y = TRUE_INTERCEPT + TRUE_RAIN_COEF * rainfall_mm + TRUE_SUN_COEF * sun_hours + noise;
            Row {
                rainfall_mm,
                sun_hours,
                hours_until_reopen: (y * 10.0).round() / 10.0,
            }
        })
        .collect()
}

/// Render rows as CSV, header first.
pub fn to_csv(rows: &[Row]) -> String {
    let mut out = String::from(CSV_HEADER);
    out.push('\n');
    for r in rows {
        writeln!(out, "{:.1},{:.1},{:.1}", r.rainfall_mm, r.sun_hours, r.hours_until_reopen)
            .expect("writing to a String cannot fail");
    }
    out
}

/// Parse the CSV written by [`to_csv`]. Refuses a wrong header or a short row.
pub fn parse_csv(text: &str) -> Result<Vec<Row>, String> {
    let mut lines = text.lines();
    match lines.next() {
        Some(h) if h == CSV_HEADER => {}
        other => return Err(format!("expected header {CSV_HEADER:?}, got {other:?}")),
    }
    lines
        .enumerate()
        .map(|(i, line)| {
            let f: Vec<f32> = line
                .split(',')
                .map(|v| v.trim().parse::<f32>())
                .collect::<Result<_, _>>()
                .map_err(|e| format!("row {}: {e}", i + 2))?;
            if f.len() != 3 {
                return Err(format!("row {}: expected 3 columns, got {}", i + 2, f.len()));
            }
            Ok(Row { rainfall_mm: f[0], sun_hours: f[1], hours_until_reopen: f[2] })
        })
        .collect()
}

/// Features (rainfall, sun) as an n x 2 matrix, and the target as a vector.
pub fn to_xy(rows: &[Row]) -> (Matrix<f32>, Vector<f32>) {
    let mut x = Vec::with_capacity(rows.len() * 2);
    let mut y = Vec::with_capacity(rows.len());
    for r in rows {
        x.push(r.rainfall_mm);
        x.push(r.sun_hours);
        y.push(r.hours_until_reopen);
    }
    let x = Matrix::from_vec(rows.len(), 2, x).expect("2 values per row");
    (x, Vector::from_vec(y))
}

/// Path of the committed dataset, relative to the workspace root.
pub const DATA_PATH: &str = "data/reopen-hours.csv";

/// Load the committed dataset from the workspace root.
pub fn load_dataset() -> Result<Vec<Row>, String> {
    let text = std::fs::read_to_string(DATA_PATH).map_err(|e| format!("read {DATA_PATH}: {e}"))?;
    parse_csv(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic() {
        assert_eq!(to_csv(&generate(N_ROWS, DATA_SEED)), to_csv(&generate(N_ROWS, DATA_SEED)));
    }

    #[test]
    fn a_different_seed_gives_different_data() {
        assert_ne!(generate(10, 1), generate(10, 2));
    }

    #[test]
    fn noise_stays_inside_its_stated_bound() {
        for r in generate(N_ROWS, DATA_SEED) {
            let clean = TRUE_INTERCEPT + TRUE_RAIN_COEF * r.rainfall_mm + TRUE_SUN_COEF * r.sun_hours;
            // +0.05 for the one-decimal rounding of the target.
            assert!((r.hours_until_reopen - clean).abs() <= NOISE_HALF_WIDTH + 0.05, "{r:?}");
        }
    }

    #[test]
    fn csv_round_trips() {
        let rows = generate(25, DATA_SEED);
        assert_eq!(parse_csv(&to_csv(&rows)).unwrap(), rows);
    }

    #[test]
    fn parse_refuses_a_wrong_header() {
        assert!(parse_csv("a,b,c\n1,2,3\n").is_err());
    }

    #[test]
    fn parse_refuses_a_short_row() {
        assert!(parse_csv(&format!("{CSV_HEADER}\n1.0,2.0\n")).is_err());
    }

    #[test]
    fn to_xy_keeps_rows_aligned() {
        let rows = generate(5, DATA_SEED);
        let (x, y) = to_xy(&rows);
        assert_eq!(x.shape(), (5, 2));
        assert_eq!(y.len(), 5);
        assert_eq!(x.get(3, 0), rows[3].rainfall_mm);
        assert_eq!(y.as_slice()[3], rows[3].hours_until_reopen);
    }
}
