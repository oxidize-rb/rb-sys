#!/bin/bash

set -ex

[[ -f /etc/rubybashrc ]] && source /etc/rubybashrc
bundle install
cargo update --dry-run
rustup component add rustfmt
rustup component add clippy