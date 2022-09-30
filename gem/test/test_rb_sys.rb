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
    skip("Skipping for mswin") if win_target?

    makefile = create_makefile do |b|
      b.env = {"NO_LINK_RUTIE" => "true"}
    end

    assert_match(/export NO_LINK_RUTIE := true/, makefile.read)
  end

  def test_uses_custom_profile
    makefile = create_makefile do |b|
      b.profile = :dev
    end

    assert(makefile.read.include?("RB_SYS_CARGO_PROFILE ?= dev"), "Expected to find RB_SYS_CARGO_PROFILE ?= dev") unless win_target?
    assert_match(/--profile \$\(RB_SYS_CARGO_PROFILE\)/, makefile.read)
  end

  def test_uses_extra_features
    makefile = create_makefile do |b|
      b.features = ["foo", "bar"]
    end

    assert(makefile.read.include?("RB_SYS_CARGO_FEATURES ?= foo,bar"), "Expected to find RB_SYS_CARGO_PROFILE ?= foo,bar") unless win_target?
    assert_match(/--features \$\(RB_SYS_CARGO_FEATURES\)/, makefile.read)
  end

  def test_uses_extra_rustc_args
    makefile = create_makefile do |b|
      b.extra_rustc_args = ["-C", "debuginfo=42"]
    end

    assert_match(/-C debuginfo=42$/, makefile.read)
  end

  def test_uses_extra_rustflags
    skip("Skipping for mswin") if win_target?

    makefile = create_makefile do |b|
      b.extra_rustflags = ["--cfg=foo"]
    end

    content = makefile.read

    assert content.include?("export RUSTFLAGS := $(RUSTFLAGS) $(RB_SYS_EXTRA_RUSTFLAGS)")
    assert content.include?("RB_SYS_EXTRA_RUSTFLAGS ?= --cfg=foo")
  end

  def test_uses_custom_target
    makefile = create_makefile do |b|
      b.target = "wasm32-unknown-unknown"
    end

    assert_match(/CARGO_BUILD_TARGET \?= wasm32-unknown-unknown/, makefile.read) unless win_target?
    assert_match(/--target \$\(CARGO_BUILD_TARGET\)/, makefile.read)
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

  def win_target?
    target_platform = RbConfig::CONFIG["target_os"]
    !!Gem::WIN_PATTERNS.find { |r| target_platform =~ r }
  end
end
