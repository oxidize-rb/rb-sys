#!/bin/bash

set -ex

# shellcheck disable=SC1091
[[ -f /etc/rubybashrc ]] && source /etc/rubybashrc
[[ -f /usr/local/share/chruby/chruby.sh ]] && source /usr/local/share/chruby/chruby.sh

bundle install --jobs 3
cargo update --dry-run
rustup component add rustfmt
rustup component add clippy
