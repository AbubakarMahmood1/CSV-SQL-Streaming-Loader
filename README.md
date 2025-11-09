# CSV-SQL-Streaming-Loader
Rust CLI “CSV→SQL Streaming Loader” (batched COPY, schema inference, progress UI)

Why it matters: Rust remains the most‑admired language; a streaming loader that handles multi‑GB files shows performance & reliability without resorting to heavy infra. 
Stack Overflow

What you’ll ship:

CLI that infers types, streams rows, retries, and emits metrics

Unit tests + property tests; release binaries
Roadmap:

Parser + schema guess → 2) Postgres COPY w/ batches → 3) progress + retries → 4) tests + perf notes.
