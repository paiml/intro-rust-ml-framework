//! Lesson 3.2.3: decide on a model by comparing candidates on held-out data.
//!
//! Every candidate sees the same TF-IDF features of the 401 training rows and
//! is scored on the same 70 validation rows, one classifier per trail:
//!
//! - majority class: always answer the most common training answer (the floor);
//! - multinomial naive Bayes;
//! - logistic regression, unweighted;
//! - logistic regression with balanced class weights.
//!
//! Provable contract: the choice is earned. The chosen candidate (most rows
//! with both trails right) beats the majority-class baseline on BOTH trails.
//!
//! Run from the workspace root:
//!   cargo run --release -p trail-status --bin compare

use aprender::classification::{LogisticRegression, MultinomialNB};
use aprender::metrics::classification::accuracy;
use aprender::primitives::Matrix;
use trail_status::bundle::{new_classifier, new_vectorizer, targets, to_f32};
use trail_status::data::{load, TRAIN_PATH, VAL_PATH};

struct Score {
    name: &'static str,
    blankets: f32,
    rope: f32,
    both: usize,
}

/// Train one classifier per trail with `fit_predict` and score it.
fn score(
    name: &'static str,
    gold: (&[usize], &[usize]),
    mut fit_predict: impl FnMut(&[usize]) -> Vec<usize>,
    train_y: (&[usize], &[usize]),
) -> Score {
    let pb = fit_predict(train_y.0);
    let pr = fit_predict(train_y.1);
    let both = (0..gold.0.len())
        .filter(|&i| pb[i] == gold.0[i] && pr[i] == gold.1[i])
        .count();
    Score {
        name,
        blankets: accuracy(&pb, gold.0),
        rope: accuracy(&pr, gold.1),
        both,
    }
}

fn majority(y: &[usize]) -> usize {
    usize::from(y.iter().filter(|&&v| v == 1).count() * 2 >= y.len())
}

fn main() {
    let train = load(TRAIN_PATH).expect("training data");
    let val = load(VAL_PATH).expect("validation data");
    let train_texts: Vec<&str> = train.iter().map(|r| r.text.as_str()).collect();
    let val_texts: Vec<&str> = val.iter().map(|r| r.text.as_str()).collect();

    let mut vectorizer = new_vectorizer();
    let x_train: Matrix<f32> = to_f32(&vectorizer.fit_transform(&train_texts).expect("fit"));
    let x_val: Matrix<f32> = to_f32(&vectorizer.transform(&val_texts).expect("transform"));
    let (tb, tr) = targets(&train);
    let (vb, vr) = targets(&val);
    let gold = (vb.as_slice(), vr.as_slice());
    let ty = (tb.as_slice(), tr.as_slice());
    println!(
        "same features for every candidate: {} rows x {} TF-IDF terms",
        x_train.shape().0,
        x_train.shape().1
    );

    let scores = [
        score("majority class", gold, |y| vec![majority(y); val.len()], ty),
        score(
            "naive Bayes",
            gold,
            |y| {
                let mut m = MultinomialNB::new();
                m.fit(&x_train, y).expect("fit");
                m.predict(&x_val)
            },
            ty,
        ),
        score(
            "logistic, unweighted",
            gold,
            |y| {
                let mut m = LogisticRegression::new()
                    .with_learning_rate(0.5)
                    .with_max_iter(5000)
                    .with_l2_penalty(0.001);
                m.fit(&x_train, y).expect("fit");
                m.predict(&x_val)
            },
            ty,
        ),
        score(
            "logistic, balanced",
            gold,
            |y| {
                let mut m = new_classifier();
                m.fit(&x_train, y).expect("fit");
                m.predict(&x_val)
            },
            ty,
        ),
    ];

    println!(
        "\n{:<22} {:>14} {:>10} {:>12}",
        "candidate", "Blankets Creek", "Rope Mill", "both right"
    );
    for s in &scores {
        println!(
            "{:<22} {:>14.3} {:>10.3} {:>9}/{}",
            s.name,
            s.blankets,
            s.rope,
            s.both,
            val.len()
        );
    }

    // Most rows with both answers right; on a tie, the one listed first.
    let chosen = scores.iter().skip(1).fold(
        &scores[1],
        |best, s| if s.both > best.both { s } else { best },
    );
    let base = &scores[0];
    println!(
        "\nchosen: {} ({} of {} rows with both trails right)",
        chosen.name,
        chosen.both,
        val.len()
    );

    assert!(
        chosen.blankets > base.blankets,
        "beat the baseline on Blankets Creek"
    );
    assert!(chosen.rope > base.rope, "beat the baseline on Rope Mill");
    println!(
        "contract: the choice is earned ({} beats majority class on both trails: {:.3} > {:.3}, {:.3} > {:.3}) OK",
        chosen.name, chosen.blankets, base.blankets, chosen.rope, base.rope
    );
}
