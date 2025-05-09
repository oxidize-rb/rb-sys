#!/bin/bash

set -euo pipefail

# Script to delete unused files that cause issues or warnings
# Currently removes gemspecs from older Ruby versions that have security vulnerabilities

# Base path where ruby installations are located
ROOT_PATH="/usr/local/rake-compiler/ruby"

echo "Removing vulnerable gemspec files from $ROOT_PATH"

# Ruby 2.4.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-2.4*/lib/ruby/gems/2.4.0/specifications/default/rdoc-5.0.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.4*/lib/ruby/gems/2.4.0/specifications/rake-12.0.0.gemspec" -delete

# Ruby 2.5.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-2.5*/lib/ruby/gems/2.5.0/specifications/default/rdoc-6.0.1.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.5*/lib/ruby/gems/2.5.0/specifications/default/webrick-1.4.2.1.gemspec" -delete

# Ruby 2.6.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-2.6*/lib/ruby/gems/2.6.0/specifications/default/bundler-1.17.2.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.6*/lib/ruby/gems/2.6.0/specifications/default/rdoc-6.1.2.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.6*/lib/ruby/gems/2.6.0/specifications/default/rexml-3.1.9.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.6*/lib/ruby/gems/2.6.0/specifications/default/webrick-1.4.4.gemspec" -delete

# Ruby 2.7.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-2.7*/lib/ruby/gems/2.7.0/specifications/default/bundler-2.1.4.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.7*/lib/ruby/gems/2.7.0/specifications/default/cgi-0.1.0.2.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.7*/lib/ruby/gems/2.7.0/specifications/default/rdoc-6.2.1.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.7*/lib/ruby/gems/2.7.0/specifications/default/rexml-3.2.3.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.7*/lib/ruby/gems/2.7.0/specifications/default/uri-0.10.0.2.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-2.7*/lib/ruby/gems/2.7.0/specifications/default/webrick-1.6.1.gemspec" -delete

# Ruby 3.0.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-3.0*/lib/ruby/gems/3.0.0/specifications/default/cgi-0.2.2.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.0*/lib/ruby/gems/3.0.0/specifications/default/net-imap-0.1.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.0*/lib/ruby/gems/3.0.0/specifications/default/uri-0.10.3.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.0*/lib/ruby/gems/3.0.0/specifications/rexml-3.2.5.gemspec" -delete

# Ruby 3.1.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-3.1*/lib/ruby/gems/3.1.0/specifications/default/cgi-0.3.6.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.1*/lib/ruby/gems/3.1.0/specifications/default/uri-0.12.2.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.1*/lib/ruby/gems/3.1.0/specifications/net-imap-0.2.4.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.1*/lib/ruby/gems/3.1.0/specifications/rexml-3.2.5.gemspec" -delete

# Ruby 3.2.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-3.2*/lib/ruby/gems/3.2.0/specifications/default/cgi-0.3.6.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.2*/lib/ruby/gems/3.2.0/specifications/default/uri-0.12.3.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.2*/lib/ruby/gems/3.2.0/specifications/net-imap-0.3.4.1.gemspec" -delete

# Ruby 3.3.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-3.3*/lib/ruby/gems/3.3.0/specifications/default/cgi-0.4.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.3*/lib/ruby/gems/3.3.0/specifications/default/uri-0.13.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.3*/lib/ruby/gems/3.3.0/specifications/net-imap-0.4.9.1.gemspec" -delete

# Ruby 3.4.x vulnerabilities
find "$ROOT_PATH" -path "*/ruby-3.4*/lib/ruby/gems/3.4.0/specifications/default/cgi-0.4.1.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.4*/lib/ruby/gems/3.4.0/specifications/default/uri-1.0.2.gemspec" -delete
find "$ROOT_PATH" -path "*/ruby-3.4*/lib/ruby/gems/3.4.0/specifications/net-imap-0.5.4.gemspec" -delete

echo "Deleted vulnerable gemspec files"
