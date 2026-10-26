//! Trail-status HTTP API. The model is embedded in this binary (see `model.rs`).
//!
//! A small single-threaded server:
//!
//! | route | answer |
//! |---|---|
//! | `GET /` | 200, this API's documentation as JSON |
//! | `POST /predict` `{"text": "..."}` | 200, both trails with confidences |
//! | `POST /predict` with bad JSON, no `text`, or empty text | 400, `{"error": ...}` |
//! | `GET /predict` (any method but POST) | 405, `{"error": ...}` |
//! | anything else | 404, `{"error": ...}` |
//!
//! Provable contract: load once, answer every request. The model is loaded
//! one time, before the first request is accepted, and every request gets a
//! JSON answer with one of the documented status codes.
//!
//! Usage: trail-status-api [--host 127.0.0.1] [--port 8080]

use serde::Deserialize;
use tiny_http::{Header, Method, Request, Response, Server};
use trail_status::model::TrailStatusModel;

/// Every answer: a JSON body in memory.
type Reply = Response<std::io::Cursor<Vec<u8>>>;

#[derive(Deserialize)]
struct PredictRequest {
    text: String,
}

/// The documentation `GET /` returns.
fn docs(addr: &str) -> String {
    serde_json::json!({
        "service": "trail-status-api",
        "description": "Is Blankets Creek open? Is Rope Mill open? Send a trail status update, get both answers.",
        "routes": {
            "GET /": "200: this documentation",
            "POST /predict": {
                "request": {"text": "string, required, non-empty"},
                "200": {"blankets_creek_open": "bool", "blankets_creek_confidence": "number 0.5..1",
                        "rope_mill_open": "bool", "rope_mill_confidence": "number 0.5..1"},
                "400": "body is not JSON, has no text, or text is empty",
                "405": "any method other than POST"
            },
            "anything else": "404"
        },
        "example": format!("curl -s -X POST http://{addr}/predict -d '{{\"text\": \"blankets creek is open. rope mill remains closed.\"}}'")
    })
    .to_string()
}

fn json(status: u16, body: String) -> Reply {
    let header =
        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).expect("static header");
    // A final newline, so a terminal prompt starts on its own line after `curl`.
    Response::from_string(body + "\n")
        .with_status_code(status)
        .with_header(header)
}

fn error(status: u16, message: &str) -> Reply {
    json(status, serde_json::json!({ "error": message }).to_string())
}

/// Route one request. Pure: no I/O but reading the body.
fn handle(model: &TrailStatusModel, addr: &str, request: &mut Request) -> Reply {
    match (request.method(), request.url()) {
        (Method::Get, "/") => json(200, docs(addr)),
        (Method::Post, "/predict") => {
            let mut body = String::new();
            if request.as_reader().read_to_string(&mut body).is_err() {
                return error(400, "could not read the request body");
            }
            match serde_json::from_str::<PredictRequest>(&body) {
                Ok(r) if r.text.trim().is_empty() => error(400, "text is empty"),
                Ok(r) => json(
                    200,
                    serde_json::to_string(&model.predict(&r.text)).expect("serialize"),
                ),
                Err(e) => error(400, &format!("invalid request: {e}")),
            }
        }
        (_, "/predict") => error(405, "use POST /predict"),
        _ => error(404, "not found: GET / for docs, POST /predict to predict"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str, default: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| default.to_string())
    };
    let addr = format!("{}:{}", flag("--host", "127.0.0.1"), flag("--port", "8080"));

    // Step 1: load the model ONCE, before accepting anything.
    let model = TrailStatusModel::load();
    // Step 2: listen on the address.
    let server = Server::http(&addr).unwrap_or_else(|e| panic!("bind {addr}: {e}"));
    println!("model loaded once; listening on http://{addr}  (GET / for docs, POST /predict)");

    // Step 3: route each request, and answer with a documented status only.
    for mut request in server.incoming_requests() {
        let response = handle(&model, &addr, &mut request);
        let status = response.status_code().0;
        assert!(
            [200, 400, 404, 405].contains(&status),
            "undocumented status {status}"
        );
        println!("{} {} -> {status}", request.method(), request.url());
        let _ = request.respond(response);
    }
}
