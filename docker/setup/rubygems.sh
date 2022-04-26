#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  local td
  td="$(mktemp -d)"
  builtin pushd "${td}"

  # Using my fork for now
  git clone --single-branch --branch cargo-builder-target --depth 1 https://github.com/ianks/rubygems .
  ruby setup.rb

  builtin popd
  rm -rf "${td}"
  rm "${0}"
}

main "${@}"
