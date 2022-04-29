#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  # shellcheck disable=SC2129
  echo "export PATH=\"/usr/local/cargo/bin:\$PATH\"" >> /etc/rubybashrc
  echo "export RUSTUP_HOME=\"$RUSTUP_HOME\"" >> /etc/rubybashrc
  echo "export CARGO_HOME=\"$CARGO_HOME\"" >> /etc/rubybashrc
  echo "export RUBY_TARGET=\"$RUBY_TARGET\"" >> /etc/rubybashrc
  echo "export RUST_TARGET=\"$RUST_TARGET\"" >> /etc/rubybashrc
  echo "export RUSTUP_DEFAULT_TOOLCHAIN=\"$RUSTUP_DEFAULT_TOOLCHAIN\"" >> /etc/rubybashrc
  echo "export BINDGEN_EXTRA_CLANG_ARGS=\"$BINDGEN_EXTRA_CLANG_ARGS\"" >> /etc/rubybashrc
  echo "export PKG_CONFIG_ALLOW_CROSS=\"$PKG_CONFIG_ALLOW_CROSS\"" >> /etc/rubybashrc
  echo "export LIBCLANG_PATH=\"$LIBCLANG_PATH\"" >> /etc/rubybashrc
  echo "export CARGO_BUILD_TARGET=\"$RUST_TARGET\"" >> /etc/rubybashrc
  echo "export CARGO=\"/usr/local/cargo/bin/cargo\"" >> /etc/rubybashrc

  rm "${0}"
}

main "${@}"
