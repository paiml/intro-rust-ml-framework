//! Loading the trail-status data.
//!
//! Each JSONL row is `{"input": text, "label": 0..=3}`. The label folds two
//! yes/no answers into one number (see `data/trail-status/label_map.json`):
//!
//! | label | name | Blankets Creek | Rope Mill |
//! |---|---|---|---|
//! | 0 | both_open | open | open |
//! | 1 | blankets_open_rope_closed | open | closed |
//! | 2 | blankets_closed_rope_open | closed | open |
//! | 3 | both_closed | closed | closed |

use serde::Deserialize;

pub const TRAIN_PATH: &str = "data/trail-status/train.jsonl";
pub const VAL_PATH: &str = "data/trail-status/val.jsonl";
pub const LABEL_MAP_PATH: &str = "data/trail-status/label_map.json";

/// The label names, in label-index order. Checked against `label_map.json`.
pub const LABELS: [&str; 4] = [
    "both_open",
    "blankets_open_rope_closed",
    "blankets_closed_rope_open",
    "both_closed",
];

/// One labelled status update.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Example {
    #[serde(rename = "input")]
    pub text: String,
    pub label: usize,
}

impl Example {
    pub fn blankets_open(&self) -> bool {
        decode(self.label).0
    }

    pub fn rope_open(&self) -> bool {
        decode(self.label).1
    }
}

/// Label index -> (Blankets Creek open, Rope Mill open). Panics on a label
/// outside 0..=3; [`parse_jsonl`] refuses those first.
pub fn decode(label: usize) -> (bool, bool) {
    match label {
        0 => (true, true),
        1 => (true, false),
        2 => (false, true),
        3 => (false, false),
        other => panic!("label {other} is outside 0..=3"),
    }
}

/// Parse JSONL text. Refuses a malformed row or a label outside 0..=3.
pub fn parse_jsonl(text: &str) -> Result<Vec<Example>, String> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, line)| {
            let ex: Example =
                serde_json::from_str(line).map_err(|e| format!("line {}: {e}", i + 1))?;
            if ex.label >= LABELS.len() {
                return Err(format!(
                    "line {}: label {} is outside 0..=3",
                    i + 1,
                    ex.label
                ));
            }
            Ok(ex)
        })
        .collect()
}

/// Load a JSONL file, relative to the workspace root.
pub fn load(path: &str) -> Result<Vec<Example>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    parse_jsonl(&text)
}

/// The label names `label_map.json` declares, in order.
pub fn label_map() -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct LabelMap {
        labels: Vec<String>,
    }
    let text = std::fs::read_to_string(LABEL_MAP_PATH)
        .map_err(|e| format!("read {LABEL_MAP_PATH}: {e}"))?;
    let map: LabelMap =
        serde_json::from_str(&text).map_err(|e| format!("{LABEL_MAP_PATH}: {e}"))?;
    Ok(map.labels)
}

/// Whitespace tokens, lowercased: exactly what the model's TF-IDF step sees.
pub fn tokens(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_lowercase).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_decode_to_both_trails() {
        assert_eq!(decode(0), (true, true));
        assert_eq!(decode(1), (true, false));
        assert_eq!(decode(2), (false, true));
        assert_eq!(decode(3), (false, false));
    }

    #[test]
    fn parse_reads_text_and_label() {
        let rows = parse_jsonl("{\"input\": \"all trails open\", \"label\": 0}\n\n{\"input\": \"closed\", \"label\": 3}\n").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].text, "all trails open");
        assert!(rows[0].blankets_open() && rows[0].rope_open());
        assert!(!rows[1].blankets_open() && !rows[1].rope_open());
    }

    #[test]
    fn parse_refuses_a_label_outside_the_map() {
        assert!(parse_jsonl("{\"input\": \"x\", \"label\": 4}\n").is_err());
    }

    #[test]
    fn parse_refuses_a_malformed_row() {
        assert!(parse_jsonl("{\"input\": \"x\"}\n").is_err());
    }

    #[test]
    fn tokens_are_lowercased_whitespace_splits() {
        assert_eq!(
            tokens("Blankets  Creek is OPEN."),
            vec!["blankets", "creek", "is", "open."]
        );
    }
}
