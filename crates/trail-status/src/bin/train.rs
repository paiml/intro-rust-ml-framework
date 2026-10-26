//! Lesson 3.2.4: train the trail-status model and save it as one `.apr` file.
//!
//! Fits TF-IDF features and one balanced LogisticRegression per trail on the
//! 401 training rows, scores both on the 70 validation rows, saves the bundle,
//! then loads it back from disk and predicts again.
//!
//! Provable contract: the saved file IS the model. Reloaded from disk, the
//! bundle predicts exactly what the in-memory model predicted on every
//! validation row; the hand-rolled TF-IDF used at inference matches aprender's
//! TfidfVectorizer on every row; and validation accuracy is at least 0.85 on
//! both trails.
//!
//! Training is deterministic, and the `.apr` header's `created_at` is fixed
//! (`bundle::SAVED_AT`), so training twice writes the same bytes: the same
//! sha256 every time.
//!
//! Run from the workspace root:
//!   cargo run --release -p trail-status --bin train [output.apr]
//! (default output: crates/trail-status/models/trail_status_model.apr, the file
//! the CLI and API embed)

use aprender::metrics::classification::accuracy;
use trail_status::bundle::{new_vectorizer, targets, to_f32, TrailStatusBundle};
use trail_status::data::{load, TRAIN_PATH, VAL_PATH};
use trail_status::{sha256_hex, MODEL_PATH};

const ACCURACY_FLOOR: f32 = 0.85;

fn main() {
    let train = load(TRAIN_PATH).expect("training data");
    let val = load(VAL_PATH).expect("validation data");
    println!("train: {} rows   val: {} rows", train.len(), val.len());

    // Step 1: fit TF-IDF and both classifiers on the training rows only.
    let model = TrailStatusBundle::train(&train).expect("train");
    println!(
        "vocabulary: {} terms (1- and 2-grams seen in at least 2 rows)",
        model.vocabulary.len()
    );

    // Step 2: score on the validation rows.
    let texts: Vec<&str> = val.iter().map(|r| r.text.as_str()).collect();
    let preds = model.predict_many(&texts);
    let (gold_b, gold_r) = targets(&val);
    let pred_b: Vec<usize> = preds
        .iter()
        .map(|p| usize::from(p.blankets_creek_open))
        .collect();
    let pred_r: Vec<usize> = preds
        .iter()
        .map(|p| usize::from(p.rope_mill_open))
        .collect();
    let acc_b = accuracy(&pred_b, &gold_b);
    let acc_r = accuracy(&pred_r, &gold_r);
    let exact = (0..val.len())
        .filter(|&i| pred_b[i] == gold_b[i] && pred_r[i] == gold_r[i])
        .count();
    println!(
        "val accuracy   Blankets Creek {acc_b:.3}   Rope Mill {acc_r:.3}   both right {exact}/{}",
        val.len()
    );

    // Step 3: save ONE file: vocabulary, IDF weights and both classifiers.
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| MODEL_PATH.to_string());
    model.save(&out).expect("save");
    let bytes = std::fs::read(&out).expect("read back");
    println!("saved {out} ({} bytes)", bytes.len());
    println!("sha256 {}", sha256_hex(&bytes));

    // Step 4: load it back from disk and predict again.
    let reloaded = TrailStatusBundle::from_bytes(&bytes).expect("reload");
    let again = reloaded.predict_many(&texts);
    let same = preds.iter().zip(&again).filter(|(a, b)| a == b).count();
    println!(
        "reloaded from disk: {same}/{} validation predictions identical",
        val.len()
    );

    // The inference-time TF-IDF must equal the library's, or the model sees different numbers than it trained on.
    let mut vectorizer = new_vectorizer();
    let train_texts: Vec<&str> = train.iter().map(|r| r.text.as_str()).collect();
    vectorizer.fit_transform(&train_texts).expect("fit");
    let library = to_f32(&vectorizer.transform(&texts).expect("transform"));
    let hand = model.features(&texts);
    let max_diff = library
        .as_slice()
        .iter()
        .zip(hand.as_slice())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);

    assert_eq!(
        same,
        val.len(),
        "the reloaded file must predict what was saved"
    );
    assert_eq!(library.shape(), hand.shape(), "same feature shape");
    assert!(
        max_diff < 1e-6,
        "TF-IDF differs from the library by {max_diff}"
    );
    assert!(
        acc_b.min(acc_r) >= ACCURACY_FLOOR,
        "accuracy below {ACCURACY_FLOOR}"
    );
    println!(
        "contract: the saved file is the model ({same}/{} identical after reload, TF-IDF matches the library, accuracy >= {ACCURACY_FLOOR}) OK",
        val.len()
    );
}
