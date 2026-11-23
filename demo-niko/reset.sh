#!/bin/sh
# Deletes the demo folder: the sandbox home, the demo binaries and the work copies.
set -e
real="${SYMPOSIUM_DEMO_REAL_HOME:-$HOME}"
demo="${DEMO:-$real/symposium-demo}"
case "$demo" in
  */symposium-demo) rm -rf "$demo"; echo "removed $demo" ;;
  *) echo "refusing to remove $demo (expected a folder named symposium-demo)" >&2; exit 1 ;;
esac
