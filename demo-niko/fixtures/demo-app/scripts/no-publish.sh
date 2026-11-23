#!/bin/sh
input=$(cat)
if printf '%s' "$input" | grep -q 'cargo publish' && ! printf '%s' "$input" | grep -q -- '--dry-run'; then
  echo "demo-app team rule: never run cargo publish from an agent session; use cargo publish --dry-run" >&2
  exit 2
fi
exit 0
