# trail-status data

A hand-labelled corpus of trail-status updates from the SORBA Woodstock
trail-status page. `trail-status.csv` holds the hand labels; `train.jsonl` and
`val.jsonl` were derived from it and from further raw status updates, with each
row's two open/closed labels folded into one 4-class label (`label_map.json`).

| file | rows | shape |
|---|---|---|
| `trail-status.csv` | 80 | `status,Blankets_Creek,Rope_Mill` (hand labels, 1 = open) |
| `train.jsonl` | 401 | `{"input": text, "label": 0..=3}` |
| `val.jsonl` | 70 | same |
| `label_map.json` | — | label index → `both_open`, `blankets_open_rope_closed`, `blankets_closed_rope_open`, `both_closed` |
| `provenance.csv` | — | where each JSONL row came from |

These files are inputs. Nothing in this repository rewrites them.
