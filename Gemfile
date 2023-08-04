# frozen_string_literal: true

source "https://rubygems.org"

gemspec path: "gem"
gemspec path: "examples/rust_reverse"

gem "rake", "~> 13.0"
gem "minitest", "5.15.0"
gem "rake-compiler", "~> 1.2.5" # Small bug in 1.2.4 that breaks Ruby 2.5
gem "yard"

if RUBY_VERSION >= "2.6.0"
  gem "standard", "~> 1.12.1"
end
