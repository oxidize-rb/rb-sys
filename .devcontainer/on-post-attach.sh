#!/bin/bash

set -ex

# shellcheck disable=SC1091
[[ -f /etc/rubybashrc ]] && source /etc/rubybashrc

bundle install --jobs 3
cargo update --dry-run
rustup component add rustfmt
rustup component add clippy
