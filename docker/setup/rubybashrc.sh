#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

set_target_env_for_key_matching() {
  local var_name
  var_name="$(env | grep "${1}" | cut -d '=' -f 1)"
  local var_value
  var_value="$(env | grep "${1}" | cut -d '=' -f2-)"
  echo "export ${var_name}=\"${var_value}\"" >> /etc/rubybashrc
}

main() {
  # shellcheck disable=SC2129
  echo "export PATH=\"/usr/local/cargo/bin:\$PATH\"" >> /etc/rubybashrc
  echo "export RUSTUP_HOME=\"$RUSTUP_HOME\"" >> /etc/rubybashrc
  echo "export CARGO_HOME=\"$CARGO_HOME\"" >> /etc/rubybashrc
  echo "export RUBY_TARGET=\"$RUBY_TARGET\"" >> /etc/rubybashrc
  echo "export RUST_TARGET=\"$RUST_TARGET\"" >> /etc/rubybashrc
  echo "export RUSTUP_DEFAULT_TOOLCHAIN=\"$RUSTUP_DEFAULT_TOOLCHAIN\"" >> /etc/rubybashrc
  echo "export PKG_CONFIG_ALLOW_CROSS=\"$PKG_CONFIG_ALLOW_CROSS\"" >> /etc/rubybashrc
  echo "export LIBCLANG_PATH=\"$LIBCLANG_PATH\"" >> /etc/rubybashrc
  echo "export CARGO_BUILD_TARGET=\"$RUST_TARGET\"" >> /etc/rubybashrc
  echo "export CARGO=\"/usr/local/cargo/bin/cargo\"" >> /etc/rubybashrc
  echo "export RB_SYS_CARGO_PROFILE=\"release\"" >> /etc/rubybashrc

  set_target_env_for_key_matching "^BINDGEN_EXTRA_CLANG_ARGS_"
  set_target_env_for_key_matching "^CC_"
  set_target_env_for_key_matching "^CXX_"
  set_target_env_for_key_matching "^AR_"
  set_target_env_for_key_matching "^CARGO_TARGET_.*_LINKER"

  rm "${0}"
}

main "${@}"
