#!/bin/bash

set -euo pipefail

OUTFILE="${1:-/etc/rubybashrc}"

validate() {
  [ -d "$CARGO_HOME" ] || {
    echo "CARGO_HOME is not a dir"
    exit 1
  }
  [ -d "$RUSTUP_HOME" ] || {
    echo "RUSTUP_HOME is not a dir"
    exit 1
  }
  [ -d "$LIBCLANG_PATH" ] || {
    echo "LIBCLANG_PATH is not a dir"
    exit 1
  }
}

set_target_env_for_key_matching() {
  should_validate="$2"

  if env | grep "${1}"; then
    local var_name
    var_name="$(env | grep "${1}" | cut -d '=' -f 1)"
    local var_value
    var_value="$(env | grep "${1}" | cut -d '=' -f2-)"

    if [ "${should_validate}" = "true" ]; then
      command -v "${var_value}" || {
        echo "env var ${var_name} is not an executable"
        exit 1
      }
    fi

    echo "export ${var_name}=\"${var_value}\"" >> "$OUTFILE"
  fi
}

main() {
  set_target_env_for_key_matching "^BINDGEN_EXTRA_CLANG_ARGS_" false
  set_target_env_for_key_matching "^CC_" true
  set_target_env_for_key_matching "^CXX_" true
  set_target_env_for_key_matching "^AR_" true
  set_target_env_for_key_matching "^CMAKE_" true
  set_target_env_for_key_matching "^CARGO_TARGET_.*_LINKER" true

  # shellcheck disable=SC2129
  echo "export RUBY_CC_VERSION=\"3.2.0:3.1.0:3.0.0:2.7.0:2.6.0\"" >> "$OUTFILE"
  echo "export PATH=\"/usr/local/cargo/bin:\$PATH\"" >> "$OUTFILE"
  echo "export RUSTUP_HOME=\"$RUSTUP_HOME\"" >> "$OUTFILE"
  echo "export CARGO_HOME=\"$CARGO_HOME\"" >> "$OUTFILE"
  echo "export RUBY_TARGET=\"$RUBY_TARGET\"" >> "$OUTFILE"
  echo "export RCD_PLATFORM=\"$RUBY_TARGET\"" >> "$OUTFILE"
  echo "export RUST_TARGET=\"$RUST_TARGET\"" >> "$OUTFILE"
  echo "export RUSTUP_DEFAULT_TOOLCHAIN=\"$RUSTUP_DEFAULT_TOOLCHAIN\"" >> "$OUTFILE"
  echo "export PKG_CONFIG_ALLOW_CROSS=\"$PKG_CONFIG_ALLOW_CROSS\"" >> "$OUTFILE"
  echo "export LIBCLANG_PATH=\"$LIBCLANG_PATH\"" >> "$OUTFILE"
  echo "export CARGO_BUILD_TARGET=\"$RUST_TARGET\"" >> "$OUTFILE"
  echo "export CARGO=\"/usr/local/cargo/bin/cargo\"" >> "$OUTFILE"
  echo "export RB_SYS_CARGO_PROFILE=\"release\"" >> "$OUTFILE"

  # add musl path if exists
  if [ -d "/usr/local/musl/bin/" ]
  then
    # https://github.com/rust-lang/cargo/issues/10143
    # https://github.com/rust-lang/cargo/blob/master/src/cargo/core/compiler/build_context/target_info.rs#L612
    echo "export RUSTFLAGS=\"-C target-feature=-crt-static \$RUSTFLAGS\"" >> "$OUTFILE"
    echo "export PATH=\"/usr/local/musl/bin:\$PATH\"" >> "$OUTFILE"
  fi

  cat "$OUTFILE"
  validate

  rm "${0}"
}

main "${@}"
