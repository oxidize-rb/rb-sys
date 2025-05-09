#!/bin/bash

set -euo pipefail

# Script to delete unused files that cause issues or warnings
# Currently removes bundler gemspecs from older Ruby versions that have security vulnerabilities

# Find and remove bundler-1.17.2 gemspecs which have known vulnerabilities
find /usr/local/rake-compiler/ruby -path "*/ruby-*/lib/ruby/gems/*/specifications/default/bundler-1.17.2.gemspec" -delete

echo "Deleted unused gemspec files with vulnerabilities"