# frozen_string_literal: true

require "test_helper"
require "rb_sys/mkmf/config"
require "rb_sys/cargo_builder"

class TestRubygemsDetection < Minitest::Test
  def setup
    @original_env = ENV.to_h
    # Clear test-related environment variables
    ENV.delete("SOURCE_DATE_EPOCH")
    ENV.delete("NIX_STORE")
    ENV.delete("RB_SYS_TEST")
  end

  def teardown
    # Restore original environment by clearing and re-setting
    ENV.clear
    @original_env.each { |k, v| ENV[k] = v }
  end

  def test_rubygems_invoked_false_in_normal_context
    config = create_config
    refute config.rubygems_invoked?, "Should not detect rubygems in normal context"
  end

  def test_rubygems_invoked_true_with_source_date_epoch
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    config = create_config
    assert config.rubygems_invoked?, "Should detect rubygems when SOURCE_DATE_EPOCH is set"
  end

  def test_rubygems_invoked_false_with_rb_sys_test
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    ENV["RB_SYS_TEST"] = "1"
    config = create_config
    refute config.rubygems_invoked?, "Should not detect rubygems when RB_SYS_TEST=1"
  end

  def test_rubygems_invoked_false_in_nix_environment
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    ENV["NIX_STORE"] = "/nix/store"
    config = create_config
    refute config.rubygems_invoked?, "Should not detect rubygems in nix environment"
  end

  def test_rubygems_invoked_false_in_nix_with_empty_value
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    ENV["NIX_STORE"] = ""
    config = create_config
    refute config.rubygems_invoked?, "Should not detect rubygems when NIX_STORE is set (even if empty)"
  end

  def test_clean_after_install_follows_rubygems_detection
    # Test in normal context
    config = create_config
    refute config.clean_after_install, "Should not clean in development context"

    # Test with SOURCE_DATE_EPOCH
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    config = create_config
    assert config.clean_after_install, "Should clean when installed by rubygems"

    # Test in nix environment
    ENV["NIX_STORE"] = "/nix/store"
    config = create_config
    refute config.clean_after_install, "Should not clean in nix environment"
  end

  def test_cargo_builder_rubygems_detection
    # Test CargoBuilder's rubygems_invoked? method directly
    cargo_builder = create_cargo_builder_with_profile(:dev)

    # Normal context
    refute cargo_builder.rubygems_invoked?, "CargoBuilder should not detect rubygems in normal context"

    # With SOURCE_DATE_EPOCH
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    assert cargo_builder.rubygems_invoked?, "CargoBuilder should detect rubygems with SOURCE_DATE_EPOCH"

    # In nix environment
    ENV["NIX_STORE"] = "/nix/store"
    refute cargo_builder.rubygems_invoked?, "CargoBuilder should not detect rubygems in nix"
  end

  def test_profile_selection_based_on_rubygems_detection
    # Test that CargoBuilder uses release profile when rubygems invoked
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    cargo_builder = create_cargo_builder_with_profile(:dev)
    assert_equal :release, cargo_builder.profile, "Should use release profile when rubygems invoked"

    # Test that CargoBuilder respects configured profile in nix
    ENV.delete("SOURCE_DATE_EPOCH")
    ENV.delete("NIX_STORE")
    ENV["SOURCE_DATE_EPOCH"] = "315619200"
    ENV["NIX_STORE"] = "/nix/store"
    cargo_builder = create_cargo_builder_with_profile(:dev)
    assert_equal :dev, cargo_builder.profile, "Should respect configured profile in nix environment"
  end

  private

  def create_config
    # Create a mock builder
    builder = Object.new
    def builder.config=(config)
    end
    RbSys::Mkmf::Config.new(builder)
  end

  def create_cargo_builder_with_profile(profile)
    # Create a stub spec object
    spec = Object.new

    # Create a cargo builder instance and set its profile
    cargo_builder = RbSys::CargoBuilder.new(spec)
    cargo_builder.profile = profile
    cargo_builder
  end
end
