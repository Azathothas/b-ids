# Published formats

Generated. Do not edit: this table is derived from the generator, so a format that is added, changed or declined moves this file in the same change.

## Published

| ask for | file | carries |
| --- | --- | --- |
| `json` | `corpus.json` | lossless |
| `ndjson` | `corpus.ndjson` | lossless |
| `yaml` | `corpus.yaml` | lossless |
| `toml` | `corpus.toml` | every field that is not null |
| `sqlite` | `corpus.sql` | lossless |
| `csv` | `corpus.csv` | the eight flat columns |
| `tsv` | `corpus.tsv` | the eight flat columns |
| `md` | `corpus.md` | the eight flat columns |
| `protobuf` | `corpus.proto` | a definition, not values |

## Declined

Weighed and not published. A consumer who wants one of these can read why rather than guess.

| asked for | why not |
| --- | --- |
| `cbor` | a binary codec is a dependency here and a decoder this project owns forever, and it serves the same consumer MessagePack would. Nothing consumes either today. The lossless artefact for a program is the SQLite dump, which needs no library at all. |
| `msgpack` | the same trade as CBOR, and publishing both would be two binary encodings of one model with nothing choosing between them. |
