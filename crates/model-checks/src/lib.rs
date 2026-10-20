//! Contract checks for the apr model demos.
//!
//! Every check is a pure function over text that `apr` or `apr serve`
//! produced, so each one is tested here against real output saved in
//! `fixtures/`, with no model and no server needed.

use serde_json::Value;

// ── 2.2.2: anatomy ────────────────────────────────────────────────────────

/// One tensor as `apr tensors --json` reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub name: String,
    pub shape: Vec<u64>,
    pub dtype: String,
    pub size_bytes: u64,
}

impl Tensor {
    /// Parameters in the tensor: the product of its shape.
    pub fn params(&self) -> u64 {
        self.shape.iter().product()
    }
}

/// What `apr inspect --json` claims about the whole model.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub total_params: u64,
    pub tensor_count: u64,
    pub architecture: String,
}

pub fn parse_inspect(json: &str) -> Result<Summary, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("inspect JSON: {e}"))?;
    let num = |k: &str| v[k].as_u64().ok_or_else(|| format!("inspect JSON: no numeric {k:?}"));
    Ok(Summary {
        total_params: num("total_params")?,
        tensor_count: num("tensor_count")?,
        architecture: v["architecture"].as_str().unwrap_or("unknown").to_string(),
    })
}

pub fn parse_tensors(json: &str) -> Result<Vec<Tensor>, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("tensors JSON: {e}"))?;
    v["tensors"]
        .as_array()
        .ok_or("tensors JSON: no \"tensors\" array")?
        .iter()
        .map(|t| {
            Ok(Tensor {
                name: t["name"].as_str().ok_or("tensor without a name")?.to_string(),
                shape: t["shape"]
                    .as_array()
                    .ok_or("tensor without a shape")?
                    .iter()
                    .map(|d| d.as_u64().ok_or("non-integer dimension"))
                    .collect::<Result<_, _>>()?,
                dtype: t["dtype"].as_str().unwrap_or("?").to_string(),
                size_bytes: t["size_bytes"].as_u64().ok_or("tensor without size_bytes")?,
            })
        })
        .collect::<Result<_, &str>>()
        .map_err(str::to_string)
}

/// Bytes per stored value for the dtypes this course meets.
pub fn bytes_per_value(dtype: &str) -> Option<u64> {
    match dtype {
        "F32" => Some(4),
        "F16" | "BF16" => Some(2),
        "I8" | "U8" => Some(1),
        _ => None,
    }
}

/// Group a tensor name into the part of the network it belongs to.
pub fn component(name: &str) -> &'static str {
    if name.contains("embed_tokens") {
        "embedding"
    } else if name.contains("self_attn") {
        "attention"
    } else if name.contains(".mlp.") {
        "mlp"
    } else if name.contains("norm") {
        "norm"
    } else if name.contains("lm_head") {
        "lm_head"
    } else {
        "other"
    }
}

/// Number of transformer layers: one more than the highest `layers.N`.
pub fn layer_count(tensors: &[Tensor]) -> u64 {
    tensors
        .iter()
        .filter_map(|t| t.name.split("layers.").nth(1)?.split('.').next()?.parse::<u64>().ok())
        .max()
        .map_or(0, |n| n + 1)
}

// ── 2.2.4: quantization ───────────────────────────────────────────────────

/// The answer `apr run` printed: the lines between `Output:` and `Completed in`.
pub fn parse_run_output(stdout: &str) -> Option<String> {
    let after = stdout.split("Output:\n").nth(1)?;
    let answer = after.split("\nCompleted in").next()?;
    Some(answer.trim().to_string())
}

/// One model in a size-versus-quality comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantRow {
    pub name: String,
    pub size_bytes: u64,
    pub answer: String,
}

/// The size-for-quality contract: every quantized model is smaller than the
/// baseline AND gives the baseline's answer to the same greedy prompt.
pub fn check_quantized(baseline: &QuantRow, quantized: &[QuantRow]) -> Result<(), String> {
    if quantized.is_empty() {
        return Err("no quantized models to compare".into());
    }
    for q in quantized {
        if q.size_bytes >= baseline.size_bytes {
            return Err(format!("{} is not smaller than {}", q.name, baseline.name));
        }
        if q.answer != baseline.answer {
            return Err(format!("{} answered {:?}, baseline answered {:?}", q.name, q.answer, baseline.answer));
        }
    }
    Ok(())
}

// ── 2.3.2 / 2.3.3: the OpenAI-style API ───────────────────────────────────

/// The parts of a chat completion a client actually reads.
#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    pub object: String,
    pub role: String,
    pub content: String,
    pub finish_reason: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

pub fn parse_completion(json: &str) -> Result<Completion, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("completion JSON: {e}"))?;
    let choice = &v["choices"][0];
    let s = |x: &Value, what: &str| x.as_str().map(str::to_string).ok_or(format!("missing {what}"));
    let n = |k: &str| v["usage"][k].as_u64().ok_or(format!("missing usage.{k}"));
    Ok(Completion {
        object: s(&v["object"], "object")?,
        role: s(&choice["message"]["role"], "choices[0].message.role")?,
        content: s(&choice["message"]["content"], "choices[0].message.content")?,
        finish_reason: s(&choice["finish_reason"], "choices[0].finish_reason")?,
        prompt_tokens: n("prompt_tokens")?,
        completion_tokens: n("completion_tokens")?,
        total_tokens: n("total_tokens")?,
    })
}

