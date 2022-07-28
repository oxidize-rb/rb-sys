#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  local td
  td="$(mktemp -d)"
  builtin pushd "${td}"

  git clone --single-branch --depth 1 https://github.com/rubygems/rubygems .
  ruby setup.rb

  builtin popd
  rm -rf "${td}"
  rm "${0}"
}

main "${@}"
