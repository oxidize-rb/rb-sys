# frozen_string_literal: true

require "test_helper"

class TestRbSys < Minitest::Test
  def test_equality
    refute_equal RbSys::ToolchainInfo.new("aarch64-linux"), RbSys::ToolchainInfo.new("arm64-darwin")
    assert_equal RbSys::ToolchainInfo.new("arm64-darwin"), RbSys::ToolchainInfo.new("arm64-darwin")
  end

  def test_supported
    assert RbSys::ToolchainInfo.new("arm64-darwin").supported?
    refute RbSys::ToolchainInfo.new("x86-linux").supported?
  end

  def test_local
    skip("Skipping for mswin") if win_target?
    assert RbSys::ToolchainInfo.local.is_a?(RbSys::ToolchainInfo)
    assert_equal RbSys::ToolchainInfo.local.gem_platform, Gem::Platform.local
  end

  def test_to_s
    assert_equal "arm64-darwin", RbSys::ToolchainInfo.new("arm64-darwin-21").to_s
  end
end
