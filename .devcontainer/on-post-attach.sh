#!/bin/bash

set -ex

source /etc/rubybashrc
bundle install
cargo update --dry-run