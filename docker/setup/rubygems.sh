#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  gem update --system --no-document
}

main "${@}"
