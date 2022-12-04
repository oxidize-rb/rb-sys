#!/usr/bin/env bash

set -e

is_debug_info_enabled() {
  if [[ "${DEBUG:-false}" == "true" ]]; then
    true
  elif [[ "${CI:-false}" == "true" ]]; then
    false
  else
    true
  fi
}

set_env_var() {
  if [[ -n "$GITHUB_ENV" ]]; then
    echo "${1}=${2}" >> "$GITHUB_ENV"
  else
    echo "export ${1}=${2}"
  fi
}

main() {
  mkdir -p "${DEBUG_OUTPUT_DIR:-/tmp/debug}"

  if is_debug_info_enabled; then
    set_env_var "RUST_BACKTRACE" "1"
    set_env_var "RUST_LOG" "debug"
    set_env_var "RB_SYS_VERBOSE" "true"
    set_env_var "CARGO_TERM_VERBOSE" "true"
    set_env_var "RUBYOPT" "-w"
    set_env_var "V" "1"
  fi
}

main