/// Join the `delta.content` pieces of a server-sent-events stream, in order.
/// Returns the text and the number of content chunks.
pub fn join_stream(sse: &str) -> Result<(String, usize), String> {
    let mut text = String::new();
    let mut chunks = 0;
    for line in sse.lines() {
        let Some(data) = line.strip_prefix("data: ") else { continue };
        if data.trim() == "[DONE]" {
            break;
        }
        let v: Value = serde_json::from_str(data).map_err(|e| format!("stream chunk: {e}"))?;
        if let Some(piece) = v["choices"][0]["delta"]["content"].as_str() {
            text.push_str(piece);
            chunks += 1;
        }
    }
    Ok((text, chunks))
}

// ── 2.3.4: who answered ───────────────────────────────────────────────────

/// Read one counter from a Prometheus text page, e.g. `realizar_requests_total`.
pub fn prometheus_value(page: &str, name: &str) -> Option<f64> {
    page.lines()
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once(' ')?;
            (k == name).then(|| v.trim().parse().ok())?
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSPECT: &str = include_str!("../fixtures/inspect.json");
    const TENSORS: &str = include_str!("../fixtures/tensors.json");

    #[test]
    fn real_qwen_parameters_add_up() {
        let s = parse_inspect(INSPECT).unwrap();
        let t = parse_tensors(TENSORS).unwrap();
        assert_eq!(s.total_params, 494_032_768);
        assert_eq!(t.len() as u64, s.tensor_count);
        assert_eq!(t.iter().map(Tensor::params).sum::<u64>(), s.total_params);
    }

    #[test]
    fn real_qwen_bytes_match_bf16() {
        for t in parse_tensors(TENSORS).unwrap() {
            assert_eq!(t.size_bytes, t.params() * bytes_per_value(&t.dtype).unwrap(), "{}", t.name);
        }
    }

    #[test]
    fn real_qwen_has_24_layers() {
        assert_eq!(layer_count(&parse_tensors(TENSORS).unwrap()), 24);
    }

    #[test]
    fn components_are_named() {
        assert_eq!(component("model.embed_tokens.weight"), "embedding");
        assert_eq!(component("model.layers.3.self_attn.q_proj.weight"), "attention");
        assert_eq!(component("model.layers.3.mlp.up_proj.weight"), "mlp");
        assert_eq!(component("model.layers.3.input_layernorm.weight"), "norm");
    }

    #[test]
    fn a_tensor_list_missing_a_tensor_does_not_add_up() {
        let s = parse_inspect(INSPECT).unwrap();
        let mut t = parse_tensors(TENSORS).unwrap();
        t.pop();
        assert_ne!(t.iter().map(Tensor::params).sum::<u64>(), s.total_params);
    }

    #[test]
    fn run_output_answer_is_extracted() {
        let out = include_str!("../fixtures/apr-run-paris.txt");
        assert_eq!(parse_run_output(out).as_deref(), Some("Paris"));
        assert_eq!(parse_run_output("no answer here"), None);
    }

    fn row(name: &str, size: u64, answer: &str) -> QuantRow {
        QuantRow { name: name.into(), size_bytes: size, answer: answer.into() }
    }

    #[test]
    fn smaller_and_same_answer_passes() {
        let base = row("f16", 1000, "Paris");
        assert!(check_quantized(&base, &[row("int8", 640, "Paris"), row("int4", 360, "Paris")]).is_ok());
    }

    #[test]
    fn a_changed_answer_fails() {
        let base = row("f16", 1000, "Paris");
        assert!(check_quantized(&base, &[row("int4", 360, "London")]).is_err());
    }

    #[test]
    fn a_file_that_did_not_shrink_fails() {
        let base = row("f16", 1000, "Paris");
        assert!(check_quantized(&base, &[row("q", 1000, "Paris")]).is_err());
    }

    #[test]
    fn nothing_to_compare_fails() {
        assert!(check_quantized(&row("f16", 1000, "Paris"), &[]).is_err());
    }

    #[test]
    fn completion_fields_are_read() {
        let c = parse_completion(include_str!("../fixtures/chat-paris.json")).unwrap();
        assert_eq!(c.object, "chat.completion");
        assert_eq!(c.role, "assistant");
        assert_eq!(c.content, "Paris");
        assert_eq!(c.total_tokens, c.prompt_tokens + c.completion_tokens);
    }

    #[test]
    fn a_completion_without_usage_is_refused() {
        assert!(parse_completion(r#"{"object":"chat.completion","choices":[{"message":{"role":"assistant","content":"x"},"finish_reason":"stop"}]}"#).is_err());
    }

    #[test]
    fn stream_pieces_join_in_order() {
        let sse = "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n\
                   data: {\"choices\":[{\"delta\":{\"content\":\"Red\"}}]}\n\n\
                   data: {\"choices\":[{\"delta\":{\"content\":\", Blue\"}}]}\n\n\
                   data: [DONE]\n";
        assert_eq!(join_stream(sse).unwrap(), ("Red, Blue".to_string(), 2));
    }

    #[test]
    fn prometheus_counters_are_read() {
        let page = "# HELP realizar_requests_total x\n# TYPE realizar_requests_total counter\n\
                    realizar_requests_total 4\nrealizar_requests_failed 0\n";
        assert_eq!(prometheus_value(page, "realizar_requests_total"), Some(4.0));
        assert_eq!(prometheus_value(page, "realizar_requests_failed"), Some(0.0));
        assert_eq!(prometheus_value(page, "missing"), None);
    }
}
