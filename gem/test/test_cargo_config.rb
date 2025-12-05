# frozen_string_literal: true

require "test_helper"
require "minitest/mock"
require "fileutils"
require "rb_sys/cargo/config"

class TestCargoConfig < Minitest::Test
  def test_parse_section_style
    assert_equal "aarch64-apple-darwin", parse("[build]\ntarget = \"aarch64-apple-darwin\"")
  end

  def test_parse_dotted_key_style
    assert_equal "x86_64-unknown-linux-gnu", parse("build.target = \"x86_64-unknown-linux-gnu\"")
  end

  def test_parse_with_comment
    assert_equal "aarch64-apple-darwin", parse("[build]\ntarget = \"aarch64-apple-darwin\" # M1")
  end

  def test_parse_ignores_wrong_section
    assert_nil parse("[target.x86_64-unknown-linux-gnu]\ntarget = \"wrong\"")
  end

  def test_parse_single_quoted_string
    assert_equal "aarch64-apple-darwin", parse("[build]\ntarget = 'aarch64-apple-darwin'")
  end

  def test_parse_inline_table
    assert_equal "x86_64-unknown-linux-gnu", parse("build = { target = \"x86_64-unknown-linux-gnu\" }")
  end

  def test_parse_inline_table_target_not_first
    assert_equal "aarch64-apple-darwin", parse("build = { jobs = 4, target = \"aarch64-apple-darwin\" }")
  end

  def test_parse_section_header_with_comment
    assert_equal "x86_64", parse("[build] # build settings\ntarget = \"x86_64\"")
  end

  def test_parse_section_header_with_whitespace
    assert_equal "x86_64", parse("[ build ]\ntarget = \"x86_64\"")
  end

  def test_parse_quoted_section_name
    assert_equal "x86_64", parse("[\"build\"]\ntarget = \"x86_64\"")
  end

  def test_parse_quoted_key
    assert_equal "x86_64", parse("[build]\n\"target\" = \"x86_64\"")
  end

  def test_finds_config_in_ancestor
    with_tmpdir do |dir|
      child = File.join(dir, "a", "b")
      FileUtils.mkdir_p(child)
      write_config(dir, "from-ancestor")
      assert_equal "from-ancestor", isolated_build_target(child)
    end
  end

  def test_closer_config_wins
    with_tmpdir do |dir|
      child = File.join(dir, "child")
      FileUtils.mkdir_p(child)
      write_config(dir, "parent")
      write_config(child, "child")
      assert_equal "child", isolated_build_target(child)
    end
  end

  def test_config_toml_preferred_over_config
    with_tmpdir do |dir|
      cargo_dir = File.join(dir, ".cargo")
      FileUtils.mkdir_p(cargo_dir)
      File.write(File.join(cargo_dir, "config.toml"), "[build]\ntarget = \"toml\"")
      File.write(File.join(cargo_dir, "config"), "[build]\ntarget = \"legacy\"")
      assert_equal "toml", isolated_build_target(dir)
    end
  end

  def test_checks_cargo_home_env
    with_tmpdir do |dir|
      write_config(dir, "from-cargo-home")
      ENV["CARGO_HOME"] = File.join(dir, ".cargo")
      RbSys::Cargo::Config.stub(:find_in_hierarchy, nil) do
        Dir.mktmpdir do |empty_dir|
          assert_equal "from-cargo-home", RbSys::Cargo::Config.build_target(start_dir: empty_dir)
        end
      end
    ensure
      ENV.delete("CARGO_HOME")
    end
  end

  private

  def parse(content)
    RbSys::Cargo::Config.send(:parse_build_target, content)
  end

  def write_config(dir, target)
    cargo_dir = File.join(dir, ".cargo")
    FileUtils.mkdir_p(cargo_dir)
    File.write(File.join(cargo_dir, "config.toml"), "[build]\ntarget = \"#{target}\"")
  end

  def with_tmpdir
    Dir.mktmpdir { |dir| yield dir }
  end

  def isolated_build_target(start_dir)
    RbSys::Cargo::Config.stub(:find_in_home, nil) do
      RbSys::Cargo::Config.build_target(start_dir: start_dir)
    end
  end
end
