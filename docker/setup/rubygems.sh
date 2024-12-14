#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
  rubygems_version="3.5.23"
  bundler_version="2.5.23"

  gem update --no-document --system $rubygems_version
  gem install bundler:$bundler_version --no-document
  gem install rb_sys --no-document

  # prevent bundler from trampolining itself to a higher version
  echo "export BUNDLER_VERSION=\"$bundler_version\"" >> /etc/rubybashrc
}

main "${@}"
