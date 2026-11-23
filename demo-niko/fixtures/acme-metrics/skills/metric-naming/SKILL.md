---
name: metric-naming
description: Naming rules for acme-metrics counters and gauges. Use when adding or renaming a metric.
---

# Metric naming

- Names are `snake_case`, prefixed by the subsystem: `http_requests_total`.
- Counters end in `_total`; durations end in `_seconds`.
