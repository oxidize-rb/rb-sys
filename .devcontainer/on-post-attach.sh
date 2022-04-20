#!/bin/bash

set -ex

source /etc/rubybashrc
bundle install
cargo update --dry-run
rustup component add rustfmt
rustup component add clippy