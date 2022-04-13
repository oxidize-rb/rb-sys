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
    target_dir = makefile.to_s.gsub("/Makefile", "")

    assert_match(/cargo rustc --target-dir #{target_dir}/, makefile.read)
  end

  private

  def create_makefile
    require "mkmf"
    require "rb_sys/mkmf"
    cargo_dir = Dir.mktmpdir("rb_sys_test")

    create_rust_makefile("foo_ext", cargo_dir)
    Pathname.new(File.join(cargo_dir, "Makefile"))
  end
end
