//! Writes `data/reopen-hours.csv`, the generated dataset for lessons 2.1.2 and 2.1.3.
//!
//! Provable contract: determinism. Generating twice from the same seed gives
//! byte-identical CSV, and the file on disk is exactly that CSV.
//!
//! Run from the workspace root:
//!   cargo run -p regression --bin gen-data

use regression::{generate, to_csv, DATA_PATH, DATA_SEED, N_ROWS};

fn main() {
    let csv = to_csv(&generate(N_ROWS, DATA_SEED));
    std::fs::write(DATA_PATH, &csv).unwrap_or_else(|e| panic!("write {DATA_PATH}: {e}"));
    println!("wrote {DATA_PATH}: {N_ROWS} rows, seed {DATA_SEED}");

    let again = to_csv(&generate(N_ROWS, DATA_SEED));
    let on_disk = std::fs::read_to_string(DATA_PATH).expect("read back");
    assert_eq!(csv, again, "same seed must give the same data");
    assert_eq!(csv, on_disk, "the file must hold exactly what was generated");
    println!("contract: determinism (same seed => byte-identical CSV) OK");
}
