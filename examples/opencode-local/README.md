# opencode against a local `apr serve`

`opencode.json` in this directory is a project config. Run opencode from here and it
talks to the model served by `apr serve run models/qwen2.5-0.5b-instruct.apr --port 8089`
instead of any hosted provider. No account and no API key.

| key | why |
|---|---|
| `provider.apr` | an OpenAI-compatible provider whose `baseURL` is the local server |
| `model` / `small_model` | use that local model for answers and for session titles |
| `agent.ask` | a minimal agent: a one-line system prompt and **no tools**. opencode's default agent sends a large prompt plus tool definitions, which a 0.5B model on a CPU takes about a minute to read; `ask` answers in seconds |

```bash
cd examples/opencode-local
opencode run --agent ask "In one sentence, what is Rust?"
```

Demo 2.3.4 records exactly this inside a network namespace with no route out
(`unshare -rn`), so the capture can never download anything.
Remote features are switched off with `OPENCODE_DISABLE_MODELS_FETCH`,
`OPENCODE_DISABLE_DEFAULT_PLUGINS`, `OPENCODE_DISABLE_AUTOUPDATE`,
`OPENCODE_DISABLE_LSP_DOWNLOAD` and `OPENCODE_DISABLE_SHARE`.

opencode needs its `@opencode-ai/plugin` dependency already installed in its config
directory (`~/.config/opencode/node_modules`); with a fresh config directory it tries
to `npm install` it, which needs the network.
