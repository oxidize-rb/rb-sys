#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  local td
  td="$(mktemp -d)"
  builtin pushd "${td}"

  local url="https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
  curl --retry 3 --proto '=https' --tlsv1.2 -sSf "$url" > rustup-init
  curl --retry 3 --proto '=https' --tlsv1.2 -sSf "$url.sha256" > rustup-init.sha256
  # Remove "target/x86_64-unknown-linux-gnu/release/" string from rustup-init.sha256
  sed -i 's:\*target/x86_64-unknown-linux-gnu/release/::' rustup-init.sha256
  sha256sum -c rustup-init.sha256
  chmod +x rustup-init
  ./rustup-init --no-modify-path --default-toolchain "$RUSTUP_DEFAULT_TOOLCHAIN" --profile minimal -y
  rustup target add "$RUST_TARGET"

  # Use git CLI to fetch crates (avoid memory issues in QEMU environments)
  printf "[net]\ngit-fetch-with-cli = true" >> "$CARGO_HOME/config.toml"

  chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME"

  echo "export RUSTUP_PERMIT_COPY_RENAME=\"true\"" >> "/etc/rubybashrc"

  builtin popd
  rm -rf "${td}"
  rm "${0}"
}

main "${@}"
