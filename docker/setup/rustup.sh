#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  local td
  td="$(mktemp -d)"
  builtin pushd "${td}"

  local url="https://static.rust-lang.org/rustup/dist/$1/rustup-init"
  curl --retry 3 --proto '=https' --tlsv1.2 -sSf "$url" > rustup-init
  chmod +x rustup-init
  ./rustup-init --no-modify-path --default-toolchain "$3" --profile minimal -y || true # i686 has an ignorable error
  chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME"
  rustup target add "$2"

  builtin popd
  rm -rf "${td}"
  rm "${0}"
}

main "${@}"
