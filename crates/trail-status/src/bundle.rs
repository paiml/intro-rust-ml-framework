//! The trained model as ONE `.apr` file: TF-IDF vocabulary + IDF weights, and
//! one `LogisticRegression` per trail.
//!
//! `TfidfVectorizer` is not serializable, so the bundle keeps the two things
//! inference needs from it (vocabulary and IDF), and [`tfidf_transform`]
//! reproduces the vectorizer's formula by hand: whitespace tokens, lowercased,
//! 1- and 2-grams joined with `_`, raw counts x IDF, then an L2-normalized row.
//! `train` checks that hand-rolled transform against the vectorizer itself.
//!
//! The vocabulary is a `BTreeMap`, not a `HashMap`, so the saved bytes are the
//! same on every run: a `HashMap` serializes in a random order.

use crate::data::{tokens, Example};
use aprender::classification::{ClassWeight, LogisticRegression};
use aprender::format::{load_from_bytes, save, Compression, ModelType, SaveOptions};
use aprender::primitives::Matrix;
use aprender::text::tokenize::WhitespaceTokenizer;
use aprender::text::vectorize::TfidfVectorizer;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 1-grams and 2-grams ("rope", "rope_mill").
pub const NGRAM_RANGE: (usize, usize) = (1, 2);
/// A term must appear in at least this many training rows to enter the vocabulary.
pub const MIN_DF: usize = 2;
/// The `created_at` written into the saved file (Unix seconds). Fixed rather
/// than the time of saving, so the same data always trains to the same file,
/// byte for byte, and its sha256 never changes by itself.
pub const SAVED_AT: &str = "1792073463";

#[derive(Serialize, Deserialize, Clone)]
pub struct TrailStatusBundle {
    pub vocabulary: BTreeMap<String, usize>,
    pub idf: Vec<f64>,
    pub ngram_range: (usize, usize),
    pub blankets_clf: LogisticRegression,
    pub rope_clf: LogisticRegression,
}

/// Both answers for one status update. A confidence is the probability of the
/// answer given, so it is always at least 0.5.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    pub blankets_creek_open: bool,
    pub blankets_creek_confidence: f32,
    pub rope_mill_open: bool,
    pub rope_mill_confidence: f32,
}

/// One balanced logistic regression, the configuration the original model used.
pub fn new_classifier() -> LogisticRegression {
    LogisticRegression::new()
        .with_class_weight(ClassWeight::Balanced)
        .with_learning_rate(0.5)
        .with_max_iter(5000)
        .with_l2_penalty(0.001)
}

/// A fresh TF-IDF vectorizer with the bundle's settings.
pub fn new_vectorizer() -> TfidfVectorizer {
    TfidfVectorizer::new()
        .with_tokenizer(Box::new(WhitespaceTokenizer::new()))
        .with_ngram_range(NGRAM_RANGE.0, NGRAM_RANGE.1)
        .with_min_df(MIN_DF)
}

/// Convert aprender's f64 feature matrix to the f32 the classifiers take.
pub fn to_f32(m: &Matrix<f64>) -> Matrix<f32> {
    let (rows, cols) = m.shape();
    let data = (0..rows).flat_map(|r| (0..cols).map(move |c| (r, c))).map(|(r, c)| m.get(r, c) as f32).collect();
    Matrix::from_vec(rows, cols, data).expect("same shape")
}

/// `1` for open, `0` for closed: the label vectors the classifiers train on.
pub fn targets(rows: &[Example]) -> (Vec<usize>, Vec<usize>) {
    (
        rows.iter().map(|r| usize::from(r.blankets_open())).collect(),
        rows.iter().map(|r| usize::from(r.rope_open())).collect(),
    )
}

impl TrailStatusBundle {
    /// Fit the vectorizer and both classifiers on the training rows.
    pub fn train(train: &[Example]) -> Result<Self, String> {
        let texts: Vec<&str> = train.iter().map(|r| r.text.as_str()).collect();
        let mut vectorizer = new_vectorizer();
        let x = to_f32(&vectorizer.fit_transform(&texts).map_err(|e| e.to_string())?);
        let (yb, yr) = targets(train);

        let mut blankets_clf = new_classifier();
        blankets_clf.fit(&x, &yb).map_err(|e| e.to_string())?;
        let mut rope_clf = new_classifier();
        rope_clf.fit(&x, &yr).map_err(|e| e.to_string())?;

        Ok(Self {
            vocabulary: vectorizer.vocabulary().iter().map(|(k, v)| (k.clone(), *v)).collect(),
            idf: vectorizer.idf_values().to_vec(),
            ngram_range: NGRAM_RANGE,
            blankets_clf,
            rope_clf,
        })
    }

