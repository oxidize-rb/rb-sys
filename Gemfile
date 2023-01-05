# frozen_string_literal: true

source "https://rubygems.org"

gemspec path: "gem"
gemspec path: "examples/rust_reverse"

gem "rake", "~> 13.0"
gem "minitest", "5.17.0"
gem "rake-compiler", "~> 1.2.0"

if RUBY_VERSION >= "2.6.0"
  gem "ruby-lsp", require: false
  gem "standard", "~> 1.12.1"
end
