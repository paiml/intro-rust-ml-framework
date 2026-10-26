//! Lesson 2.3.4: prove another application's answer came from our server.
//!
//! `apr serve` counts every request it handles on its `/metrics` page. Read
//! the page before and after another program (opencode) asks its question.
//!
//! Provable contract: answered locally. The server's request counter went up
//! while the other program ran, and none of those requests failed.
//!
//! Usage: metrics-delta <before.prom> <after.prom>

use model_checks::prometheus_value;

const REQUESTS: &str = "realizar_requests_total";
const FAILED: &str = "realizar_requests_failed";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, before_path, after_path] = args.as_slice() else {
        eprintln!("usage: metrics-delta <before.prom> <after.prom>");
        std::process::exit(2);
    };
    let read = |p: &String| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("read {p}: {e}"));
    let (before, after) = (read(before_path), read(after_path));
    let get = |page: &str, k: &str| {
        prometheus_value(page, k).unwrap_or_else(|| panic!("no {k} on the metrics page"))
    };

    let handled = get(&after, REQUESTS) - get(&before, REQUESTS);
    let failed = get(&after, FAILED) - get(&before, FAILED);
    println!(
        "{REQUESTS}: {} -> {}",
        get(&before, REQUESTS),
        get(&after, REQUESTS)
    );
    println!("requests served for the other app: {handled}, failed: {failed}");

    assert!(handled >= 1.0, "the local server handled no requests");
    assert_eq!(failed, 0.0, "a request failed");
    println!("contract: answered locally ({handled} request(s) served by apr, 0 failed) OK");
}
