//! Lesson 3.3.2: check a running trail-status API against its own documentation.
//!
//! Provable contract: the API keeps its word. Every route `GET /` documents
//! returns the status code the documentation promises, and a successful
//! prediction has all four documented fields.
//!
//! Usage: api-check [127.0.0.1:8080]   (the API must already be running)

use trail_status::bundle::Prediction;
use trail_status::http::request;

fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let good = r#"{"text": "all trails are open"}"#;
    let cases: [(&str, &str, Option<&str>, u16); 7] = [
        ("GET", "/", None, 200),
        ("POST", "/predict", Some(good), 200),
        ("POST", "/predict", Some("this is not json"), 400),
        (
            "POST",
            "/predict",
            Some(r#"{"words": "no text field"}"#),
            400,
        ),
        ("POST", "/predict", Some(r#"{"text": ""}"#), 400),
        ("GET", "/predict", None, 405),
        ("GET", "/status", None, 404),
    ];

    let mut kept = 0;
    for (method, path, body, want) in cases {
        let r = request(&addr, method, path, body)
            .unwrap_or_else(|e| panic!("{e} (is the API running?)"));
        let shown = body.map(|b| format!(" {b}")).unwrap_or_default();
        let ok = r.status == want;
        kept += usize::from(ok);
        println!(
            "{}  {} (documented {want})  {method} {path}{shown}",
            if ok { "ok  " } else { "FAIL" },
            r.status
        );
        if path == "/" {
            assert!(
                r.body.contains("POST /predict"),
                "the docs must describe POST /predict"
            );
        }
        if want == 200 && path == "/predict" {
            let p: Prediction =
                serde_json::from_str(&r.body).expect("a 200 must carry all four fields");
            println!("      the answer has all four documented fields");
            assert!(p.blankets_creek_confidence >= 0.5 && p.rope_mill_confidence >= 0.5);
        }
    }

    assert_eq!(
        kept,
        cases.len(),
        "every documented route must return its documented status"
    );
    println!(
        "contract: the API keeps its word ({kept}/{} routes return their documented status) OK",
        cases.len()
    );
}
