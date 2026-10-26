//! Trail-status CLI. The model is embedded in this binary (see `model.rs`), so
//! it runs with no `.apr` file on disk.
//!
//! Provable contract: one answer per trail. Every run prints exactly one
//! verdict for each trail, and each confidence is a probability in [0.5, 1].
//!
//! Usage:
//!   trail-status-cli "blankets creek is open. rope mill remains closed."
//!   trail-status-cli --json "<text>"     (machine-readable, used by api-parity)

use trail_status::model::TrailStatusModel;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let Some(text) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: trail-status-cli [--json] \"<trail status text>\"");
        std::process::exit(2);
    };

    let model = TrailStatusModel::load();
    let p = model.predict(text);

    let in_range = |c: f32| (0.5..=1.0).contains(&c);
    assert!(
        in_range(p.blankets_creek_confidence) && in_range(p.rope_mill_confidence),
        "confidence out of range: {p:?}"
    );

    if json {
        println!("{}", serde_json::to_string(&p).expect("serialize"));
        return;
    }
    let verdict = |open: bool| if open { "OPEN" } else { "CLOSED" };
    println!("input: {text:?}");
    println!(
        "  Blankets Creek: {:<6} ({:.1}% confidence)",
        verdict(p.blankets_creek_open),
        p.blankets_creek_confidence * 100.0
    );
    println!(
        "  Rope Mill:      {:<6} ({:.1}% confidence)",
        verdict(p.rope_mill_open),
        p.rope_mill_confidence * 100.0
    );
}
