---
name: retry-policies
description: How to retry fallible operations with acme-retry. Use when adding retries, backoff, or idempotency checks around network or IO calls.
---

# Retrying with acme-retry

- Wrap the fallible call in `acme_retry::with_retries(n, || ...)`.
- Only retry idempotent operations.
- Keep `n` small (3 is the team default) and log the final error with `anyhow::Context`.