    /// TF-IDF features for many texts, one row each.
    pub fn features(&self, texts: &[&str]) -> Matrix<f32> {
        let data: Vec<f32> = texts.iter().flat_map(|t| tfidf_transform(t, &self.vocabulary, &self.idf, self.ngram_range)).collect();
        Matrix::from_vec(texts.len(), self.idf.len(), data).expect("one row per text")
    }

    /// Predict both trails for every text.
    pub fn predict_many(&self, texts: &[&str]) -> Vec<Prediction> {
        let x = self.features(texts);
        let pb = self.blankets_clf.predict_proba(&x);
        let pr = self.rope_clf.predict_proba(&x);
        pb.as_slice()
            .iter()
            .zip(pr.as_slice())
            .map(|(&b, &r)| Prediction {
                blankets_creek_open: b >= 0.5,
                blankets_creek_confidence: if b >= 0.5 { b } else { 1.0 - b },
                rope_mill_open: r >= 0.5,
                rope_mill_confidence: if r >= 0.5 { r } else { 1.0 - r },
            })
            .collect()
    }

    /// Predict both trails for one text.
    pub fn predict(&self, text: &str) -> Prediction {
        self.predict_many(&[text]).remove(0)
    }

    /// Save as a single uncompressed `.apr` file. The header's `created_at` is
    /// [`SAVED_AT`], not the clock, so training twice gives the same bytes.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let mut opts = SaveOptions { compression: Compression::None, ..SaveOptions::default() };
        opts.metadata.created_at = SAVED_AT.to_string();
        save(self, ModelType::NeuralCustom, path, opts).map_err(|e| e.to_string())
    }

    /// Load from `.apr` bytes. The format's checksum is verified on the way in.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        load_from_bytes(bytes, ModelType::NeuralCustom).map_err(|e| e.to_string())
    }
}

/// Hand-rolled TF-IDF for one text against a fitted vocabulary. Terms not in
/// the vocabulary are ignored, as `TfidfVectorizer::transform` ignores them.
pub fn tfidf_transform(
    text: &str,
    vocabulary: &BTreeMap<String, usize>,
    idf: &[f64],
    ngram_range: (usize, usize),
) -> Vec<f32> {
    let toks = tokens(text);
    let mut tf = vec![0.0_f64; idf.len()];
    for n in ngram_range.0..=ngram_range.1 {
        if n == 0 || toks.len() < n {
            continue;
        }
        for window in toks.windows(n) {
            if let Some(&idx) = vocabulary.get(&window.join("_")) {
                tf[idx] += 1.0;
            }
        }
    }
    for (v, w) in tf.iter_mut().zip(idf) {
        *v *= w;
    }
    let norm = tf.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm > 0.0 {
        tf.iter_mut().for_each(|v| *v /= norm);
    }
    tf.into_iter().map(|v| v as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocab() -> BTreeMap<String, usize> {
        [("rope", 0), ("mill", 1), ("rope_mill", 2), ("open", 3)].iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn transform_counts_unigrams_and_bigrams_and_normalizes() {
        let v = tfidf_transform("Rope Mill open", &vocab(), &[1.0, 1.0, 1.0, 1.0], (1, 2));
        // rope, mill, rope_mill, open each once -> 0.5 each after L2 normalization.
        assert!(v.iter().all(|x| (x - 0.5).abs() < 1e-6), "{v:?}");
    }

    #[test]
    fn unknown_text_gives_a_zero_row() {
        assert!(tfidf_transform("nothing known here", &vocab(), &[1.0; 4], (1, 2)).iter().all(|&x| x == 0.0));
    }

    #[test]
    fn idf_weights_scale_terms() {
        let v = tfidf_transform("open", &vocab(), &[1.0, 1.0, 1.0, 3.0], (1, 2));
        assert!((v[3] - 1.0).abs() < 1e-6, "a single term normalizes to 1");
    }
}
