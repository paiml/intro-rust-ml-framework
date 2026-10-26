//! Lesson 2.3.3: a served model answers the same way every time.
//!
//! With temperature 0 the model always picks its most likely next token, so
//! a server that is working correctly must be repeatable.
//!
//! Provable contract: greedy determinism. Two identical requests return
//! identical text, and the same request streamed token by token joins up to
//! exactly that text.
//!
//! Usage: serve-check <first.json> <second.json> <stream.sse>

use model_checks::{join_stream, parse_completion};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, first, second, stream] = args.as_slice() else {
        eprintln!("usage: serve-check <first.json> <second.json> <stream.sse>");
        std::process::exit(2);
    };
    let read = |p: &String| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("read {p}: {e}"));
    let a = parse_completion(&read(first)).expect("first response");
    let b = parse_completion(&read(second)).expect("second response");
    let (streamed, chunks) = join_stream(&read(stream)).expect("stream");

    println!("first request   {:?}", a.content);
    println!("second request  {:?}", b.content);
    println!("streamed        {streamed:?}  ({chunks} chunks)");

    assert!(!a.content.is_empty(), "empty answer");
    assert_eq!(a.content, b.content, "same request, different answers");
    assert_eq!(
        streamed, a.content,
        "stream does not join up to the plain answer"
    );
    println!("contract: greedy determinism (same request -> same answer; stream joins to it) OK");
}
