#!/bin/sh
# Download Qwen2.5-0.5B-Instruct with apr and lay out the files the lessons use.
#
#   sh scripts/get-model.sh
#
# Run it from the repository root. It needs `apr` 0.71.4 on PATH and about
# 3 GB of disk. The download (~1 GB) is cached by apr, so running this again is
# quick. Everything it creates lives under models/, which git ignores.
set -eu

cd "$(dirname "$0")/.."
command -v apr >/dev/null || { echo "apr is not on PATH: install it first (see README)" >&2; exit 1; }

echo "==> apr pull hf://Qwen/Qwen2.5-0.5B-Instruct"
src=$(apr pull hf://Qwen/Qwen2.5-0.5B-Instruct | sed -n 's/^ *Path: //p' | head -n 1)
[ -f "$src" ] || { echo "apr pull did not report a model path" >&2; exit 1; }
base=${src%.safetensors}
for ext in config.json tokenizer.json tokenizer_config.json; do
    [ -f "$base.$ext" ] || { echo "missing $base.$ext next to the model" >&2; exit 1; }
done

mkdir -p models/qwen2.5-0.5b-instruct models/compile/hf

# Single files, as `apr run` and `apr inspect` read them.
ln -sf "$src"                 models/qwen2.5-0.5b-instruct.safetensors
ln -sf "$base.tokenizer.json" models/qwen2.5-0.5b-instruct.tokenizer.json
ln -sf "$base.config.json"    models/qwen2.5-0.5b-instruct.config.json

# A Hugging Face style folder, as `apr import` reads it (config next to weights).
for dir in models/qwen2.5-0.5b-instruct models/compile/hf; do
    ln -sf "$src"                        "$dir/model.safetensors"
    ln -sf "$base.config.json"           "$dir/config.json"
    ln -sf "$base.tokenizer.json"        "$dir/tokenizer.json"
    ln -sf "$base.tokenizer_config.json" "$dir/tokenizer_config.json"
done

# The .apr version `apr serve` uses in lesson 2.3.
if [ ! -f models/qwen2.5-0.5b-instruct.apr ]; then
    echo "==> apr import -> models/qwen2.5-0.5b-instruct.apr"
    apr import models/qwen2.5-0.5b-instruct/model.safetensors -o models/qwen2.5-0.5b-instruct.apr --offline >/dev/null
fi

echo "==> ready"
ls -lL models | sed 1d
