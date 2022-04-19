# frozen_string_literal: true

require_relative "rust_reverse/version"

begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require "rust_reverse/#{$1}/rust_reverse"
rescue LoadError
  require "rust_reverse/rust_reverse"
end

module RustReverse
  class Error < StandardError; end
  # Your code goes here...
end
