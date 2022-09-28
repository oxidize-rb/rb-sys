#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  local version
  version="3.3.22"
  gem update --system "${version}" --no-document
  set +u
  rvm rubygems "${version}"
  set -u
}

main "${@}"
