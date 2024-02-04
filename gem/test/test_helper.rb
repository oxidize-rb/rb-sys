# frozen_string_literal: true

$LOAD_PATH.unshift File.expand_path("../lib", __dir__)
require "rb_sys"

require "pathname"
require "minitest/autorun"

ENV["RB_SYS_TEST"] = "1"

module TestHelpers
  def win_target?
    target_platform = RbConfig::CONFIG["target_os"]
    !!Gem::WIN_PATTERNS.find { |r| target_platform =~ r }
  end
end

Minitest::Test.include(TestHelpers)
