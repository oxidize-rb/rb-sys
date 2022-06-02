# frozen_string_literal: true

require "test_helper"

class TestRbSys < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::RbSys::VERSION
  end

  def test_includes_rbconfig_env
    makefile = create_makefile

    assert_match(/RBCONFIG_/, makefile.read)
  end

  def test_invokes_cargo_rustc
    makefile = create_makefile

    assert_match(/\$\(CARGO\) rustc --target-dir/, makefile.read)
  end

  def test_invokes_custom_env
    makefile = create_makefile do |b|
      b.env = {"NO_LINK_RUTIE" => "true"}
    end

    assert_match(/\$\(DLLIB\): export NO_LINK_RUTIE = true/, makefile.read)
  end

  def test_uses_custom_profile
    makefile = create_makefile do |b|
      b.profile = :dev
    end

    assert(makefile.read.include?("RB_SYS_CARGO_PROFILE ?= dev"), "Expected to find RB_SYS_CARGO_PROFILE ?= dev")
    assert_match(/--profile \$\(RB_SYS_CARGO_PROFILE\)/, makefile.read)
  end

  def test_uses_extra_features
    makefile = create_makefile do |b|
      b.features = ["foo", "bar"]
    end

    assert(makefile.read.include?("RB_SYS_CARGO_FEATURES ?= foo,bar"), "Expected to find RB_SYS_CARGO_PROFILE ?= foo,bar")
    assert_match(/--features \$\(RB_SYS_CARGO_FEATURES\)/, makefile.read)
  end

  def test_uses_extra_rustc_args
    makefile = create_makefile do |b|
      b.extra_rustc_args = ["-C", "debuginfo=42"]
    end

    assert_match(/-C debuginfo=42$/, makefile.read)
  end

  def test_uses_custom_target
    makefile = create_makefile do |b|
      b.target = "wasm32-unknown-unknown"
    end

    assert_match(/--target wasm32-unknown-unknown/, makefile.read)
  end

  def test_generates_deffile
    makefile = create_makefile.read

    assert makefile.include?("DEFFILE = $(TARGET_DIR)/$(TARGET)-$(arch).def")
    assert makefile.include?("$(DEFFILE):")
  end

  private

  def create_makefile(&blk)
    require "mkmf"
    require "rb_sys/mkmf"
    cargo_dir = Dir.mktmpdir("rb_sys_test")

    Dir.chdir(cargo_dir) do
      create_rust_makefile("foo_ext", &blk)
      Pathname.new(File.join(cargo_dir, "Makefile"))
    end
  end
end
