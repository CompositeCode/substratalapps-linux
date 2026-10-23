#!/usr/bin/env bash
# Run a command in the Linux dev container (build-aux/docker/Dockerfile.dev),
# with the cargo registry and target dir cached in Docker volumes.
#
#   scripts/dev.sh cargo test --workspace
#   scripts/dev.sh scripts/screenshot.sh
set -euo pipefail
cd "$(dirname "$0")/.."
image=substratal-hub-dev
if ! docker image inspect "$image" >/dev/null 2>&1; then
  docker build -t "$image" -f build-aux/docker/Dockerfile.dev build-aux/docker
fi
tty=()
[ -t 0 ] && tty=(-it)
exec docker run --rm ${tty[@]+"${tty[@]}"} \
  -v "$PWD":/src -w /src \
  -v substratal-cargo-registry:/usr/local/cargo/registry \
  -v substratal-target:/src/target \
  -e CARGO_TERM_COLOR=always \
  "$image" "$@"
