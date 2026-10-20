//! Lesson 3.2.2: explore the trail-status data before choosing a model.
//!
//! Prints row counts, the label balance (4 classes, and open/closed per
//! trail), status lengths, the most frequent words, and checks the
//! train/validation split for leakage.
//!
//! Provable contract: the data is fit to learn from. No validation text also
//! appears in training (no leakage), no training text appears twice, every
//! label is in 0..=3, and label_map.json names the four labels the code decodes.
//!
//! Run from the workspace root:
//!   cargo run --release -p trail-status --bin explore

use std::collections::{BTreeMap, HashSet};
use trail_status::data::{label_map, load, tokens, Example, LABELS, TRAIN_PATH, VAL_PATH};

/// Common words that say nothing about trail status.
const STOP: &[&str] = &["the", "are", "is", "at", "and", "to", "due", "of", "for", "will", "be", "a", "all", "in"];

fn balance(name: &str, rows: &[Example]) {
    let mut counts = [0usize; 4];
    rows.iter().for_each(|r| counts[r.label] += 1);
    println!("  {name}");
    for (label, n) in LABELS.iter().zip(counts) {
        println!("    {label:<26} {n:>3}  ({:>2.0}%)", 100.0 * n as f64 / rows.len() as f64);
    }
}

fn main() {
    let train = load(TRAIN_PATH).expect("training data");
    let val = load(VAL_PATH).expect("validation data");
    println!("rows: train {}   val {}", train.len(), val.len());

    println!("label balance");
    balance("train", &train);
    balance("val", &val);
    let open = |f: fn(&Example) -> bool| 100.0 * train.iter().filter(|r| f(r)).count() as f64 / train.len() as f64;
    println!(
        "train open rate: Blankets Creek {:.0}%   Rope Mill {:.0}%",
        open(Example::blankets_open),
        open(Example::rope_open)
    );

    let mut lengths: Vec<usize> = train.iter().map(|r| tokens(&r.text).len()).collect();
    lengths.sort_unstable();
    println!(
        "words per status: min {}   median {}   max {}",
        lengths[0],
        lengths[lengths.len() / 2],
        lengths[lengths.len() - 1]
    );

    let mut freq: BTreeMap<String, usize> = BTreeMap::new();
    for r in &train {
        for t in tokens(&r.text) {
            let t = t.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
            if !t.is_empty() && !STOP.contains(&t.as_str()) {
                *freq.entry(t).or_default() += 1;
            }
        }
    }
    let mut top: Vec<(String, usize)> = freq.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top: Vec<String> = top.iter().take(10).map(|(w, n)| format!("{w} {n}")).collect();
    println!("most frequent words: {}", top.join(", "));

    let train_texts: HashSet<&str> = train.iter().map(|r| r.text.as_str()).collect();
    let leaked = val.iter().filter(|r| train_texts.contains(r.text.as_str())).count();
    let duplicates = train.len() - train_texts.len();
    println!("leakage: {leaked} validation rows also in training   duplicates in training: {duplicates}");

    let map = label_map().expect("label map");
    assert_eq!(leaked, 0, "validation text must never appear in training");
    assert_eq!(duplicates, 0, "no training row may appear twice");
    assert!(train.iter().chain(&val).all(|r| r.label < 4), "every label in 0..=3");
    assert_eq!(map, LABELS, "label_map.json must name the labels the code decodes");
    println!("contract: the data is fit to learn from (train ∩ val = ∅, no duplicates, labels in 0..=3, label map agrees) OK");
}
