# OpenAI-style request bodies

Used by demos 2.3.2 and 2.3.3 against `apr serve run models/qwen2.5-0.5b-instruct.apr --port 8089`.

| file | sent to | what it shows |
|---|---|---|
| `chat-request.json` | `POST /v1/chat/completions` | a system + user message, greedy (`temperature: 0`) |
| `chat-request-stream.json` | `POST /v1/chat/completions` | the same request with `"stream": true` (server-sent events) |

Greedy decoding makes the answer repeatable, which is what `model-checks`'
`serve-check` contract relies on.
