#!/usr/bin/env bash

# Exec given command and copy debug info to a file if it fails
#
# Example:
#   $ run-and-upload-on-failure.sh "cargo test --all-features" "cargo-test" "target/debug"
main() {
  if "$1"; then
    if [[ "${DEBUG:-false}" == "true" ]]; then
      upload_dir "$2" "$3"
    fi

    exit 0
  else
    exit_code=$?
    upload_dir "$2" "$3"
    exit $exit_code
  fi
}


upload_dir() {
  output_dir="${DEBUG_OUTPUT_DIR:-/tmp/debug}/$1"
  output_file="${output_dir}.tar"

  mkdir -p "$output_dir"
  tar -cf "$output_file" -C "$2" .
  echo "[INFO] Debug output saved to $output_file" >&2
}

main "$@"
