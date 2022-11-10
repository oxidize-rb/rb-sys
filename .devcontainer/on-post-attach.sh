#!/bin/bash

set -ex

echo "source /usr/local/share/chruby/chruby.sh" >> "$HOME/.zshrc"
echo "chruby 3.1" >> "$HOME/.zshrc"
echo "source /usr/local/share/chruby/chruby.sh" >> "$HOME/.bashrc"
echo "chruby 3.1" >> "$HOME/.bashrc"

rustup component add rustfmt
rustup component add clippy
