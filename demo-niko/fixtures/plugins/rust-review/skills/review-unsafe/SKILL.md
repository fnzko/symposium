---
name: review-unsafe
description: Checklist for reviewing unsafe Rust blocks. Use when the user asks to review code that contains `unsafe`.
---

# Reviewing unsafe Rust

1. Every `unsafe` block has a `// SAFETY:` comment naming the invariant it relies on.
2. The invariant is upheld by every caller, not just the current one.
3. Prefer a safe abstraction with a small unsafe core over many scattered blocks.
