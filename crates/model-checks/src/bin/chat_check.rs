//! Lesson 2.3.2: read an OpenAI-style chat completion the way a client does.
//!
//! Provable contract: the response has the shape every OpenAI client relies
//! on. It is a `chat.completion`; `choices[0].message` comes back with role
//! `assistant` and non-empty content; `finish_reason` says why it stopped; and
//! the token accounting is consistent: total = prompt + completion.
//!
//! Usage: chat-check <response.json>

use model_checks::parse_completion;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: chat-check <response.json>");
        std::process::exit(2);
    });
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let c = parse_completion(&json).unwrap_or_else(|e| panic!("{path}: {e}"));

    println!("object                      {}", c.object);
    println!("choices[0].message.role     {}", c.role);
    println!("choices[0].message.content  {:?}", c.content);
    println!("choices[0].finish_reason    {}", c.finish_reason);
    println!(
        "usage                       {} prompt + {} completion = {} total tokens",
        c.prompt_tokens, c.completion_tokens, c.total_tokens
    );

    assert_eq!(c.object, "chat.completion");
    assert_eq!(c.role, "assistant");
    assert!(!c.content.trim().is_empty(), "the assistant said nothing");
    assert!(matches!(c.finish_reason.as_str(), "stop" | "length"), "unexpected finish_reason");
    assert_eq!(c.total_tokens, c.prompt_tokens + c.completion_tokens, "token accounting");
    println!("contract: OpenAI response shape (assistant message, finish_reason, total = prompt + completion) OK");
}
