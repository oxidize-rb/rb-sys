#!/usr/bin/env bash

# Exec given command and copy debug info to a file if it fails
#
# Example:
#   $ run-and-upload-on-failure.sh "cargo test --all-features" "cargo-test" "target/debug"
main() {
  if "$1"; then
    exit 0
  else
    exit_code=$?
    output_dir="${DEBUG_OUTPUT_DIR:-/tmp/debug}/$2"
    output_file="${output_dir}.tar"

    mkdir -p "$output_dir"
    tar -cf "$output_file" -C "$3" .
    echo "[ERROR] Command failed, debug output saved to $output_file" >&2
    exit $exit_code
  fi
}

main "$@"
